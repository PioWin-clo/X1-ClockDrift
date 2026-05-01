"use strict";

const PAGE_SIZE = 50;

const I18N = {
  en: {
    tagline: 'Live drift of <code>Clock::unix_timestamp</code> on the X1 blockchain.',
    updated: 'updated',
    hide_foundation: 'Hide X1 Labs',
    capybara_qualifying_only: 'Capybara qualifying only (≥1000 XNT)',

    hero1_title: 'X1 network time right now',
    hero1_chain_time: 'X1 chain consensus',
    hero1_real_utc: 'Real UTC',
    hero1_drift_label: (drift) => {
      const a = Math.abs(drift);
      const dir = drift >= 0 ? 'ahead of' : 'behind';
      const sign = drift >= 0 ? '+' : '−';
      const v = a >= 1000 ? `${(a / 1000).toFixed(2)}s` : `${a.toFixed(0)} ms`;
      return `X1 chain is ${sign}${v} ${dir} real UTC`;
    },
    hero1_trend_label: (mean, std, isStable) => {
      const trend = isStable ? 'stable' : 'drifting';
      return `24h trend: ${trend}, mean ${mean.toFixed(0)} ms ± ${std.toFixed(0)} ms`;
    },

    hero2_title: 'Validator clock health',
    health_critical: 'Critical',
    health_critical_sub: '>5s',
    health_high: 'High',
    health_high_sub: '1-5s',
    health_healthy: 'Healthy',
    health_healthy_sub: '<1s',
    health_foundation: 'X1 Labs',
    health_foundation_sub: 'separate',
    capybara_note: (n, total, pct) =>
      `${formatInt(n)} of ${formatInt(total)} validators (${pct}%) will qualify for the Capybara delegation upgrade (≥1000 XNT self-stake).`,

    chart_history_title: 'Network drift over time (last 7 days, 5-minute buckets)',
    chart_history_median: 'X1 median drift',
    chart_history_stake: 'X1 stake-weighted',
    chart_history_sentinel: 'Sentinel offset (ms)',
    chart_axis_drift_ms: 'drift (ms)',
    chart_histogram_title: 'Validator drift distribution (all tracked)',
    foundation_trend_title: 'X1 Labs foundation drift trend (14 days)',
    foundation_trend_help: 'Tracks the 12-node X1 Labs foundation cluster drift over time. Sudden shifts (>100 ms in one bucket) indicate X1 Labs changed Tachyon configuration, NTP source, or deployed an update.',
    foundation_current_drift: 'Current avg drift',
    foundation_drift_change_7d: 'Change vs 7d ago',
    foundation_active_nodes: 'Active foundation nodes',
    foundation_alert_label: '⚠️ Recent change detected',
    foundation_trend_avg: 'avg drift',
    foundation_trend_min: 'min',
    foundation_trend_max: 'max',
    top_worst_subtitle: 'Validators with drift ≥500ms · ≥100 XNT stake · ≥20 samples',
    top_best_subtitle: '≥100 samples · ≥1000 XNT · |drift|<5s · foundation excluded',
    severity_critical: 'critical',
    severity_high: 'high',
    severity_medium: 'medium',

    worst_table_title: 'Top worst validators',
    worst_table_help: 'Sorted by absolute drift (worst first). Foundation nodes flagged but appear in their natural position.',
    ranking_search_placeholder: 'search by pubkey…',
    filter_all: 'All',
    filter_critical: 'Critical only',
    filter_high: 'High and worse',
    col_rank: '#',
    col_pubkey: 'pubkey',
    col_drift: 'drift (ms)',
    col_jitter: 'jitter (ms)',
    col_n: 'n',
    col_stake: 'stake (XNT)',
    col_severity: 'tag',
    col_label: 'label',

    best_synced_title: 'Best synchronized validators (top 10)',
    best_synced_help_v04: 'Validators with the smallest absolute drift. Min 1000 XNT stake (Capybara threshold), min 100 samples, foundation excluded.',

    foundation_table_title: '🏛️ X1 Labs Foundation',
    foundation_table_help: 'Official X1 Labs infrastructure. Shown separately because their drift is operational baseline, not validator misconfiguration.',

    deeper_analytics: 'Deeper analytics',
    analytics_hint: 'distribution, correlation, signature groups',

    signature_groups_title: 'Drift signature groups',
    signature_groups_help: 'Validators sharing identical drift values. May indicate shared infrastructure or coincidental NTP setup. Not necessarily "farms".',
    clusters_detected: 'Detected groups',
    cluster_validators: 'Validators in groups',
    largest_cluster: 'Largest group',
    cluster_tooltip: (size) => `Part of ${size}-validator drift signature group`,
    largest_cluster_value: (n, stake) => `${n} validators · ${stake} XNT`,

    scatter_title: 'Stake vs drift correlation',
    scatter_help: 'Each point is a validator. X-axis = stake (log scale). Y-axis = mean drift (ms). Cluster members share colour. Click a point for details.',
    scatter_correlation: 'Correlation',
    scatter_slope: 'Slope',
    scatter_slope_value: (slope) => `${slope.toFixed(0)} ms per 10× stake`,
    scatter_no_data: 'Insufficient data',

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
    modal_severity: 'Severity',
    modal_view_explorer: 'View on X1 Explorer',
    modal_limited_data: 'Detailed history is only kept for the top 500 most-impactful and top 10 best-synced validators.',
  },

  pl: {
    tagline: 'Bieżący dryf <code>Clock::unix_timestamp</code> na blockchainie X1.',
    updated: 'aktualizacja',
    hide_foundation: 'Ukryj X1 Labs',
    capybara_qualifying_only: 'Tylko kwalifikujące się do Capybara (≥1000 XNT)',

    hero1_title: 'Czas sieci X1 teraz',
    hero1_chain_time: 'Konsensus chain X1',
    hero1_real_utc: 'Realny UTC',
    hero1_drift_label: (drift) => {
      const a = Math.abs(drift);
      const dir = drift >= 0 ? 'przed' : 'za';
      const sign = drift >= 0 ? '+' : '−';
      const v = a >= 1000 ? `${(a / 1000).toFixed(2)}s` : `${a.toFixed(0)} ms`;
      return `Chain X1 jest ${sign}${v} ${dir} realnym UTC`;
    },
    hero1_trend_label: (mean, std, isStable) => {
      const trend = isStable ? 'stabilny' : 'dryfujący';
      return `Trend 24h: ${trend}, średnia ${mean.toFixed(0)} ms ± ${std.toFixed(0)} ms`;
    },

    hero2_title: 'Stan zegarów walidatorów',
    health_critical: 'Krytyczne',
    health_critical_sub: '>5s',
    health_high: 'Wysokie',
    health_high_sub: '1-5s',
    health_healthy: 'Zdrowe',
    health_healthy_sub: '<1s',
    health_foundation: 'X1 Labs',
    health_foundation_sub: 'osobno',
    capybara_note: (n, total, pct) =>
      `${formatInt(n)} z ${formatInt(total)} walidatorów (${pct}%) zakwalifikuje się do delegacji Capybara (≥1000 XNT self-stake).`,

    chart_history_title: 'Dryf sieci w czasie (ostatnie 7 dni, kubełki 5-minutowe)',
    chart_history_median: 'Mediana X1',
    chart_history_stake: 'X1 ważone stakiem',
    chart_history_sentinel: 'Odchylenie Sentinela (ms)',
    chart_axis_drift_ms: 'dryf (ms)',
    chart_histogram_title: 'Rozkład dryfu walidatorów (wszyscy śledzeni)',
    foundation_trend_title: 'Trend dryfu fundacji X1 Labs (14 dni)',
    foundation_trend_help: 'Śledzi dryf klastra 12 nodów fundacji w czasie. Nagłe skoki (>100 ms w jednym kubełku) oznaczają że X1 Labs zmieniło konfigurację Tachyona, źródło NTP, lub wdrożyło aktualizację.',
    foundation_current_drift: 'Aktualny średni dryf',
    foundation_drift_change_7d: 'Zmiana vs 7 dni temu',
    foundation_active_nodes: 'Aktywne nody fundacji',
    foundation_alert_label: '⚠️ Wykryto niedawną zmianę',
    foundation_trend_avg: 'średni dryf',
    foundation_trend_min: 'min',
    foundation_trend_max: 'max',
    top_worst_subtitle: 'Walidatorzy z dryfem ≥500ms · ≥100 XNT stake · ≥20 próbek',
    top_best_subtitle: '≥100 próbek · ≥1000 XNT · |dryf|<5s · bez fundacji',
    severity_critical: 'krytyczny',
    severity_high: 'wysoki',
    severity_medium: 'średni',

    worst_table_title: 'Najgorsze walidatory',
    worst_table_help: 'Sortowanie po bezwzględnej wartości dryfu (najgorsze najpierw). Walidatory fundacji oznaczone, ale widoczne w naturalnej kolejności.',
    ranking_search_placeholder: 'szukaj po pubkey…',
    filter_all: 'Wszystkie',
    filter_critical: 'Tylko krytyczne',
    filter_high: 'Wysokie i gorsze',
    col_rank: '#',
    col_pubkey: 'pubkey',
    col_drift: 'dryf (ms)',
    col_jitter: 'jitter (ms)',
    col_n: 'n',
    col_stake: 'stake (XNT)',
    col_severity: 'tag',
    col_label: 'label',

    best_synced_title: 'Najlepiej zsynchronizowani walidatorzy (top 10)',
    best_synced_help_v04: 'Walidatory z najmniejszym bezwzględnym dryfem. Min 1000 XNT stake (próg Capybara), min 100 próbek, fundacja wykluczona.',

    foundation_table_title: '🏛️ X1 Labs Foundation',
    foundation_table_help: 'Oficjalna infrastruktura X1 Labs. Pokazana osobno bo ich dryf jest baseline operacyjnym, nie błędem konfiguracji walidatora.',

    deeper_analytics: 'Analityka szczegółowa',
    analytics_hint: 'rozkład, korelacja, grupy sygnatur',

    signature_groups_title: 'Grupy identycznego dryfu',
    signature_groups_help: 'Walidatory dzielące identyczne wartości dryfu. Może oznaczać wspólną infrastrukturę lub przypadkowy zbieg NTP. Niekoniecznie „farmy".',
    clusters_detected: 'Wykryte grupy',
    cluster_validators: 'Walidatorzy w grupach',
    largest_cluster: 'Największa grupa',
    cluster_tooltip: (size) => `Część grupy ${size} walidatorów o identycznej sygnaturze`,
    largest_cluster_value: (n, stake) => `${n} walidatorów · ${stake} XNT`,

    scatter_title: 'Korelacja stake vs dryf',
    scatter_help: 'Każdy punkt = walidator. Oś X = stake (skala log). Oś Y = średni dryf (ms). Walidatory w jednym klastrze mają ten sam kolor. Kliknij punkt po szczegóły.',
    scatter_correlation: 'Korelacja',
    scatter_slope: 'Nachylenie',
    scatter_slope_value: (slope) => `${slope.toFixed(0)} ms na 10× stake`,
    scatter_no_data: 'Za mało danych',

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
    modal_severity: 'Klasa',
    modal_view_explorer: 'Otwórz w X1 Explorer',
    modal_limited_data: 'Szczegółowa historia jest zapisywana tylko dla 500 walidatorów o największym wpływie i 10 najlepiej zsynchronizowanych.',
  },
};

