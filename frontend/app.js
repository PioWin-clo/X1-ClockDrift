"use strict";

const PAGE_SIZE = 50;

const I18N = {
  en: {
    tagline: 'Live drift of <code>Clock::unix_timestamp</code> on the X1 blockchain.',
    updated: 'updated',
    hide_farms: 'Hide farms',
    hero_label: 'Network drift right now',
    hero_sub: (n, sw) => `median across ${n} validators · stake-weighted ${sw} ms`,
    card_validators: 'Validators observed (24h)',
    card_samples: 'Samples (24h)',
    card_drift1s: 'Drift > 1s',
    card_stake_drift: 'Stake-weighted drift',
    chart_history_title: 'Network drift over time (last 7 days, 5-minute buckets)',
    chart_histogram_title: 'Validator drift distribution (mean drift, top 500)',
    chart_history_median: 'X1 median drift',
    chart_history_stake: 'X1 stake-weighted',
    chart_history_sentinel: 'Sentinel offset (µs)',
    chart_axis_drift_ms: 'X1 drift (ms)',
    chart_axis_offset_us: 'Sentinel offset (µs)',
    clusters_section_title: 'Operator clusters (identical drift signature)',
    clusters_help: 'Validators sharing rounded (mean, stddev, p10) drift values with at least 3 members are flagged as multi-node operators.',
    clusters_detected: 'Detected clusters',
    cluster_validators: 'Validators in clusters',
    largest_cluster: 'Largest cluster',
    cluster_tooltip: (size, stake) => `Part of ${size}-validator cluster · total stake ${stake} XNT`,
    largest_cluster_value: (n, stake) => `${n} validators · ${stake} XNT`,
    scatter_title: 'Stake vs drift correlation',
    scatter_help: 'Each point is a validator. X-axis = stake (log scale). Y-axis = mean drift (ms). Cluster members share colour. Click a point for details.',
    scatter_correlation: 'Correlation',
    scatter_slope: 'Slope',
    scatter_slope_value: (slope) => `${slope.toFixed(0)} ms per 10× stake`,
    scatter_no_data: 'Insufficient data',
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
    col_cluster: 'cluster',
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
    modal_mean: 'Mean drift',
    modal_stake: 'Stake',
    modal_cluster: 'Cluster',
    modal_samples: 'Samples',
    modal_view_explorer: 'View on X1 Explorer',
    modal_limited_data: 'Detailed history is only kept for the top 500 most-impactful and top 10 best-synced validators.',
    modal_close: 'Close',
    cluster_singleton: '—',
    cluster_value: (size) => `${size}×`,
  },
  pl: {
    tagline: 'Bieżący dryf <code>Clock::unix_timestamp</code> na blockchainie X1.',
    updated: 'aktualizacja',
    hide_farms: 'Ukryj farmy',
    hero_label: 'Dryf sieci teraz',
    hero_sub: (n, sw) => `mediana z ${n} walidatorów · ważone stakiem ${sw} ms`,
    card_validators: 'Obserwowani walidatorzy (24h)',
    card_samples: 'Próbki (24h)',
    card_drift1s: 'Dryf > 1s',
    card_stake_drift: 'Dryf ważony stakiem',
    chart_history_title: 'Dryf sieci w czasie (ostatnie 7 dni, kubełki 5-minutowe)',
    chart_histogram_title: 'Rozkład dryfu walidatorów (średni dryf, top 500)',
    chart_history_median: 'Mediana X1',
    chart_history_stake: 'X1 ważone stakiem',
    chart_history_sentinel: 'Odchylenie Sentinela (µs)',
    chart_axis_drift_ms: 'Dryf X1 (ms)',
    chart_axis_offset_us: 'Odchylenie Sentinela (µs)',
    clusters_section_title: 'Klastry operatorów (identyczna sygnatura dryfu)',
    clusters_help: 'Walidatorzy o tych samych zaokrąglonych (średnia, odch.std, p10) wartościach dryfu, w grupach co najmniej 3, są oznaczani jako operatorzy multi-node.',
    clusters_detected: 'Wykryte klastry',
    cluster_validators: 'Walidatorzy w klastrach',
    largest_cluster: 'Największy klaster',
    cluster_tooltip: (size, stake) => `Część ${size}-walidatorowego klastra · łączny stake ${stake} XNT`,
    largest_cluster_value: (n, stake) => `${n} walidatorów · ${stake} XNT`,
    scatter_title: 'Korelacja stake vs dryf',
    scatter_help: 'Każdy punkt = walidator. Oś X = stake (skala log). Oś Y = średni dryf (ms). Walidatorzy w jednym klastrze mają ten sam kolor. Kliknij punkt po szczegóły.',
    scatter_correlation: 'Korelacja',
    scatter_slope: 'Nachylenie',
    scatter_slope_value: (slope) => `${slope.toFixed(0)} ms na 10× stake`,
    scatter_no_data: 'Za mało danych',
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
    col_cluster: 'klaster',
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
    modal_mean: 'Średni dryf',
    modal_stake: 'Stake',
    modal_cluster: 'Klaster',
    modal_samples: 'Próbki',
    modal_view_explorer: 'Otwórz w X1 Explorer',
    modal_limited_data: 'Szczegółowa historia jest zapisywana tylko dla 500 walidatorów o największym wpływie i 10 najlepiej zsynchronizowanych.',
    modal_close: 'Zamknij',
    cluster_singleton: '—',
    cluster_value: (size) => `${size}×`,
  },
};

