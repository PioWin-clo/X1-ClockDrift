#!/usr/bin/env bash
#
# strontium-to-json.sh — convert `x1sr read` plain-text output into the
# `strontium.json` artifact that the X1-ClockDrift dashboard widget
# consumes.
#
# Designed to be driven by cron on the Sentinel host (e.g. every
# 5 minutes), independently of the ClockDrift daemon. The daemon's
# git_pusher stages the entire `data/` directory each export cycle, so
# an atomic write into `data/strontium.json` is automatically committed
# and pushed on the next cycle alongside the other JSON files. If
# `x1sr` is unavailable or returns no rows the script silent-skips —
# the frontend widget hides itself when `strontium.json` is missing or
# fetch fails (see app.js: state.strontium).
#
# Inputs are environment-driven so the same script works in dev,
# staging, and prod without editing:
#
#   X1SR_BIN     — path to / name of the x1sr binary (default: `x1sr`).
#   OUT_DIR      — output directory; defaults to the repo's `data/`
#                  inside the daemon's git workspace.
#   OUT_FILE     — output path; defaults to `${OUT_DIR}/strontium.json`.
#   ORACLE_PDA   — Strontium oracle program-derived address rendered
#                  in the widget. The current value is hard-coded as
#                  the v1.7.0 default; override via env when the
#                  Strontium contract migrates to a new PDA.
#   X1SR_TIMEOUT — seconds before `x1sr read` is killed (default 30).
#
# Schema produced (matches the v1.7.0 widget contract):
#   {
#     "updated_at": "<ISO8601 UTC>",
#     "entries":    [ { "utc": …, "spread_ms": …, "confidence_pct": …, "slot": … }, … ],
#     "avg_spread_ms":      <number>,
#     "avg_confidence_pct": <integer>,
#     "fleet_n":            <integer>,
#     "oracle_pda":         "<base58>"
#   }
#
# Sample raw `x1sr read` table this parser accepts (Unicode box-drawing
# pipe ─ U+2502, divider ─ U+2500/U+253C):
#
#   X1 Strontium — Oracle Ring Buffer  (288 slots, ~24h history)
#     #   │ UTC Time                   │ Spread │  Conf │ Slot
#   ──────┼────────────────────────────┼────────┼───────┼──────────────
#       1 │ 2026-05-07 18:50:01.805 │   11 ms │   91% │ 48300622
#       0 │ 2026-05-07 18:45:01.013 │   13 ms │   90% │ 48299815
#     Entries: 2  │  Avg spread: 12.0ms  │  Avg confidence: 90%

set -euo pipefail

# Cron strips PATH down to almost nothing — make sure the obvious
# install locations for `x1sr` are reachable.
export PATH="${HOME}/.local/bin:/usr/local/bin:/usr/bin:/bin:${PATH:-}"
export LC_ALL="${LC_ALL:-C.UTF-8}"

X1SR_BIN="${X1SR_BIN:-x1sr}"
OUT_DIR="${OUT_DIR:-${HOME}/strontium-meter/repo/data}"
OUT_FILE="${OUT_FILE:-${OUT_DIR}/strontium.json}"
ORACLE_PDA="${ORACLE_PDA:-cfm1Tc7CNdTa8Hm8FGWAuHXaaozSjQHNmdBD5mEVN9P}"
X1SR_TIMEOUT="${X1SR_TIMEOUT:-30}"

mkdir -p "${OUT_DIR}"

# Capture the raw table. Tolerate non-zero exit + truncated output —
# the widget hides itself on empty input, so a quiet skip is fine.
# `timeout` is part of GNU coreutils on Sentinel; if it's missing
# (e.g. minimal Alpine, macOS dev box) we fall back to running x1sr
# directly. The widget's stale-data detection guards against runaway
# x1sr processes regardless.
if command -v timeout >/dev/null 2>&1; then
    RAW="$(timeout "${X1SR_TIMEOUT}s" "${X1SR_BIN}" read 2>/dev/null || true)"
else
    RAW="$("${X1SR_BIN}" read 2>/dev/null || true)"
fi
if [[ -z "${RAW}" ]]; then
    echo "strontium-to-json: empty x1sr output, skipping write" >&2
    exit 0