const CLUSTER_COLORS = [
  '#f85149', '#a371f7', '#3fb950', '#d29922', '#58a6ff',
  '#ff7b72', '#bc8cff', '#56d364', '#e3b341', '#79c0ff',
];

const STABILITY_THRESHOLD_MS = 500;
const CAPYBARA_THRESHOLD_XNT = 1000;

const state = {
  lang: 'en',
  hideFoundation: false,
  capybaraOnly: false,
  severityFilter: 'all',
  summary: null,
  validators: [],
  history: [],
  meta: null,
  best: [],
  worst: [],                  // v0.5.0: server-side filtered worst ranking
  foundation: [],
  foundationTrend: [],        // v0.5.0: 14-day foundation drift trend
  chrony: null,
  filtered: [],
  page: 0,
  sortKey: 'mean_drift_ms_abs',
  sortDir: -1,
  query: '',
};

const el = {};
function bindElements() {
  el.updatedTs = document.getElementById('updated-ts');
  el.hero1ChainTime = document.getElementById('hero1-chain-time');
  el.hero1RealUtc = document.getElementById('hero1-real-utc');
  el.hero1Drift = document.getElementById('hero1-drift');
  el.hero1Trend = document.getElementById('hero1-trend');
  el.nCritical = document.getElementById('n-critical');
  el.nHigh = document.getElementById('n-high');
  el.nHealthy = document.getElementById('n-healthy');
  el.nFoundation = document.getElementById('n-foundation');
  el.capybaraNote = document.getElementById('capybara-note');
  el.worstBody = document.getElementById('worst-body');
  el.bestSyncedBody = document.getElementById('best-synced-body');
  el.foundationBody = document.getElementById('foundation-body');
  el.sourcesBody = document.getElementById('sources-body');
  el.pageInfo = document.getElementById('page-info');
  el.prev = document.getElementById('prev');
  el.next = document.getElementById('next');
  el.search = document.getElementById('search');
  el.severityFilter = document.getElementById('severity-filter');
  el.metaLine = document.getElementById('meta-line');
  el.clockWall = document.getElementById('clock-wall-time');
  el.clockOffset = document.getElementById('clock-offset-value');
  el.clockRms = document.getElementById('clock-rms-value');
  el.clockStratum = document.getElementById('clock-stratum-value');
  el.clockReference = document.getElementById('clock-reference-value');
  el.clockSkew = document.getElementById('clock-skew-value');
  el.btnLangEn = document.getElementById('btn-lang-en');
  el.btnLangPl = document.getElementById('btn-lang-pl');
  el.hideFoundation = document.getElementById('hide-foundation');
  el.capybaraOnly = document.getElementById('capybara-only');
  el.nClusters = document.getElementById('n-clusters');
  el.nClustered = document.getElementById('n-clustered');
  el.nClusteredPct = document.getElementById('n-clustered-pct');
  el.largestCluster = document.getElementById('largest-cluster');
  el.scatterR = document.getElementById('scatter-r');
  el.scatterSlope = document.getElementById('scatter-slope');
  el.modal = document.getElementById('validator-modal');
  el.modalPubkey = document.getElementById('modal-pubkey');
  el.modalStats = document.getElementById('modal-stats');
  el.modalEmpty = document.getElementById('modal-empty');
  el.modalClose = document.getElementById('modal-close');
  el.modalLink = document.getElementById('modal-explorer-link');
}