const CLUSTER_COLORS = [
  '#f85149', '#a371f7', '#3fb950', '#d29922', '#58a6ff',
  '#ff7b72', '#bc8cff', '#56d364', '#e3b341', '#79c0ff',
];

const state = {
  lang: 'en',
  hideFarms: true,
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
  hideFarms: document.getElementById('hide-farms'),
  farmInfo: document.getElementById('farm-info'),
  nClusters: document.getElementById('n-clusters'),
  nClustered: document.getElementById('n-clustered'),
  nClusteredPct: document.getElementById('n-clustered-pct'),
  largestCluster: document.getElementById('largest-cluster'),
  scatterR: document.getElementById('scatter-r'),
  scatterSlope: document.getElementById('scatter-slope'),
  modal: document.getElementById('validator-modal'),
  modalPubkey: document.getElementById('modal-pubkey'),
  modalStats: document.getElementById('modal-stats'),
  modalEmpty: document.getElementById('modal-empty'),
  modalClose: document.getElementById('modal-close'),
  modalLink: document.getElementById('modal-explorer-link'),
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

function initFilter() {
  const stored = localStorage.getItem('hideFarms');
  state.hideFarms = stored === null ? true : stored === '1';
  el.hideFarms.checked = state.hideFarms;
}

function applyI18n() {
  const lang = state.lang;
  const t = I18N[lang];
  document.documentElement.lang = lang;
  document.querySelectorAll('[data-i18n]').forEach((node) => {
    const key = node.dataset.i18n;
    const value = t[key];
    if (typeof value === 'string') node.textContent = value;
  });
  document.querySelectorAll('[data-i18n-html]').forEach((node) => {
    const key = node.dataset.i18nHtml;
    const value = t[key];
    if (typeof value === 'string') node.innerHTML = value;
  });
  document.querySelectorAll('[data-i18n-attr-placeholder]').forEach((node) => {
    const key = node.dataset.i18nAttrPlaceholder;
    const value = t[key];
    if (typeof value === 'string') node.placeholder = value;
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

el.hideFarms.addEventListener('change', () => {
  state.hideFarms = el.hideFarms.checked;
  localStorage.setItem('hideFarms', state.hideFarms ? '1' : '0');
  renderAll();
});

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
  if (state.page > 0) { state.page--; renderTable(); }
});
el.next.addEventListener('click', () => {
  const last = Math.max(0, Math.ceil(state.filtered.length / PAGE_SIZE) - 1);
  if (state.page < last) { state.page++; renderTable(); }
});

el.search.addEventListener('input', () => {
  state.query = el.search.value.trim().toLowerCase();
  state.page = 0;
  applyFilter();
  renderTable();
});

el.modalClose.addEventListener('click', closeModal);
el.modal.addEventListener('click', (e) => {
  if (e.target === el.modal) closeModal();
});
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && !el.modal.hidden) closeModal();
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

function visibleValidators() {
  return state.hideFarms
    ? state.validators.filter((v) => !v.is_multi_node)
    : state.validators.slice();
}

function visibleBest() {
  return state.hideFarms
    ? state.best.filter((v) => !v.is_multi_node)
    : state.best.slice();
}

function renderAll() {
  renderHeader();
  renderClock();
  renderHero();
  renderCards();
  renderClusters();
  renderHistoryChart();
  renderHistogram();
  renderScatter();
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
  const farmCount = state.validators.filter((v) => v.is_multi_node).length;
  el.farmInfo.textContent = `(${formatInt(farmCount)})`;
}

function renderClock() {
  const cells = [el.clockOffset, el.clockRms, el.clockStratum, el.clockReference, el.clockSkew];
  if (!state.chrony) { cells.forEach((c) => (c.textContent = '—')); return; }
  const t = state.chrony.tracking;
  if (!t || t.system_offset_us == null) { cells.forEach((c) => (c.textContent = '—')); return; }
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
    tr.appendChild(td(stateLabel || s.state || '—'));
    tr.appendChild(td(s.offset_us != null ? formatSignedMicros(s.offset_us) : '—', { num: true }));
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

/// Per-validator stats recomputed from the visible (filtered) validator set.
function visibleStats() {
  const visible = visibleValidators();
  if (visible.length === 0) {
    return { count: 0, samples: 0, median: 0, mean: 0, stakeWeighted: 0, drift1s: 0, drift5s: 0 };
  }
  const sortedMedians = visible.map((v) => v.median_drift_ms)
    .sort((a, b) => a - b);
  const median = sortedMedians[Math.floor(sortedMedians.length / 2)];
  const mean = visible.reduce((s, v) => s + v.mean_drift_ms, 0) / visible.length;
  let num = 0, den = 0;
  for (const v of visible) {
    const w = v.stake_lamports || 0;
    if (w <= 0) continue;
    num += v.mean_drift_ms * w;
    den += w;
  }
  const stakeWeighted = den > 0 ? num / den : mean;
  return {
    count: visible.length,
    samples: visible.reduce((s, v) => s + (v.n_samples || 0), 0),
    median,
    mean,
    stakeWeighted,
    drift1s: visible.filter((v) => Math.abs(v.mean_drift_ms) >= 1000).length,
    drift5s: visible.filter((v) => Math.abs(v.mean_drift_ms) >= 5000).length,
  };
}

function renderHero() {
  if (!state.summary) return;
  const s = visibleStats();
  el.heroValue.textContent = formatMs(s.median);
  el.heroValue.className = 'hero-value ' + colorClass(Math.abs(s.median));
  const t = I18N[state.lang];
  el.heroSub.textContent = t.hero_sub(formatInt(s.count), formatNum(s.stakeWeighted, 1));
}

function renderCards() {
  if (!state.summary) return;
  const s = visibleStats();
  el.cardValidators.textContent = formatInt(s.count);
  el.cardSamples.textContent = formatInt(s.samples);
  el.cardDrift1s.textContent = formatInt(s.drift1s);
  const pct = s.count > 0 ? (100 * s.drift1s) / s.count : 0;
  el.cardDrift1sPct.textContent = `${pct.toFixed(1)}% (>5s: ${formatInt(s.drift5s)})`;
  el.cardStakeDrift.textContent = formatMs(s.stakeWeighted);
}

function renderClusters() {
  if (!state.summary) return;
  const t = I18N[state.lang];
  const nClusters = state.summary.n_clusters_detected || 0;
  const nIn = state.summary.n_validators_in_clusters || 0;
  const nSing = state.summary.n_singletons || 0;
  const total = nIn + nSing;
  el.nClusters.textContent = formatInt(nClusters);
  el.nClustered.textContent = formatInt(nIn);
  el.nClusteredPct.textContent = total > 0 ? ` (${((100 * nIn) / total).toFixed(1)}%)` : '';
  if (nClusters > 0) {
    const stakeXnt = state.summary.largest_cluster_total_stake_xnt || 0;
    el.largestCluster.textContent = t.largest_cluster_value(
      formatInt(state.summary.largest_cluster_size || 0),
      formatNum(stakeXnt, 0)
    );
  } else {
    el.largestCluster.textContent = '—';
  }
}

let chartHistory = null;
function renderHistoryChart() {
  const ctx = document.getElementById('chart-history');
  if (!ctx || !window.Chart) return;
  const t = I18N[state.lang];
  const data = state.history || [];
  const labels = data.map((d) => d.bucket_iso);
  const median = data.map((d) => d.median_drift_ms);
  const stakeW = data.map((d) => d.stake_weighted_drift_ms);
  const sentinel = data.map((d) => d.sentinel_offset_us);
  const datasets = [
    { label: t.chart_history_median, data: median, borderColor: '#58a6ff', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2, yAxisID: 'yLeft' },
    { label: t.chart_history_stake, data: stakeW, borderColor: '#3fb950', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2, yAxisID: 'yLeft' },
    { label: t.chart_history_sentinel, data: sentinel, borderColor: '#d29922', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2, yAxisID: 'yRight', spanGaps: true },
  ];
  if (chartHistory) chartHistory.destroy();
  chartHistory = new Chart(ctx, {
    type: 'line',
    data: { labels, datasets },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { labels: { color: '#c9d1d9' } },
        tooltip: { mode: 'index', intersect: false },
      },
      scales: {
        x: { ticks: { color: '#8b949e', maxTicksLimit: 8 }, grid: { color: '#21262d' } },
        yLeft: {
          type: 'linear', position: 'left',
          ticks: { color: '#8b949e' }, grid: { color: '#21262d' },
          title: { display: true, text: t.chart_axis_drift_ms, color: '#8b949e' },
        },
        yRight: {
          type: 'linear', position: 'right',
          ticks: { color: '#d29922' }, grid: { drawOnChartArea: false },
          title: { display: true, text: t.chart_axis_offset_us, color: '#d29922' },
        },
      },
    },
  });
}

