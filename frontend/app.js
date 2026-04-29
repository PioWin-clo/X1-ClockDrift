"use strict";

const PAGE_SIZE = 50;

const I18N = {
  en: {
    tagline: 'Live drift of <code>Clock::unix_timestamp</code> on the X1 blockchain.',
    updated: 'updated',
    hero_label: 'Network drift right now',
    hero_sub: (n, sw) => `median across ${n} validators · stake-weighted ${sw} ms`,
    card_validators: 'Validators observed (24h)',
    card_samples: 'Samples (24h)',
    card_drift1s: 'Drift > 1s',
    card_stake_drift: 'Stake-weighted drift',
    chart_history_title: 'Network drift over time (last 7 days, 5-minute buckets)',
    chart_histogram_title: 'Validator drift distribution (mean drift, top 500)',
    best_synced_title: 'Best synchronized validators (top 10)',
    best_synced_help: 'Validators with the smallest absolute drift from network consensus. Minimum 5 samples required.',
    ranking_title: 'Validator ranking (sorted by impact: drift × stake)',
    ranking_search_placeholder: 'search by pubkey…',
    col_rank: '#',
    col_pubkey: 'pubkey',
    col_mean: 'mean (ms)',
    col_median: 'median (ms)',
    col_stddev: 'stddev (ms)',
    col_p10: 'p10 (ms)',
    col_p90: 'p90 (ms)',
    col_n: 'n',
    col_stake: 'stake (XNT)',
    col_impact: 'impact',
    clock_section_title: 'My clock vs world',
    clock_wall: 'Wall clock (your browser)',
    clock_offset_label: 'Sentinel offset to UTC consensus',
    clock_rms_label: 'RMS offset (long-term stability)',
    clock_stratum_label: 'Stratum',
    clock_reference: 'Reference source',
    clock_skew_label: 'Frequency skew',
    clock_help_text: "Sentinel's system clock is compared against a consensus of independent stratum-1 atomic time sources (PTB Germany, GUM Poland, CESNET Czechia, Netnod Sweden). All X1 chain drift measurements on this dashboard are referenced against this clock.",
    sources_section_title: 'NTP sources (chrony calibration)',
    sources_col_operator: 'Operator',
    sources_col_country: 'Country',
    sources_col_hostname: 'Hostname',
    sources_col_stratum: 'Stratum',
    sources_col_state: 'State',
    sources_col_offset: 'Offset (µs)',
    sources_col_last_rx: 'Last RX (s)',
    footer_repo: 'github',
    footer_methodology: 'methodology',
    prev_page: 'prev',
    next_page: 'next',
    page_info: (p, t) => `page ${p} of ${t}`,
  },
  pl: {
    tagline: 'Bieżący dryf <code>Clock::unix_timestamp</code> na blockchainie X1.',
    updated: 'aktualizacja',
    hero_label: 'Dryf sieci teraz',
    hero_sub: (n, sw) => `mediana z ${n} walidatorów · ważone stakiem ${sw} ms`,
    card_validators: 'Obserwowani walidatorzy (24h)',
    card_samples: 'Próbki (24h)',
    card_drift1s: 'Dryf > 1s',
    card_stake_drift: 'Dryf ważony stakiem',
    chart_history_title: 'Dryf sieci w czasie (ostatnie 7 dni, kubełki 5-minutowe)',
    chart_histogram_title: 'Rozkład dryfu walidatorów (średni dryf, top 500)',
    best_synced_title: 'Najlepiej zsynchronizowani walidatorzy (top 10)',
    best_synced_help: 'Walidatorzy z najmniejszym bezwzględnym dryfem od konsensusu sieci. Wymagane minimum 5 próbek.',
    ranking_title: 'Ranking walidatorów (sortowanie po wpływie: dryf × stake)',
    ranking_search_placeholder: 'szukaj po pubkey…',
    col_rank: '#',
    col_pubkey: 'pubkey',
    col_mean: 'średnia (ms)',
    col_median: 'mediana (ms)',
    col_stddev: 'odch.std (ms)',
    col_p10: 'p10 (ms)',
    col_p90: 'p90 (ms)',
    col_n: 'n',
    col_stake: 'stake (XNT)',
    col_impact: 'wpływ',
    clock_section_title: 'Mój zegar vs świat',
    clock_wall: 'Zegar (twoja przeglądarka)',
    clock_offset_label: 'Odchylenie Sentinela od konsensusu UTC',
    clock_rms_label: 'RMS odchylenia (stabilność długoterminowa)',
    clock_stratum_label: 'Stratum',
    clock_reference: 'Źródło referencyjne',
    clock_skew_label: 'Skew częstotliwości',
    clock_help_text: 'Zegar systemowy serwera Sentinel jest porównywany z konsensusem niezależnych źródeł czasu stratum-1 (PTB Niemcy, GUM Polska, CESNET Czechy, Netnod Szwecja). Wszystkie pomiary dryfu czasu X1 na tym dashboardzie są odniesione do tego zegara.',
    sources_section_title: 'Źródła NTP (kalibracja chrony)',
    sources_col_operator: 'Operator',
    sources_col_country: 'Kraj',
    sources_col_hostname: 'Host',
    sources_col_stratum: 'Stratum',
    sources_col_state: 'Stan',
    sources_col_offset: 'Odchylenie (µs)',
    sources_col_last_rx: 'Ostatni RX (s)',
    footer_repo: 'github',
    footer_methodology: 'metodologia',
    prev_page: 'poprzednia',
    next_page: 'następna',
    page_info: (p, t) => `strona ${p} z ${t}`,
  },
};