function initLanguage() {
  let lang = localStorage.getItem('lang');
  if (!lang) {
    const browserLang = (navigator.language || 'en').toLowerCase();
    lang = browserLang.startsWith('pl') ? 'pl' : 'en';
    localStorage.setItem('lang', lang);
  }
  state.lang = lang === 'pl' ? 'pl' : 'en';
}

function initFilters() {
  state.hideFoundation = localStorage.getItem('hideFoundation') === '1';
  state.capybaraOnly = localStorage.getItem('capybaraOnly') === '1';
  el.hideFoundation.checked = state.hideFoundation;
  el.capybaraOnly.checked = state.capybaraOnly;
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
  document.querySelectorAll('#severity-filter option[data-i18n]').forEach((opt) => {
    const v = t[opt.dataset.i18n];
    if (typeof v === 'string') opt.textContent = v;
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

async function loadAll() {
  try {
    const [summary, validators, history, meta, best, worst, foundation, foundationTrend, chrony] =
      await Promise.all([
        fetchJSON('data/summary.json'),
        fetchJSON('data/validators.json'),
        fetchJSON('data/history.json'),
        fetchJSON('data/meta.json'),
        fetchJSONOptional('data/best_validators.json'),
        fetchJSONOptional('data/worst_validators.json'),
        fetchJSONOptional('data/foundation.json'),
        fetchJSONOptional('data/foundation_drift_trend.json'),
        fetchJSONOptional('data/chrony.json'),
      ]);
    state.summary = summary;
    state.validators = validators || [];
    state.history = history || [];
    state.meta = meta;
    state.best = best || [];
    state.worst = worst || [];
    state.foundation = foundation || [];
    state.foundationTrend = foundationTrend || [];
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
  return state.validators.filter((v) => {
    if (state.hideFoundation && v.is_foundation) return false;
    if (state.capybaraOnly && !v.qualifies_capybara) return false;
    return true;
  });
}

function renderAll() {
  renderHeader();
  renderHero1();
  renderHero2();
  renderClock();
  renderHistoryChart();
  renderFoundationTrend();    // v0.5.0
  applyWorstFilter();
  renderWorstTable();
  renderBestSynced();
  renderFoundation();
  renderHistogram();
  renderScatter();
  renderClusters();
  renderSources();
  renderFooter();
}

function renderHeader() {
  if (state.summary && state.summary.generated_at_utc) {
    el.updatedTs.textContent = state.summary.generated_at_utc;
  }
}

let hero1TickHandle = null;
function renderHero1() {
  // Hero #1 ticks live: wall clock + chain time (= wall clock + drift_ms_now).
  // No separate setInterval — we already tick wall clock at 100ms.
  tickHero1();
  if (!state.summary) return;
  const t = I18N[state.lang];
  const drift = state.summary.drift_ms_now ?? 0;
  el.hero1Drift.textContent = t.hero1_drift_label(drift);
  el.hero1Drift.className = 'hero1-drift ' + heroDriftColorClass(Math.abs(drift));
  const mean = state.summary.drift_24h_mean_ms ?? 0;
  const std = state.summary.drift_24h_stddev_ms ?? 0;
  const isStable = std < STABILITY_THRESHOLD_MS;
  el.hero1Trend.textContent = t.hero1_trend_label(mean, std, isStable);
}

function tickHero1() {
  const now = new Date();
  const driftMs = state.summary?.drift_ms_now ?? 0;
  const chainNow = new Date(now.getTime() + driftMs);
  el.hero1RealUtc.textContent = formatIsoMillis(now);
  el.hero1ChainTime.textContent = formatIsoMillis(chainNow);
}

function heroDriftColorClass(absMs) {
  if (absMs < 200) return 'good';
  if (absMs < 1000) return 'warn';
  return 'bad';
}

function renderHero2() {
  // Hero #2 numbers come from the live filtered population so the
  // toggles in the header (hide-foundation, capybara-only) actually
  // affect the prominent counts. Fall back to summary.json values
  // when no filtering is active, for consistency with snapshot.
  const visible = visibleValidators();
  const counts = {
    critical: 0, high: 0, healthy: 0, foundation: 0, capybara: 0,
  };
  for (const v of visible) {
    if (v.is_foundation) counts.foundation++;
    else if (v.severity === 'critical') counts.critical++;
    else if (v.severity === 'high') counts.high++;
    else if (v.severity === 'healthy') counts.healthy++;
    if (v.qualifies_capybara) counts.capybara++;
  }
  el.nCritical.textContent = formatInt(counts.critical);
  el.nHigh.textContent = formatInt(counts.high);
  el.nHealthy.textContent = formatInt(counts.healthy);
  el.nFoundation.textContent = formatInt(counts.foundation);

  const t = I18N[state.lang];
  const total = visible.length;
  const pct = total > 0 ? ((100 * counts.capybara) / total).toFixed(1) : '0.0';
  el.capybaraNote.textContent = t.capybara_note(counts.capybara, total, pct);
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
  tickHero1();
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
  // v0.5.1: Sentinel offset rendered on the SAME ms scale as X1 drift
  // (was µs on a separate right axis, which visually exaggerated ~25 µs
  // RMS oscillations to compete with -1600..+400 ms X1 drift). The whole
  // point of this dashboard is "Sentinel stable, X1 chaotic" — Sentinel
  // should look like a flat line near 0 against the X1 series, not its
  // own dedicated waveform.
  const sentinel = data.map((d) =>
    d.sentinel_offset_us != null ? d.sentinel_offset_us / 1000 : null,
  );
  const sentinelLabel = t.chart_history_sentinel;
  const datasets = [
    { label: t.chart_history_median, data: median, borderColor: '#58a6ff', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2, yAxisID: 'yLeft' },
    { label: t.chart_history_stake, data: stakeW, borderColor: '#3fb950', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2, yAxisID: 'yLeft' },
    { label: sentinelLabel, data: sentinel, borderColor: '#d29922', backgroundColor: 'transparent', pointRadius: 0, borderWidth: 1.5, tension: 0.2, yAxisID: 'yLeft', spanGaps: true },
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
        tooltip: {
          mode: 'index',
          intersect: false,
          callbacks: {
            label: (item) => {
              const v = item.parsed.y;
              if (v == null) return null;
              // Sentinel offset is ~µs-scale even after the ÷1000;
              // 3 decimals lets it stay legible alongside X1 ms values.
              const decimals = item.dataset.label === sentinelLabel ? 3 : 1;
              return `${item.dataset.label}: ${v.toFixed(decimals)} ms`;
            },
          },
        },
      },
      scales: {
        x: { ticks: { color: '#8b949e', maxTicksLimit: 8 }, grid: { color: '#21262d' } },
        yLeft: { type: 'linear', position: 'left', ticks: { color: '#8b949e' }, grid: { color: '#21262d' }, title: { display: true, text: t.chart_axis_drift_ms, color: '#8b949e' } },
      },
    },
  });
}

/// v0.5.0: 14-day foundation drift trend chart. Three datasets:
///   * avg drift (solid blue line, primary metric)
///   * min drift (light green, lower envelope)
///   * max drift (light red, upper envelope, fills to min for shaded band)
/// Plus stat-cards: current, 7d-ago delta, active node count, alert if
/// any 1h bucket jumped >100ms vs the previous bucket.
const FOUNDATION_JUMP_THRESHOLD_MS = 100;
let chartFoundationTrend = null;

function renderFoundationTrend() {
  const sectionEl = document.getElementById('foundation-trend-section');
  const data = state.foundationTrend || [];
  if (data.length === 0) {
    if (sectionEl) sectionEl.style.display = 'none';
    return;
  }
  if (sectionEl) sectionEl.style.display = '';

  const t = I18N[state.lang];
  const last = data[data.length - 1];

  document.getElementById('foundation-current-drift').textContent =
    formatMsRaw(last.avg_drift_ms) + ' ms';
  document.getElementById('foundation-active-nodes').textContent =
    `${last.nodes_active} / 12`;

  // 7d-ago bucket: nearest entry whose bucket_ms is within ±1h of target.
  const sevenDaysAgoTarget = last.bucket_ms - 7 * 86400 * 1000;
  let sevenDaysAgo = null;
  let bestDelta = Number.POSITIVE_INFINITY;
  for (const b of data) {
    const delta = Math.abs(b.bucket_ms - sevenDaysAgoTarget);
    if (delta < bestDelta && delta <= 3600 * 1000) {
      bestDelta = delta;
      sevenDaysAgo = b;
    }
  }
  const changeEl = document.getElementById('foundation-drift-change-7d');
  if (sevenDaysAgo) {
    const change = last.avg_drift_ms - sevenDaysAgo.avg_drift_ms;
    const sign = change >= 0 ? '+' : '−';
    changeEl.textContent = `${sign}${Math.abs(change).toFixed(0)} ms`;
    changeEl.classList.toggle('stat-warning', Math.abs(change) > FOUNDATION_JUMP_THRESHOLD_MS);
  } else {
    changeEl.textContent = '—';
    changeEl.classList.remove('stat-warning');
  }

  // Alert: largest single-bucket jump > threshold.
  let alert = null;
  for (let i = 1; i < data.length; i++) {
    const jump = Math.abs(data[i].avg_drift_ms - data[i - 1].avg_drift_ms);
    if (jump > FOUNDATION_JUMP_THRESHOLD_MS) {
      if (!alert || jump > alert.jump) {
        alert = {
          bucket_ms: data[i].bucket_ms,
          jump,
          prev: data[i - 1].avg_drift_ms,
          curr: data[i].avg_drift_ms,
        };
      }
    }
  }
  const alertCard = document.getElementById('foundation-alert-card');
  const alertVal = document.getElementById('foundation-alert-value');
  if (alert) {
    alertCard.hidden = false;
    const ts = new Date(alert.bucket_ms).toISOString().slice(0, 16).replace('T', ' ');
    alertVal.textContent =
      `${ts}Z: ${alert.prev.toFixed(0)} → ${alert.curr.toFixed(0)} ms (Δ${alert.jump.toFixed(0)} ms)`;
  } else {
    alertCard.hidden = true;
  }

  const ctx = document.getElementById('chart-foundation-trend');
  if (!ctx || !window.Chart) return;
  if (chartFoundationTrend) chartFoundationTrend.destroy();
  chartFoundationTrend = new Chart(ctx, {
    type: 'line',
    data: {
      labels: data.map((d) => new Date(d.bucket_ms).toISOString().slice(0, 16).replace('T', ' ')),
      datasets: [
        {
          label: t.foundation_trend_max,
          data: data.map((d) => d.max_drift_ms),
          borderColor: 'rgba(248, 81, 73, 0.4)',
          backgroundColor: 'rgba(248, 81, 73, 0.05)',
          fill: '+1',
          tension: 0.2,
          pointRadius: 0,
          borderWidth: 1,
        },
        {
          label: t.foundation_trend_min,
          data: data.map((d) => d.min_drift_ms),
          borderColor: 'rgba(63, 185, 80, 0.4)',
          backgroundColor: 'transparent',
          fill: false,
          tension: 0.2,
          pointRadius: 0,
          borderWidth: 1,
        },
        {
          label: t.foundation_trend_avg,
          data: data.map((d) => d.avg_drift_ms),
          borderColor: '#58a6ff',
          backgroundColor: 'transparent',
          fill: false,
          tension: 0.2,
          pointRadius: 0,
          borderWidth: 2,
        },
      ],
    },
    options: chartCommonOpts({ yLabel: 'drift (ms)' }),
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
    x: v.stake_xnt, y: v.mean_drift_ms,
    pubkey: v.pubkey, cluster: v.cluster_id, raw: v,
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
        { label: 'validators', data: points, backgroundColor: colors, pointRadius: 4 },
        { type: 'line', label: 'trend', data: lineData, borderColor: '#f85149', borderWidth: 1.5, borderDash: [4, 4], pointRadius: 0, fill: false },
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
        x: { type: 'logarithmic', ticks: { color: '#8b949e' }, grid: { color: '#21262d' }, title: { display: true, text: 'Stake (XNT, log)', color: '#8b949e' } },
        y: { ticks: { color: '#8b949e' }, grid: { color: '#21262d' }, title: { display: true, text: 'Mean drift (ms)', color: '#8b949e' } },
      },
    },
  });
}

