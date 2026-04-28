"use strict";

const PAGE_SIZE = 50;

const state = {
  summary: null,
  validators: [],
  history: [],
  meta: null,
  filtered: [],
  page: 0,
  sortKey: "mean_drift_ms_abs",
  sortDir: -1,
  query: "",
};

const el = {
  updated: document.getElementById("updated"),
  heroValue: document.getElementById("hero-value"),
  heroSub: document.getElementById("hero-sub"),
  cardValidators: document.getElementById("card-validators"),
  cardSamples: document.getElementById("card-samples"),
  cardDrift1s: document.getElementById("card-drift1s"),
  cardDrift1sPct: document.getElementById("card-drift1s-pct"),
  cardStakeDrift: document.getElementById("card-stake-drift"),
  rankingBody: document.getElementById("ranking-body"),
  pageInfo: document.getElementById("page-info"),
  prev: document.getElementById("prev"),
  next: document.getElementById("next"),
  search: document.getElementById("search"),
  metaLine: document.getElementById("meta-line"),
};

document.querySelectorAll("table.ranking th").forEach((th) => {
  th.addEventListener("click", () => {
    const key = th.dataset.sort;
    if (!key) return;
    if (state.sortKey === key || (key === "mean_drift_ms" && state.sortKey === "mean_drift_ms_abs")) {
      state.sortDir = -state.sortDir;
    } else {
      state.sortKey = key;
      state.sortDir = key === "pubkey" ? 1 : -1;
    }
    state.page = 0;
    renderTable();
  });
});

el.prev.addEventListener("click", () => {
  if (state.page > 0) { state.page--; renderTable(); }
});
el.next.addEventListener("click", () => {
  const last = Math.max(0, Math.ceil(state.filtered.length / PAGE_SIZE) - 1);
  if (state.page < last) { state.page++; renderTable(); }
});

el.search.addEventListener("input", () => {
  state.query = el.search.value.trim().toLowerCase();
  state.page = 0;
  applyFilter();
  renderTable();
});

async function loadAll() {
  try {
    const [summary, validators, history, meta] = await Promise.all([
      fetchJSON("data/summary.json"),
      fetchJSON("data/validators.json"),
      fetchJSON("data/history.json"),
      fetchJSON("data/meta.json"),
    ]);
    state.summary = summary;
    state.validators = validators || [];
    state.history = history || [];
    state.meta = meta;
    renderAll();
  } catch (e) {
    console.error("load failed", e);
    el.updated.textContent = "data unavailable";
  }
}

async function fetchJSON(path) {
  const r = await fetch(path, { cache: "no-store" });
  if (!r.ok) throw new Error(`${path}: ${r.status}`);
  return await r.json();
}

function renderAll() {
  renderHeader();
  renderHero();
  renderCards();
  renderHistoryChart();
  renderHistogram();
  applyFilter();
  renderTable();
  renderFooter();
}

function renderHeader() {
  if (state.summary && state.summary.generated_at_utc) {
    el.updated.textContent = `updated ${state.summary.generated_at_utc}`;
  }
}

function renderHero() {
  if (!state.summary) return;
  const ms = state.summary.median_drift_ms;
  el.heroValue.textContent = formatMs(ms);
  el.heroValue.className = "hero-value " + colorClass(Math.abs(ms));
  el.heroSub.textContent = `median across ${formatInt(state.summary.n_validators_observed)} validators · stake-weighted ${formatMs(state.summary.stake_weighted_drift_ms)}`;
}

function renderCards() {
  if (!state.summary) return;
  el.cardValidators.textContent = formatInt(state.summary.n_validators_observed);
  el.cardSamples.textContent = formatInt(state.summary.n_samples_24h);
  el.cardDrift1s.textContent = formatInt(state.summary.validators_with_drift_over_1s);
  const total = state.summary.n_validators_observed || 0;
  const pct = total > 0 ? (100 * state.summary.validators_with_drift_over_1s / total) : 0;
  el.cardDrift1sPct.textContent = `${pct.toFixed(1)}% (>5s: ${formatInt(state.summary.validators_with_drift_over_5s)})`;
  el.cardStakeDrift.textContent = formatMs(state.summary.stake_weighted_drift_ms);
}

let chartHistory = null;
function renderHistoryChart() {
  const ctx = document.getElementById("chart-history");
  if (!ctx || !window.Chart) return;
  const data = state.history || [];
  const labels = data.map((d) => d.bucket_iso);
  const median = data.map((d) => d.median_drift_ms);
  const stakeW = data.map((d) => d.stake_weighted_drift_ms);
  if (chartHistory) chartHistory.destroy();
  chartHistory = new Chart(ctx, {
    type: "line",
    data: {
      labels,
      datasets: [
        { label: "median", data: median, borderColor: "#58a6ff", backgroundColor: "transparent", pointRadius: 0, borderWidth: 1.5, tension: 0.2 },
        { label: "stake-weighted", data: stakeW, borderColor: "#3fb950", backgroundColor: "transparent", pointRadius: 0, borderWidth: 1.5, tension: 0.2 },
      ],
    },
    options: chartCommonOpts({ yLabel: "drift (ms)" }),
  });
}

let chartHistogram = null;
function renderHistogram() {
  const ctx = document.getElementById("chart-histogram");
  if (!ctx || !window.Chart) return;
  const buckets = buildHistogram(state.validators.map((v) => v.mean_drift_ms));
  if (chartHistogram) chartHistogram.destroy();
  chartHistogram = new Chart(ctx, {
    type: "bar",
    data: {
      labels: buckets.labels,
      datasets: [{ label: "validators", data: buckets.counts, backgroundColor: "#58a6ff" }],
    },
    options: chartCommonOpts({ yLabel: "validators" }),
  });
}