const state = {
  lang: 'en',
  summary: null,
  validators: [],
  history: [],
  meta: null,
  best: [],
  chrony: null,
  filtered: [],
  page: 0,
  sortKey: 'mean_drift_ms_abs',
  sortDir: -1,
  query: '',
};

const el = {
  updatedTs: document.getElementById('updated-ts'),
  heroValue: document.getElementById('hero-value'),
  heroSub: document.getElementById('hero-sub'),
  cardValidators: document.getElementById('card-validators'),
  cardSamples: document.getElementById('card-samples'),
  cardDrift1s: document.getElementById('card-drift1s'),
  cardDrift1sPct: document.getElementById('card-drift1s-pct'),
  cardStakeDrift: document.getElementById('card-stake-drift'),
  rankingBody: document.getElementById('ranking-body'),
  pageInfo: document.getElementById('page-info'),
  prev: document.getElementById('prev'),
  next: document.getElementById('next'),
  search: document.getElementById('search'),
  metaLine: document.getElementById('meta-line'),
  bestSyncedBody: document.getElementById('best-synced-body'),
  sourcesBody: document.getElementById('sources-body'),
  clockWall: document.getElementById('clock-wall-time'),
  clockOffset: document.getElementById('clock-offset-value'),
  clockRms: document.getElementById('clock-rms-value'),
  clockStratum: document.getElementById('clock-stratum-value'),
  clockReference: document.getElementById('clock-reference-value'),
  clockSkew: document.getElementById('clock-skew-value'),
  btnLangEn: document.getElementById('btn-lang-en'),
  btnLangPl: document.getElementById('btn-lang-pl'),
};

function initLanguage() {
  let lang = localStorage.getItem('lang');
  if (!lang) {
    const browserLang = (navigator.language || 'en').toLowerCase();
    lang = browserLang.startsWith('pl') ? 'pl' : 'en';
    localStorage.setItem('lang', lang);
  }
  state.lang = lang === 'pl' ? 'pl' : 'en';
}

function applyI18n() {
  const lang = state.lang;
  const t = I18N[lang];
  document.documentElement.lang = lang;

  document.querySelectorAll('[data-i18n]').forEach((node) => {
    const key = node.dataset.i18n;
    const value = t[key];
    if (typeof value === 'string') {
      node.textContent = value;
    }
  });
  document.querySelectorAll('[data-i18n-html]').forEach((node) => {
    const key = node.dataset.i18nHtml;
    const value = t[key];
    if (typeof value === 'string') {
      node.innerHTML = value;
    }
  });
  document.querySelectorAll('[data-i18n-attr-placeholder]').forEach((node) => {
    const key = node.dataset.i18nAttrPlaceholder;
    const value = t[key];
    if (typeof value === 'string') {
      node.placeholder = value;
    }
  });

  el.btnLangEn.classList.toggle('active', lang === 'en');
  el.btnLangPl.classList.toggle('active', lang === 'pl');
}

function setLanguage(lang) {
  state.lang = lang;
  localStorage.setItem('lang', lang);
  applyI18n();
  renderAll();
}

el.btnLangEn.addEventListener('click', () => setLanguage('en'));
el.btnLangPl.addEventListener('click', () => setLanguage('pl'));

document.querySelectorAll('table.ranking th').forEach((th) => {
  th.addEventListener('click', () => {
    const key = th.dataset.sort;
    if (!key) return;
    if (state.sortKey === key || (key === 'mean_drift_ms' && state.sortKey === 'mean_drift_ms_abs')) {
      state.sortDir = -state.sortDir;
    } else {
      state.sortKey = key;
      state.sortDir = key === 'pubkey' ? 1 : -1;
    }
    state.page = 0;
    renderTable();
  });
});