function computeRegression(points) {
  const n = points.length;
  if (n < 2) return { r: 0, slope: 0, intercept: 0 };
  const meanX = points.reduce((s, p) => s + Math.log10(p.x), 0) / n;
  const meanY = points.reduce((s, p) => s + p.y, 0) / n;
  let num = 0, denX = 0, denY = 0;
  for (const p of points) {
    const dx = Math.log10(p.x) - meanX;
    const dy = p.y - meanY;
    num += dx * dy; denX += dx * dx; denY += dy * dy;
  }
  const r = (denX > 0 && denY > 0) ? num / Math.sqrt(denX * denY) : 0;
  const slope = denX > 0 ? num / denX : 0;
  const intercept = meanY - slope * meanX;
  return { r, slope, intercept };
}

function renderClusters() {
  if (!state.summary) return;
  const t = I18N[state.lang];
  const nClusters = state.summary.n_signature_groups ?? state.summary.n_clusters_detected ?? 0;
  const nIn = state.summary.n_validators_in_groups ?? state.summary.n_validators_in_clusters ?? 0;
  const nSing = state.summary.n_singletons ?? 0;
  const total = nIn + nSing;
  el.nClusters.textContent = formatInt(nClusters);
  el.nClustered.textContent = formatInt(nIn);
  el.nClusteredPct.textContent = total > 0 ? ` (${((100 * nIn) / total).toFixed(1)}%)` : '';
  if (nClusters > 0) {
    const stakeXnt = state.summary.largest_cluster_total_stake_xnt || 0;
    el.largestCluster.textContent = t.largest_cluster_value(
      formatInt(state.summary.largest_cluster_size || 0),
      formatNum(stakeXnt, 0),
    );
  } else {
    el.largestCluster.textContent = '—';
  }
}