let chartHistogram = null;
function renderHistogram() {
  const ctx = document.getElementById('chart-histogram');
  if (!ctx || !window.Chart) return;
  const buckets = buildHistogram(visibleValidators().map((v) => v.mean_drift_ms));
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
      legend: { labels: { color: '#c9d1d9' } },
      tooltip: { mode: 'index', intersect: false },
    },
    scales: {
      x: { ticks: { color: '#8b949e', maxTicksLimit: 8 }, grid: { color: '#21262d' } },
      y: { ticks: { color: '#8b949e' }, grid: { color: '#21262d' }, title: { display: true, text: yLabel, color: '#8b949e' } },
    },
  };
}

let chartScatter = null;
function renderScatter() {
  const ctx = document.getElementById('chart-scatter');
  if (!ctx || !window.Chart) return;
  const t = I18N[state.lang];
  const visible = visibleValidators().filter((v) => v.stake_xnt > 0);
  if (visible.length < 5) {
    el.scatterR.textContent = t.scatter_no_data;
    el.scatterSlope.textContent = '—';
    if (chartScatter) { chartScatter.destroy(); chartScatter = null; }
    return;
  }
  const points = visible.map((v) => ({
    x: v.stake_xnt,
    y: v.mean_drift_ms,
    pubkey: v.pubkey,
    cluster: v.cluster_id,
    raw: v,
  }));
  const colors = points.map((p) => p.cluster
    ? CLUSTER_COLORS[(p.cluster - 1) % CLUSTER_COLORS.length]
    : 'rgba(150,150,150,0.55)');

  const reg = computeRegression(points);
  el.scatterR.textContent = `r = ${reg.r.toFixed(3)}`;
  el.scatterSlope.textContent = t.scatter_slope_value(reg.slope);

  const xs = points.map((p) => p.x);
  const xMin = Math.max(0.0001, Math.min(...xs));
  const xMax = Math.max(...xs);
  const lineData = [
    { x: xMin, y: reg.intercept + reg.slope * Math.log10(xMin) },
    { x: xMax, y: reg.intercept + reg.slope * Math.log10(xMax) },
  ];

  if (chartScatter) chartScatter.destroy();
  chartScatter = new Chart(ctx, {
    type: 'scatter',
    data: {
      datasets: [
        {
          label: 'validators',
          data: points,
          backgroundColor: colors,
          pointRadius: 4,
        },
        {
          type: 'line',
          label: 'trend',
          data: lineData,
          borderColor: '#f85149',
          borderWidth: 1.5,
          borderDash: [4, 4],
          pointRadius: 0,
          fill: false,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      onClick: (_evt, elems) => {
        if (!elems || elems.length === 0) return;
        const e = elems[0];
        if (e.datasetIndex !== 0) return;
        const p = points[e.index];
        if (p && p.pubkey) openValidatorModal(p.raw);
      },
      plugins: {
        legend: { display: false },
        tooltip: {
          callbacks: {
            label: (ctx) => {
              if (ctx.datasetIndex !== 0) return null;
              const p = ctx.raw;
              return `${p.pubkey}: ${p.y.toFixed(1)} ms @ ${formatNum(p.x, 0)} XNT`;
            },
          },
        },
      },
      scales: {
        x: {
          type: 'logarithmic',
          ticks: { color: '#8b949e' },
          grid: { color: '#21262d' },
          title: { display: true, text: 'Stake (XNT, log)', color: '#8b949e' },
        },
        y: {
          ticks: { color: '#8b949e' },
          grid: { color: '#21262d' },
          title: { display: true, text: 'Mean drift (ms)', color: '#8b949e' },
        },
      },
    },
  });
}

/// Linear regression in log10(stake) space.
/// Returns Pearson r and slope (drift change per 10× stake).
function computeRegression(points) {
  const n = points.length;
  if (n < 2) return { r: 0, slope: 0, intercept: 0 };
  const meanX = points.reduce((s, p) => s + Math.log10(p.x), 0) / n;
  const meanY = points.reduce((s, p) => s + p.y, 0) / n;
  let num = 0, denX = 0, denY = 0;
  for (const p of points) {
    const dx = Math.log10(p.x) - meanX;
    const dy = p.y - meanY;
    num += dx * dy;
    denX += dx * dx;
    denY += dy * dy;
  }
  const r = (denX > 0 && denY > 0) ? num / Math.sqrt(denX * denY) : 0;
  const slope = denX > 0 ? num / denX : 0;
  const intercept = meanY - slope * meanX;
  return { r, slope, intercept };
}

function renderBestSynced() {
  el.bestSyncedBody.innerHTML = '';
  const visible = visibleBest();
  visible.forEach((b) => {
    const tr = document.createElement('tr');
    tr.appendChild(td(String(b.rank)));
    tr.appendChild(pubkeyCell(b.vote_account, b));
    const meanTd = td(formatMsRaw(b.mean_drift_ms), { num: true });
    meanTd.classList.add(bestSyncedColor(b.mean_drift_ms));
    tr.appendChild(meanTd);
    tr.appendChild(td(formatMsRaw(b.stddev_drift_ms), { num: true }));
    tr.appendChild(td(formatInt(b.n_samples), { num: true }));
    tr.appendChild(td(formatNum(b.stake_xnt, 0), { num: true }));
    tr.appendChild(clusterCell(b));
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
  const base = visibleValidators();
  state.filtered = q ? base.filter((v) => v.pubkey.toLowerCase().includes(q)) : base.slice();
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
    tr.appendChild(pubkeyCell(v.pubkey, v));
    tr.appendChild(driftTd(v.mean_drift_ms));
    tr.appendChild(driftTd(v.median_drift_ms));
    tr.appendChild(td(formatMsRaw(v.stddev_drift_ms), { num: true }));
    tr.appendChild(driftTd(v.p10_drift_ms));
    tr.appendChild(driftTd(v.p90_drift_ms));
    tr.appendChild(td(formatInt(v.n_samples), { num: true }));
    tr.appendChild(td(formatNum(v.stake_xnt, 0), { num: true }));
    tr.appendChild(td(formatNum(v.weighted_impact_ms_xnt, 0), { num: true }));
    tr.appendChild(clusterCell(v));
    el.rankingBody.appendChild(tr);
  });
  const last = Math.max(0, Math.ceil(state.filtered.length / PAGE_SIZE) - 1);
  const t = I18N[state.lang];
  el.pageInfo.textContent = `${t.page_info(state.page + 1, last + 1)} (${formatInt(state.filtered.length)})`;
  el.prev.disabled = state.page === 0;
  el.next.disabled = state.page >= last;
}

function pubkeyCell(pubkey, data) {
  const e = td(shorten(pubkey), { mono: true, title: pubkey });
  e.classList.add('pubkey-cell');
  e.addEventListener('click', () => openValidatorModal(data));
  return e;
}

function clusterCell(v) {
  const t = I18N[state.lang];
  const isCluster = v.is_multi_node && v.cluster_size > 1;
  const e = document.createElement('td');
  e.className = 'cluster-cell';
  if (isCluster) {
    e.textContent = t.cluster_value(v.cluster_size);
    const stakeXnt = (v.stake_xnt || 0) * v.cluster_size;
    e.title = t.cluster_tooltip(v.cluster_size, formatNum(stakeXnt, 0));
    if (v.cluster_id) {
      e.style.color = CLUSTER_COLORS[(v.cluster_id - 1) % CLUSTER_COLORS.length];
    }
  } else {
    e.textContent = t.cluster_singleton;
    e.classList.add('cluster-cell-singleton');
  }
  return e;
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
  if (!state.meta) { el.metaLine.textContent = ''; return; }
  el.metaLine.textContent =
    `daemon ${state.meta.daemon_version}` +
    ` · ${formatInt(state.meta.total_slots_observed)} slots / ${formatInt(state.meta.total_votes_collected)} votes` +
    ` · last ${state.meta.generated_at_utc}`;
}

let modalChart = null;
async function openValidatorModal(data) {
  const t = I18N[state.lang];
  const pubkey = data.pubkey || data.vote_account;
  el.modalPubkey.textContent = pubkey;
  el.modalLink.href = `https://explorer.x1.xyz/address/${pubkey}`;

  const cluster = data.is_multi_node && data.cluster_size > 1
    ? t.cluster_value(data.cluster_size) + (data.cluster_id ? ` (#${data.cluster_id})` : '')
    : '—';
  el.modalStats.innerHTML = '';
  const stats = [
    [t.modal_mean, formatMs(data.mean_drift_ms)],
    [t.modal_stake, `${formatNum(data.stake_xnt, 0)} XNT`],
    [t.modal_cluster, cluster],
    [t.modal_samples, formatInt(data.n_samples)],
  ];
  for (const [k, v] of stats) {
    const div = document.createElement('div');
    div.className = 'modal-stat';
    const k_span = document.createElement('span'); k_span.textContent = `${k}: `;
    const v_span = document.createElement('span'); v_span.className = 'modal-stat-value mono'; v_span.textContent = v;
    div.appendChild(k_span); div.appendChild(v_span);
    el.modalStats.appendChild(div);
  }

  el.modal.hidden = false;
  el.modalEmpty.hidden = true;
  if (modalChart) { modalChart.destroy(); modalChart = null; }

  try {
    const r = await fetch(`data/validators/${pubkey}.json`, { cache: 'no-store' });
    if (!r.ok) {
      el.modalEmpty.hidden = false;
      return;
    }
    const history = await r.json();
    renderModalChart(history.buckets || []);
  } catch (e) {
    console.warn('modal history fetch failed', e);
    el.modalEmpty.hidden = false;
  }
}

function renderModalChart(buckets) {
  const ctx = document.getElementById('modal-chart');
  if (!ctx || !window.Chart) return;
  if (buckets.length === 0) { el.modalEmpty.hidden = false; return; }
  const labels = buckets.map((b) => {
    const d = new Date(b.ts * 1000);
    return d.toISOString().substring(0, 16).replace('T', ' ');
  });
  const drift = buckets.map((b) => b.drift_ms);
  if (modalChart) modalChart.destroy();
  modalChart = new Chart(ctx, {
    type: 'line',
    data: {
      labels,
      datasets: [{
        label: 'drift (ms)',
        data: drift,
        borderColor: '#58a6ff',
        backgroundColor: 'transparent',
        pointRadius: 0,
        borderWidth: 1.5,
        tension: 0.2,
      }],
    },
    options: chartCommonOpts({ yLabel: 'drift (ms)' }),
  });
}

function closeModal() {
  el.modal.hidden = true;
  if (modalChart) { modalChart.destroy(); modalChart = null; }
}

initLanguage();
initFilter();
applyI18n();
tickWallClock();
setInterval(tickWallClock, 100);
loadAll();
setInterval(loadAll, 60_000);