fi

# Walk the table line by line, skipping headers / dividers / footer.
# The unicode pipe (│, U+2502) is the column separator; awk handles
# it natively under a UTF-8 locale.
entries_json=""
while IFS= read -r line; do
    case "${line}" in
        *"#"*"UTC Time"*) continue ;;
        *"───"*)          continue ;;
        *"Entries:"*)     continue ;;
        *"Strontium"*)    continue ;;
        "")               continue ;;
    esac
    parsed="$(printf '%s\n' "${line}" | awk -F'│' '
        NF >= 5 {
            for (i = 1; i <= 5; i++) {
                gsub(/^[ \t]+|[ \t]+$/, "", $i)
            }
            printf "%s\t%s\t%s\t%s\t%s\n", $1, $2, $3, $4, $5
        }
    ')"
    [[ -z "${parsed}" ]] && continue
    IFS=$'\t' read -r idx ts spread conf slot <<< "${parsed}" || continue
    [[ -z "${idx:-}" || ! "${idx}" =~ ^[0-9]+$ ]] && continue

    # Strip unit suffixes — table prints "  11 ms" / "  91%". Slot is
    # bare digits in the rightmost cell.
    spread_num="${spread% ms}"
    spread_num="${spread_num// /}"
    conf_num="${conf%\%}"
    conf_num="${conf_num// /}"
    slot_num="${slot// /}"

    # Defensive numeric guard — drop the row if any field is non-numeric.
    [[ "${spread_num}" =~ ^[0-9]+(\.[0-9]+)?$ ]] || continue
    [[ "${conf_num}"   =~ ^[0-9]+(\.[0-9]+)?$ ]] || continue
    [[ "${slot_num}"   =~ ^[0-9]+$            ]] || continue

    # "2026-05-07 18:50:01.805" → "2026-05-07T18:50:01.805Z"
    utc_iso="${ts/ /T}Z"

    entry_obj="{\"utc\":\"${utc_iso}\",\"spread_ms\":${spread_num},\"confidence_pct\":${conf_num},\"slot\":${slot_num}}"
    if [[ -n "${entries_json}" ]]; then
        entries_json="${entries_json},${entry_obj}"
    else
        entries_json="${entry_obj}"
    fi
done <<< "${RAW}"

# Footer:  Entries: 2  │  Avg spread: 12.0ms  │  Avg confidence: 90%
footer="$(printf '%s\n' "${RAW}" | grep -E '^[[:space:]]*Entries:' | head -n1 || true)"
fleet_n="$(printf '%s' "${footer}"   | grep -oE 'Entries:[[:space:]]*[0-9]+'         | grep -oE '[0-9]+'         | head -n1 || true)"
avg_spread="$(printf '%s' "${footer}" | grep -oE 'Avg spread:[[:space:]]*[0-9.]+'    | grep -oE '[0-9.]+'        | head -n1 || true)"
avg_conf="$(printf '%s' "${footer}"   | grep -oE 'Avg confidence:[[:space:]]*[0-9]+' | grep -oE '[0-9]+'         | head -n1 || true)"
fleet_n="${fleet_n:-0}"
avg_spread="${avg_spread:-0}"
avg_conf="${avg_conf:-0}"

updated_at="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"

# Atomic replace so a half-written file never leaks to the daemon's
# next git push. tmp file is created next to the destination so the
# rename stays on the same filesystem.
TMP_FILE="$(mktemp "${OUT_FILE}.XXXXXX")"
trap 'rm -f "${TMP_FILE}"' EXIT
cat > "${TMP_FILE}" <<JSON
{
  "updated_at": "${updated_at}",
  "entries": [${entries_json}],
  "avg_spread_ms": ${avg_spread},
  "avg_confidence_pct": ${avg_conf},
  "fleet_n": ${fleet_n},
  "oracle_pda": "${ORACLE_PDA}"
}
JSON
mv -f "${TMP_FILE}" "${OUT_FILE}"
trap - EXIT

echo "strontium-to-json: wrote ${OUT_FILE} (fleet_n=${fleet_n}, avg_spread=${avg_spread}ms, avg_conf=${avg_conf}%)" >&2