function renderBestSynced() {
  el.bestSyncedBody.innerHTML = '';
  if (!Array.isArray(state.best) || state.best.length === 0) return;
  state.best.forEach((b) => {
    const tr = document.createElement('tr');
    tr.appendChild(td(String(b.rank)));
    tr.appendChild(pubkeyCell(b.vote_account, { ...b, pubkey: b.vote_account }));
    const meanTd = td(formatMsRaw(b.mean_drift_ms), { num: true });
    meanTd.classList.add(bestSyncedColor(b.mean_drift_ms));
    tr.appendChild(meanTd);
    tr.appendChild(td(formatMsRaw(b.stddev_drift_ms), { num: true }));
    tr.appendChild(td(formatInt(b.n_samples), { num: true }));
    tr.appendChild(td(formatNum(b.stake_xnt, 0), { num: true }));
    el.bestSyncedBody.appendChild(tr);
  });
}

function renderFoundation() {
  el.foundationBody.innerHTML = '';
  if (!Array.isArray(state.foundation) || state.foundation.length === 0) return;
  state.foundation.forEach((f) => {
    const tr = document.createElement('tr');
    tr.classList.add('row-foundation');
    tr.appendChild(td(String(f.rank)));
    tr.appendChild(td(f.label || 'X1 Labs', { mono: false }));
    tr.appendChild(pubkeyCell(f.vote_account, {
      pubkey: f.vote_account,
      mean_drift_ms: f.mean_drift_ms,
      stake_xnt: f.stake_xnt,
      stake_lamports: f.stake_lamports,
      n_samples: f.n_samples,
      is_foundation: true,
      foundation_label: f.label,
    }));
    tr.appendChild(driftTd(f.mean_drift_ms));
    tr.appendChild(td(formatMsRaw(f.stddev_drift_ms), { num: true }));
    tr.appendChild(td(formatInt(f.n_samples), { num: true }));
    tr.appendChild(td(formatNum(f.stake_xnt, 0), { num: true }));
    el.foundationBody.appendChild(tr);
  });
}

