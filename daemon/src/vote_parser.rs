use serde::Deserialize;
use serde_json::Value;

pub const VOTE_PROGRAM_ID: &str = "Vote111111111111111111111111111111111111111";

#[derive(Debug, Clone)]
pub struct ParsedVote {
    pub vote_account: String,
    #[allow(dead_code)]
    pub vote_authority: String,
    pub timestamp: i64,
    pub last_voted_slot: u64,
}

#[derive(Deserialize)]
struct ParsedInstruction {
    parsed: Option<ParsedContent>,
    #[serde(rename = "programId", default)]
    program_id: String,
}

#[derive(Deserialize)]
struct ParsedContent {
    #[serde(rename = "type")]
    instr_type: String,
    info: ParsedInfo,
}

#[derive(Deserialize)]
struct ParsedInfo {
    #[serde(rename = "voteAccount")]
    vote_account: String,
    #[serde(rename = "voteAuthority")]
    vote_authority: Option<String>,
    #[serde(rename = "towerSync")]
    tower_sync: Option<TowerSyncData>,
    #[serde(rename = "voteStateUpdate")]
    vote_state_update: Option<VoteStateUpdateData>,
    vote: Option<LegacyVoteData>,
    #[serde(rename = "compactUpdateVoteState")]
    compact_update_vote_state: Option<VoteStateUpdateData>,
}

#[derive(Deserialize)]
struct TowerSyncData {
    timestamp: Option<i64>,
    lockouts: Vec<Lockout>,
}

#[derive(Deserialize)]
struct VoteStateUpdateData {
    timestamp: Option<i64>,
    lockouts: Vec<Lockout>,
}

#[derive(Deserialize)]
struct LegacyVoteData {
    timestamp: Option<i64>,
    slots: Vec<u64>,
}

#[derive(Deserialize)]
struct Lockout {
    slot: u64,
}