el.prev.addEventListener('click', () => {
  if (state.page > 0) {
    state.page--;
    renderTable();
  }
});
el.next.addEventListener('click', () => {
  const last = Math.max(0, Math.ceil(state.filtered.length / PAGE_SIZE) - 1);
  if (state.page < last) {
    state.page++;
    renderTable();
  }
});

el.search.addEventListener('input', () => {
  state.query = el.search.value.trim().toLowerCase();
  state.page = 0;
  applyFilter();
  renderTable();
});

async function loadAll() {
  try {
    const [summary, validators, history, meta, best, chrony] = await Promise.all([
      fetchJSON('data/summary.json'),
      fetchJSON('data/validators.json'),
      fetchJSON('data/history.json'),
      fetchJSON('data/meta.json'),
      fetchJSONOptional('data/best_validators.json'),
      fetchJSONOptional('data/chrony.json'),
    ]);
    state.summary = summary;
    state.validators = validators || [];
    state.history = history || [];
    state.meta = meta;
    state.best = best || [];
    state.chrony = chrony;
    renderAll();
  } catch (e) {
    console.error('load failed', e);
    el.updatedTs.textContent = '—';
  }
}

async function fetchJSON(path) {
  const r = await fetch(path, { cache: 'no-store' });
  if (!r.ok) throw new Error(`${path}: ${r.status}`);
  return await r.json();
}

async function fetchJSONOptional(path) {
  try {
    const r = await fetch(path, { cache: 'no-store' });
    if (!r.ok) return null;
    return await r.json();
  } catch (e) {
    console.warn(`optional fetch failed: ${path}`, e);
    return null;
  }
}

function renderAll() {
  renderHeader();
  renderClock();
  renderHero();
  renderCards();
  renderHistoryChart();
  renderHistogram();
  renderBestSynced();
  applyFilter();
  renderTable();
  renderSources();
  renderFooter();
}

function renderHeader() {
  if (state.summary && state.summary.generated_at_utc) {
    el.updatedTs.textContent = state.summary.generated_at_utc;
  }
}

function renderClock() {
  if (!state.chrony) {
    el.clockOffset.textContent = '—';
    el.clockRms.textContent = '—';
    el.clockStratum.textContent = '—';
    el.clockReference.textContent = '—';
    el.clockSkew.textContent = '—';
    return;
  }
  const t = state.chrony.tracking;
  if (!t || t.system_offset_us == null) {
    el.clockOffset.textContent = '—';
    el.clockRms.textContent = '—';
    el.clockStratum.textContent = '—';
    el.clockReference.textContent = '—';
    el.clockSkew.textContent = '—';
    return;
  }
  el.clockOffset.textContent = formatSignedMicros(t.system_offset_us);
  el.clockOffset.className = 'clock-cell-value mono ' + offsetColorClass(Math.abs(t.system_offset_us));
  el.clockRms.textContent = (t.rms_offset_us == null) ? '—' : `${formatInt(t.rms_offset_us)} µs`;
  el.clockStratum.textContent = (t.stratum == null) ? '—' : String(t.stratum);
  if (t.reference_hostname) {
    const opPart = t.reference_operator ? ` (${t.reference_operator})` : '';
    el.clockReference.textContent = `${t.reference_hostname}${opPart}`;
  } else if (t.reference_ip) {
    el.clockReference.textContent = t.reference_ip;
  } else {
    el.clockReference.textContent = '—';
  }
  el.clockSkew.textContent = (t.skew_ppm == null) ? '—' : `${Number(t.skew_ppm).toFixed(3)} ppm`;
}

function renderSources() {
  el.sourcesBody.innerHTML = '';
  if (!state.chrony || !Array.isArray(state.chrony.sources)) return;
  const lang = state.lang;
  state.chrony.sources.forEach((s) => {
    const tr = document.createElement('tr');
    tr.appendChild(td(s.operator || '—'));
    tr.appendChild(td(s.country_code || '—'));
    tr.appendChild(td(s.hostname || s.ip || '—', { mono: true }));
    tr.appendChild(td(s.stratum != null ? String(s.stratum) : '—', { num: true }));
    const stateLabel = lang === 'pl' ? s.state_label_pl : s.state_label_en;
    const stateTd = td(stateLabel || s.state || '—');
    tr.appendChild(stateTd);
    tr.appendChild(
      td(s.offset_us != null ? formatSignedMicros(s.offset_us) : '—', { num: true })
    );
    tr.appendChild(td(s.last_rx_secs != null ? String(s.last_rx_secs) : '—', { num: true }));
    el.sourcesBody.appendChild(tr);
  });
}