function bestSyncedColor(ms) {
  const a = Math.abs(ms);
  if (a < 50) return 'best-good';
  if (a < 200) return 'best-ok';
  return 'best-neutral';
}

/// v0.5.0: source data is now `state.worst` (server-side filtered top
/// worst from `worst_validators.json`), not the full `state.validators`.
/// Frontend filters: search by pubkey, severity dropdown, hide-foundation
/// + capybara-only toggles.
function applyWorstFilter() {
  const q = state.query;
  const sevFilter = state.severityFilter;
  state.filtered = (state.worst || []).filter((v) => {
    if (state.hideFoundation && v.is_foundation) return false;
    // Capybara qualifying = stake >= 1000 XNT (10^12 lamports)
    if (state.capybaraOnly && (v.stake_lamports || 0) < 1_000_000_000_000) return false;
    const pubkey = v.vote_account || v.pubkey || '';
    if (q && !pubkey.toLowerCase().includes(q)) return false;
    if (sevFilter === 'critical' && v.severity !== 'critical') return false;
    if (sevFilter === 'high' && v.severity !== 'critical' && v.severity !== 'high') return false;
    return true;
  });
  sortFiltered();
}

function sortFiltered() {
  const key = state.sortKey;
  const dir = state.sortDir;
  state.filtered.sort((a, b) => {
    let av, bv;
    if (key === 'mean_drift_ms_abs') {
      av = Math.abs(a.mean_drift_ms); bv = Math.abs(b.mean_drift_ms);
    } else if (key === 'severity') {
      const order = { critical: 4, high: 3, foundation: 2, healthy: 1 };
      av = order[a.severity] || 0; bv = order[b.severity] || 0;
    } else if (key === 'pubkey') {
      // v0.5.0: worst entries use `vote_account`; validators use `pubkey`.
      av = a.vote_account || a.pubkey || ''; bv = b.vote_account || b.pubkey || '';
    } else {
      av = a[key]; bv = b[key];
    }
    if (typeof av === 'string') return dir * av.localeCompare(bv);
    return dir * ((av || 0) - (bv || 0));
  });
}