function buildHistogram(values) {
  if (values.length === 0) return { labels: [], counts: [] };
  const edges = [-10000, -5000, -2000, -1000, -500, -200, -100, -50, -10, 10, 50, 100, 200, 500, 1000, 2000, 5000, 10000];
  const counts = new Array(edges.length + 1).fill(0);
  for (const v of values) {
    let placed = false;
    for (let i = 0; i < edges.length; i++) {
      if (v < edges[i]) { counts[i]++; placed = true; break; }
    }
    if (!placed) counts[edges.length]++;
  }
  const labels = [];
  labels.push(`< ${edges[0]}`);
  for (let i = 0; i < edges.length - 1; i++) labels.push(`${edges[i]}…${edges[i + 1]}`);
  labels.push(`≥ ${edges[edges.length - 1]}`);
  return { labels, counts };
}

function chartCommonOpts({ yLabel }) {
  return {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { labels: { color: "#c9d1d9" } },
      tooltip: { mode: "index", intersect: false },
    },
    scales: {
      x: { ticks: { color: "#8b949e", maxTicksLimit: 8 }, grid: { color: "#21262d" } },
      y: { ticks: { color: "#8b949e" }, grid: { color: "#21262d" }, title: { display: true, text: yLabel, color: "#8b949e" } },
    },
  };
}

function applyFilter() {
  const q = state.query;
  state.filtered = q
    ? state.validators.filter((v) => v.pubkey.toLowerCase().includes(q))
    : state.validators.slice();
  sortFiltered();
}

function sortFiltered() {
  const key = state.sortKey;
  const dir = state.sortDir;
  state.filtered.sort((a, b) => {
    let av, bv;
    if (key === "mean_drift_ms_abs") { av = Math.abs(a.mean_drift_ms); bv = Math.abs(b.mean_drift_ms); }
    else if (key === "rank") { return 0; }
    else { av = a[key]; bv = b[key]; }
    if (typeof av === "string") return dir * av.localeCompare(bv);
    return dir * ((av || 0) - (bv || 0));
  });
}

function renderTable() {
  sortFiltered();
  const start = state.page * PAGE_SIZE;
  const slice = state.filtered.slice(start, start + PAGE_SIZE);
  el.rankingBody.innerHTML = "";
  slice.forEach((v, i) => {
    const tr = document.createElement("tr");
    tr.appendChild(td(String(start + i + 1)));
    tr.appendChild(td(shorten(v.pubkey), { mono: true, title: v.pubkey }));
    tr.appendChild(driftTd(v.mean_drift_ms));
    tr.appendChild(driftTd(v.median_drift_ms));
    tr.appendChild(td(formatMsRaw(v.stddev_drift_ms), { num: true }));
    tr.appendChild(driftTd(v.p10_drift_ms));
    tr.appendChild(driftTd(v.p90_drift_ms));
    tr.appendChild(td(formatInt(v.n_samples), { num: true }));
    tr.appendChild(td(formatNum(v.stake_xnt, 0), { num: true }));
    tr.appendChild(td(formatNum(v.weighted_impact_ms_xnt, 0), { num: true }));
    el.rankingBody.appendChild(tr);
  });
  const last = Math.max(0, Math.ceil(state.filtered.length / PAGE_SIZE) - 1);
  el.pageInfo.textContent = `page ${state.page + 1} / ${last + 1} (${state.filtered.length} validators)`;
  el.prev.disabled = state.page === 0;
  el.next.disabled = state.page >= last;
}

function td(text, opts) {
  opts = opts || {};
  const e = document.createElement("td");
  e.textContent = text;
  if (opts.num) e.classList.add("num");
  if (opts.title) e.title = opts.title;
  return e;
}
function driftTd(ms) {
  const e = td(formatMsRaw(ms), { num: true });
  if (ms > 100) e.classList.add("pos");
  else if (ms < -100) e.classList.add("neg");
  else e.classList.add("zero");
  return e;
}

function shorten(s) {
  if (!s || s.length <= 16) return s || "";
  return `${s.slice(0, 6)}…${s.slice(-6)}`;
}

function colorClass(absMs) {
  if (absMs < 200) return "good";
  if (absMs < 1000) return "warn";
  return "bad";
}

function formatMs(ms) {
  if (ms === undefined || ms === null || Number.isNaN(ms)) return "—";
  const sign = ms >= 0 ? "+" : "−";
  const v = Math.abs(ms);
  if (v >= 1000) return `${sign}${(v / 1000).toFixed(2)}s`;
  return `${sign}${v.toFixed(0)} ms`;
}
function formatMsRaw(ms) {
  if (ms === undefined || ms === null || Number.isNaN(ms)) return "—";
  return ms.toFixed(1);
}
function formatInt(n) {
  if (n === undefined || n === null) return "—";
  return Number(n).toLocaleString("en-US");
}
function formatNum(n, decimals) {
  if (n === undefined || n === null || Number.isNaN(n)) return "—";
  return Number(n).toLocaleString("en-US", { maximumFractionDigits: decimals, minimumFractionDigits: decimals });
}

function renderFooter() {
  if (!state.meta) { el.metaLine.textContent = ""; return; }
  el.metaLine.textContent =
    `daemon ${state.meta.daemon_version}` +
    ` · ${formatInt(state.meta.total_slots_observed)} slots / ${formatInt(state.meta.total_votes_collected)} votes` +
    ` · last ${state.meta.generated_at_utc}`;
}

loadAll();
setInterval(loadAll, 60_000);