function tickWallClock() {
  const now = new Date();
  const yyyy = now.getUTCFullYear();
  const mm = String(now.getUTCMonth() + 1).padStart(2, '0');
  const dd = String(now.getUTCDate()).padStart(2, '0');
  const hh = String(now.getUTCHours()).padStart(2, '0');
  const mi = String(now.getUTCMinutes()).padStart(2, '0');
  const ss = String(now.getUTCSeconds()).padStart(2, '0');
  const ms = String(now.getUTCMilliseconds()).padStart(3, '0');
  if (el.clockWall) {
    el.clockWall.textContent = `${yyyy}-${mm}-${dd} ${hh}:${mi}:${ss}.${ms} UTC`;
  }
}

function renderHero() {
  if (!state.summary) return;
  const ms = state.summary.median_drift_ms;
  el.heroValue.textContent = formatMs(ms);
  el.heroValue.className = 'hero-value ' + colorClass(Math.abs(ms));
  const t = I18N[state.lang];
  el.heroSub.textContent = t.hero_sub(
    formatInt(state.summary.n_validators_observed),
    formatNum(state.summary.stake_weighted_drift_ms, 1)
  );
}

function renderCards() {
  if (!state.summary) return;
  el.cardValidators.textContent = formatInt(state.summary.n_validators_observed);
  el.cardSamples.textContent = formatInt(state.summary.n_samples_24h);
  el.cardDrift1s.textContent = formatInt(state.summary.validators_with_drift_over_1s);
  const total = state.summary.n_validators_observed || 0;
  const pct = total > 0 ? (100 * state.summary.validators_with_drift_over_1s) / total : 0;
  el.cardDrift1sPct.textContent = `${pct.toFixed(1)}% (>5s: ${formatInt(state.summary.validators_with_drift_over_5s)})`;
  el.cardStakeDrift.textContent = formatMs(state.summary.stake_weighted_drift_ms);
}

let chartHistory = null;
function renderHistoryChart() {
  const ctx = document.getElementById('chart-history');
  if (!ctx || !window.Chart) return;
  const data = state.history || [];
  const labels = data.map((d) => d.bucket_iso);
  const median = data.map((d) => d.median_drift_ms);
  const stakeW = data.map((d) => d.stake_weighted_drift_ms);
  if (chartHistory) chartHistory.destroy();
  chartHistory = new Chart(ctx, {
    type: 'line',
    data: {
      labels,
      datasets: [
        { label: 'median', data: median, borderColor: '#58a6ff', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2 },
        { label: 'stake-weighted', data: stakeW, borderColor: '#3fb950', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2 },
      ],
    },
    options: chartCommonOpts({ yLabel: 'drift (ms)' }),
  });
}

let chartHistogram = null;
function renderHistogram() {
  const ctx = document.getElementById('chart-histogram');
  if (!ctx || !window.Chart) return;
  const buckets = buildHistogram(state.validators.map((v) => v.mean_drift_ms));
  if (chartHistogram) chartHistogram.destroy();
  chartHistogram = new Chart(ctx, {
    type: 'bar',
    data: {
      labels: buckets.labels,
      datasets: [{ label: 'validators', data: buckets.counts, backgroundColor: '#58a6ff' }],
    },
    options: chartCommonOpts({ yLabel: 'validators' }),
  });
}

function buildHistogram(values) {
  if (values.length === 0) return { labels: [], counts: [] };
  const edges = [-10000, -5000, -2000, -1000, -500, -200, -100, -50, -10, 10, 50, 100, 200, 500, 1000, 2000, 5000, 10000];
  const counts = new Array(edges.length + 1).fill(0);
  for (const v of values) {
    let placed = false;
    for (let i = 0; i < edges.length; i++) {
      if (v < edges[i]) {
        counts[i]++;
        placed = true;
        break;
      }
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
      legend: { labels: { color: '#c9d1d9' } },
      tooltip: { mode: 'index', intersect: false },
    },
    scales: {
      x: { ticks: { color: '#8b949e', maxTicksLimit: 8 }, grid: { color: '#21262d' } },
      y: { ticks: { color: '#8b949e' }, grid: { color: '#21262d' }, title: { display: true, text: yLabel, color: '#8b949e' } },
    },
  };
}