/// Parse a single instruction from a `jsonParsed`-encoded transaction.
/// Returns Some only if it's a vote instruction with both a timestamp and
/// at least one lockout/slot we can read.
pub fn parse_instruction(instr_value: &Value) -> Option<ParsedVote> {
    let instr: ParsedInstruction = serde_json::from_value(instr_value.clone()).ok()?;

    if instr.program_id != VOTE_PROGRAM_ID {
        return None;
    }

    let parsed = instr.parsed?;
    let info = parsed.info;

    let (timestamp, last_slot) = if let Some(ts) = info.tower_sync {
        (ts.timestamp?, ts.lockouts.iter().map(|l| l.slot).max()?)
    } else if let Some(c) = info.compact_update_vote_state {
        (c.timestamp?, c.lockouts.iter().map(|l| l.slot).max()?)
    } else if let Some(v) = info.vote_state_update {
        (v.timestamp?, v.lockouts.iter().map(|l| l.slot).max()?)
    } else if let Some(v) = info.vote {
        (v.timestamp?, v.slots.iter().copied().max()?)
    } else {
        tracing::debug!(
            instr_type = %parsed.instr_type,
            "vote instruction has no recognized payload"
        );
        return None;
    };

    let vote_authority = info
        .vote_authority
        .unwrap_or_else(|| info.vote_account.clone());

    Some(ParsedVote {
        vote_account: info.vote_account,
        vote_authority,
        timestamp,
        last_voted_slot: last_slot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_towersync() {
        let raw = r#"{
            "parsed": {
                "type": "towersync",
                "info": {
                    "voteAccount": "AgR7NMtttfGyBPfZsGpp7EbsqQUoLzFnwuhs6J9Bmngo",
                    "voteAuthority": "3wNQsfS9L6nSm59dU2qryvR74GJaq27iqetkXUxQFChX",
                    "towerSync": {
                        "blockId": "8Tob7PNEYjM3ZQuo2cxiThpzvDapmxt9EivdJumv4AzC",
                        "hash": "ARsLDJs5id5uw8yBHC2VjJ2ZZoH3P94pon4KnP1zYDTk",
                        "lockouts": [
                            {"confirmation_count": 31, "slot": 46219523},
                            {"confirmation_count": 1, "slot": 46219553}
                        ],
                        "root": 46219522,
                        "timestamp": 1777394172
                    }
                }
            },
            "program": "vote",
            "programId": "Vote111111111111111111111111111111111111111",
            "stackHeight": null
        }"#;
        let v: Value = serde_json::from_str(raw).unwrap();
        let rec = parse_instruction(&v).unwrap();
        assert_eq!(rec.vote_account, "AgR7NMtttfGyBPfZsGpp7EbsqQUoLzFnwuhs6J9Bmngo");
        assert_eq!(rec.vote_authority, "3wNQsfS9L6nSm59dU2qryvR74GJaq27iqetkXUxQFChX");
        assert_eq!(rec.timestamp, 1777394172);
        assert_eq!(rec.last_voted_slot, 46219553);
    }

    #[test]
    fn ignores_non_vote_instruction() {
        let raw = r#"{
            "parsed": {"type": "transfer", "info": {"source": "x", "destination": "y", "lamports": 1}},
            "program": "system",
            "programId": "11111111111111111111111111111111"
        }"#;
        let v: Value = serde_json::from_str(raw).unwrap();
        assert!(parse_instruction(&v).is_none());
    }

    #[test]
    fn skips_vote_instruction_without_timestamp() {
        let raw = r#"{
            "parsed": {
                "type": "towersync",
                "info": {
                    "voteAccount": "VA",
                    "voteAuthority": "AUTH",
                    "towerSync": {
                        "lockouts": [{"confirmation_count": 1, "slot": 100}],
                        "root": 99
                    }
                }
            },
            "program": "vote",
            "programId": "Vote111111111111111111111111111111111111111"
        }"#;
        let v: Value = serde_json::from_str(raw).unwrap();
        assert!(parse_instruction(&v).is_none());
    }

    #[test]
    fn falls_back_to_vote_account_when_authority_missing() {
        let raw = r#"{
            "parsed": {
                "type": "towersync",
                "info": {
                    "voteAccount": "VA",
                    "towerSync": {
                        "lockouts": [{"confirmation_count": 1, "slot": 100}],
                        "root": 99,
                        "timestamp": 1777000000
                    }
                }
            },
            "program": "vote",
            "programId": "Vote111111111111111111111111111111111111111"
        }"#;
        let v: Value = serde_json::from_str(raw).unwrap();
        let rec = parse_instruction(&v).unwrap();
        assert_eq!(rec.vote_account, "VA");
        assert_eq!(rec.vote_authority, "VA");
    }

    /// Integration-style test against a captured X1 mainnet block.
    /// Skips silently when the fixture is not present so CI stays green.
    /// Drop the raw `getBlock` JSON-RPC response (the whole envelope, with
    /// `result` at the root) at `daemon/tests/fixtures/block_46219555.json`.
    #[test]
    fn integration_block_46219555() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/block_46219555.json");
        if !path.exists() {
            eprintln!("skipping: fixture not present at {}", path.display());
            return;
        }
        let bytes = std::fs::read(&path).expect("read fixture");
        let envelope: Value = serde_json::from_slice(&bytes).expect("parse fixture");
        let block = envelope.get("result").unwrap_or(&envelope);

        let block_time = block
            .get("blockTime")
            .and_then(|v| v.as_i64())
            .expect("fixture must have blockTime");

        let mut votes: Vec<ParsedVote> = Vec::new();
        if let Some(txs) = block.get("transactions").and_then(|v| v.as_array()) {
            for tx in txs {
                if let Some(ixs) = tx
                    .pointer("/transaction/message/instructions")
                    .and_then(|v| v.as_array())
                {
                    for ix in ixs {
                        if let Some(rec) = parse_instruction(ix) {
                            votes.push(rec);
                        }
                    }
                }
            }
        }

        assert!(
            (1500..=2500).contains(&votes.len()),
            "expected 1500–2500 vote records, got {}",
            votes.len()
        );

        // Hard sanity bound: any timestamp >1h from blockTime almost certainly
        // means the parser is reading the wrong field. Per-validator drift
        // (the thing we're measuring) is the median, not individual extremes —
        // some validators legitimately run tens of seconds off, so we cannot
        // assert ±10s on every sample.
        let mut deltas: Vec<i64> = votes
            .iter()
            .map(|v| (v.timestamp - block_time).abs())
            .collect();
        for v in &votes {
            let delta = (v.timestamp - block_time).abs();
            assert!(
                delta <= 3600,
                "vote ts {} differs from block_time {} by {}s (>1h) for {}",
                v.timestamp,
                block_time,
                delta,
                v.vote_account
            );
        }
        deltas.sort();
        let median = deltas[deltas.len() / 2];
        assert!(
            median <= 10,
            "median |ts - blockTime| was {median}s; expected ≤10s for a healthy network"
        );
    }
}