function renderWorstTable() {
  sortFiltered();
  const start = state.page * PAGE_SIZE;
  const slice = state.filtered.slice(start, start + PAGE_SIZE);
  el.worstBody.innerHTML = '';
  slice.forEach((v, i) => {
    const tr = document.createElement('tr');
    // v0.4.1: row tint follows severity only. Foundation status is shown
    // by the 🏛️ badge in the severity cell, not by background colour, so
    // an X1 Labs node with critical drift goes red — same as any other.
    tr.classList.add(`row-${v.severity || 'unknown'}`);
    // v0.5.0: source data is `worst_validators.json` which uses
    // `vote_account` as the pubkey field; modal/click handlers look
    // for `.pubkey`, so normalize here.
    const pubkey = v.vote_account || v.pubkey;
    const modalData = { ...v, pubkey };
    tr.appendChild(td(String(start + i + 1)));
    tr.appendChild(severityCell(v));
    tr.appendChild(pubkeyCell(pubkey, modalData));
    tr.appendChild(driftTd(v.mean_drift_ms));
    tr.appendChild(td(formatMsRaw(v.stddev_drift_ms), { num: true }));
    tr.appendChild(td(formatInt(v.n_samples), { num: true }));
    tr.appendChild(td(formatNum(v.stake_xnt, 0), { num: true }));
    el.worstBody.appendChild(tr);
  });
  const last = Math.max(0, Math.ceil(state.filtered.length / PAGE_SIZE) - 1);
  const t = I18N[state.lang];
  el.pageInfo.textContent = `${t.page_info(state.page + 1, last + 1)} (${formatInt(state.filtered.length)})`;
  el.prev.disabled = state.page === 0;
  el.next.disabled = state.page >= last;
}