function renderBestSynced() {
  el.bestSyncedBody.innerHTML = '';
  if (!Array.isArray(state.best) || state.best.length === 0) return;
  state.best.forEach((b) => {
    const tr = document.createElement('tr');
    tr.appendChild(td(String(b.rank)));
    tr.appendChild(td(shorten(b.vote_account), { mono: true, title: b.vote_account }));
    const meanTd = td(formatMsRaw(b.mean_drift_ms), { num: true });
    meanTd.classList.add(bestSyncedColor(b.mean_drift_ms));
    tr.appendChild(meanTd);
    tr.appendChild(td(formatMsRaw(b.stddev_drift_ms), { num: true }));
    tr.appendChild(td(formatInt(b.n_samples), { num: true }));
    tr.appendChild(td(formatNum(b.stake_xnt, 0), { num: true }));
    el.bestSyncedBody.appendChild(tr);
  });
}

function bestSyncedColor(ms) {
  const a = Math.abs(ms);
  if (a < 50) return 'best-good';
  if (a < 200) return 'best-ok';
  return 'best-neutral';
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
    if (key === 'mean_drift_ms_abs') {
      av = Math.abs(a.mean_drift_ms);
      bv = Math.abs(b.mean_drift_ms);
    } else if (key === 'rank') {
      return 0;
    } else {
      av = a[key];
      bv = b[key];
    }
    if (typeof av === 'string') return dir * av.localeCompare(bv);
    return dir * ((av || 0) - (bv || 0));
  });
}

function renderTable() {
  sortFiltered();
  const start = state.page * PAGE_SIZE;
  const slice = state.filtered.slice(start, start + PAGE_SIZE);
  el.rankingBody.innerHTML = '';
  slice.forEach((v, i) => {
    const tr = document.createElement('tr');
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
  const t = I18N[state.lang];
  el.pageInfo.textContent = `${t.page_info(state.page + 1, last + 1)} (${formatInt(state.filtered.length)})`;
  el.prev.disabled = state.page === 0;
  el.next.disabled = state.page >= last;
}

function td(text, opts) {
  opts = opts || {};
  const e = document.createElement('td');
  e.textContent = text;
  if (opts.num) e.classList.add('num');
  if (opts.title) e.title = opts.title;
  if (opts.mono) e.classList.add('mono');
  return e;
}
function driftTd(ms) {
  const e = td(formatMsRaw(ms), { num: true });
  if (ms > 100) e.classList.add('pos');
  else if (ms < -100) e.classList.add('neg');
  else e.classList.add('zero');
  return e;
}

function shorten(s) {
  if (!s || s.length <= 16) return s || '';
  return `${s.slice(0, 6)}…${s.slice(-6)}`;
}

function colorClass(absMs) {
  if (absMs < 200) return 'good';
  if (absMs < 1000) return 'warn';
  return 'bad';
}

// 100 µs / 1 ms / 10 ms / 100 ms thresholds for chrony offset
function offsetColorClass(absUs) {
  if (absUs < 100) return 'good';
  if (absUs < 1000) return 'good-yellow';
  if (absUs < 10000) return 'warn';
  if (absUs < 100000) return 'warn-strong';
  return 'bad';
}

function formatMs(ms) {
  if (ms === undefined || ms === null || Number.isNaN(ms)) return '—';
  const sign = ms >= 0 ? '+' : '−';
  const v = Math.abs(ms);
  if (v >= 1000) return `${sign}${(v / 1000).toFixed(2)}s`;
  return `${sign}${v.toFixed(0)} ms`;
}
function formatMsRaw(ms) {
  if (ms === undefined || ms === null || Number.isNaN(ms)) return '—';
  return Number(ms).toFixed(1);
}
function formatSignedMicros(us) {
  if (us === undefined || us === null || Number.isNaN(us)) return '—';
  const sign = us >= 0 ? '+' : '−';
  const v = Math.abs(us);
  return `${sign}${formatInt(v)} µs`;
}
function formatInt(n) {
  if (n === undefined || n === null) return '—';
  return Number(n).toLocaleString('en-US');
}
function formatNum(n, decimals) {
  if (n === undefined || n === null || Number.isNaN(n)) return '—';
  return Number(n).toLocaleString('en-US', { maximumFractionDigits: decimals, minimumFractionDigits: decimals });
}

function renderFooter() {
  if (!state.meta) {
    el.metaLine.textContent = '';
    return;
  }
  el.metaLine.textContent =
    `daemon ${state.meta.daemon_version}` +
    ` · ${formatInt(state.meta.total_slots_observed)} slots / ${formatInt(state.meta.total_votes_collected)} votes` +
    ` · last ${state.meta.generated_at_utc}`;
}

initLanguage();
applyI18n();
tickWallClock();
setInterval(tickWallClock, 100);
loadAll();
setInterval(loadAll, 60_000);