function severityCell(v) {
  // v0.4.1: severity icon and foundation badge are independent. A
  // foundation node with critical drift now shows BOTH 🚨 and 🏛️ —
  // pre-fix, the foundation icon hid the severity.
  const e = document.createElement('td');
  e.classList.add('severity-cell');
  const sevIcon = document.createElement('span');
  sevIcon.classList.add('sev-icon');
  switch (v.severity) {
    case 'critical':
      sevIcon.textContent = '🚨';
      sevIcon.classList.add('sev-critical');
      sevIcon.title = 'Critical: drift > 5s';
      break;
    case 'high':
      sevIcon.textContent = '⚠️';
      sevIcon.classList.add('sev-high');
      sevIcon.title = 'High: drift 1–5s';
      break;
    case 'healthy':
      sevIcon.textContent = '✅';
      sevIcon.classList.add('sev-healthy');
      sevIcon.title = 'Healthy: drift < 1s';
      break;
    default:
      sevIcon.textContent = '·';
      sevIcon.classList.add('sev-unknown');
      sevIcon.title = 'Insufficient data';
      break;
  }
  e.appendChild(sevIcon);
  if (v.is_foundation) {
    const foundationBadge = document.createElement('span');
    foundationBadge.classList.add('foundation-badge');
    foundationBadge.textContent = '🏛️';
    foundationBadge.title = v.foundation_label || 'X1 Labs Foundation';
    e.appendChild(foundationBadge);
  }
  return e;
}

function pubkeyCell(pubkey, data) {
  const e = td(shorten(pubkey), { mono: true, title: pubkey });
  e.classList.add('pubkey-cell');
  e.addEventListener('click', () => openValidatorModal(data));
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
function formatIsoMillis(d) {
  const yyyy = d.getUTCFullYear();
  const mm = String(d.getUTCMonth() + 1).padStart(2, '0');
  const dd = String(d.getUTCDate()).padStart(2, '0');
  const hh = String(d.getUTCHours()).padStart(2, '0');
  const mi = String(d.getUTCMinutes()).padStart(2, '0');
  const ss = String(d.getUTCSeconds()).padStart(2, '0');
  const ms = String(d.getUTCMilliseconds()).padStart(3, '0');
  return `${yyyy}-${mm}-${dd} ${hh}:${mi}:${ss}.${ms} UTC`;
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
    ? `${data.cluster_size}× group${data.cluster_id ? ` #${data.cluster_id}` : ''}`
    : '—';
  let severityDisplay = '—';
  if (data.is_foundation) severityDisplay = `🏛️ ${data.foundation_label || 'X1 Labs'}`;
  else if (data.severity === 'critical') severityDisplay = '🚨 critical';
  else if (data.severity === 'high') severityDisplay = '⚠️ high';
  else if (data.severity === 'healthy') severityDisplay = '✅ healthy';

  el.modalStats.innerHTML = '';
  const stats = [
    [t.modal_severity, severityDisplay],
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
    if (!r.ok) { el.modalEmpty.hidden = false; return; }
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
        label: 'drift (ms)', data: drift,
        borderColor: '#58a6ff', backgroundColor: 'transparent',
        pointRadius: 0, borderWidth: 1.5, tension: 0.2,
      }],
    },
    options: chartCommonOpts({ yLabel: 'drift (ms)' }),
  });
}

function closeModal() {
  el.modal.hidden = true;
  if (modalChart) { modalChart.destroy(); modalChart = null; }
}

function wireEventHandlers() {
  el.btnLangEn.addEventListener('click', () => setLanguage('en'));
  el.btnLangPl.addEventListener('click', () => setLanguage('pl'));
  el.hideFoundation.addEventListener('change', () => {
    state.hideFoundation = el.hideFoundation.checked;
    localStorage.setItem('hideFoundation', state.hideFoundation ? '1' : '0');
    renderAll();
  });
  el.capybaraOnly.addEventListener('change', () => {
    state.capybaraOnly = el.capybaraOnly.checked;
    localStorage.setItem('capybaraOnly', state.capybaraOnly ? '1' : '0');
    renderAll();
  });
  el.severityFilter.addEventListener('change', () => {
    state.severityFilter = el.severityFilter.value;
    state.page = 0;
    applyWorstFilter();
    renderWorstTable();
  });
  document.querySelectorAll('table.worst-table th[data-sort]').forEach((th) => {
    th.addEventListener('click', () => {
      const key = th.dataset.sort;
      if (!key) return;
      if (state.sortKey === key) state.sortDir = -state.sortDir;
      else { state.sortKey = key; state.sortDir = key === 'pubkey' ? 1 : -1; }
      state.page = 0;
      renderWorstTable();
    });
  });
  el.prev.addEventListener('click', () => { if (state.page > 0) { state.page--; renderWorstTable(); } });
  el.next.addEventListener('click', () => {
    const last = Math.max(0, Math.ceil(state.filtered.length / PAGE_SIZE) - 1);
    if (state.page < last) { state.page++; renderWorstTable(); }
  });
  el.search.addEventListener('input', () => {
    state.query = el.search.value.trim().toLowerCase();
    state.page = 0;
    applyWorstFilter();
    renderWorstTable();
  });
  el.modalClose.addEventListener('click', closeModal);
  el.modal.addEventListener('click', (e) => { if (e.target === el.modal) closeModal(); });
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && !el.modal.hidden) closeModal(); });
}

bindElements();
initLanguage();
initFilters();
applyI18n();
wireEventHandlers();
tickWallClock();
setInterval(tickWallClock, 100);
loadAll();
setInterval(loadAll, 60_000);
