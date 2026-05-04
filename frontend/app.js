"use strict";

const PAGE_SIZE = 50;

const I18N = {
  en: {
    tagline: 'Time integrity monitor for the X1 blockchain — Layer 1 (vote pipeline) + Layer 2 (clock drift).',
    updated: 'updated',
    hide_foundation: 'Hide X1 Labs',
    stake_filter_label: 'Min stake (XNT, optional)',
    stake_filter_hint: 'Client-side lens. Server keeps all stake levels.',

    // v1.0.0 Hero #1 — vote pipeline latency
    hero1_title: 'X1 vote pipeline latency vs UTC',
    hero1_subtitle: 'Time from vote creation to on-chain visibility',
    hero1_explanation: 'Solana/Tachyon vote pipeline takes ~400-850 ms inherently (signing + gossip + block inclusion). Values in this range are normal — Layer 1 baseline, not clock drift. See methodology.',
    hero1_lag_label: (lagMs) => {
      const a = Math.abs(lagMs);
      const sign = lagMs >= 0 ? '+' : '−';
      const v = a >= 1000 ? `${(a / 1000).toFixed(2)} s` : `${a.toFixed(0)} ms`;
      return `${sign}${v}`;
    },
    hero1_trend_label: (mean, std, isStable) => {
      const trend = isStable ? 'stable' : 'fluctuating';
      return `24h trend: ${trend}, mean ${mean.toFixed(0)} ms ± ${std.toFixed(0)} ms`;
    },

    // v1.0.0 Hero #2 — pipeline health (drift bands rebucketed)
    hero2_title: 'Validator pipeline health',
    hero2_subtitle: 'Distribution by deviation from foundation baseline',
    health_normal_label: 'Normal pipeline',
    health_normal_desc: '|deviation| < 500 ms',
    health_slow_label: 'Slow pipeline',
    health_slow_desc: '500 ms – 5 s — investigate network/CPU',
    health_drift_label: 'Clock drift',
    health_drift_desc: '|drift| ≥ 5 s — Layer 2, real clock issue',
    health_foundation_label: 'X1 Labs',
    health_foundation_desc: 'baseline reference',

    // v1.0.0 Network chart — pipeline-latency framing
    chart_history_title: 'Vote pipeline latency over time (5-minute buckets)',
    chart_history_help: 'Aggregate vote-to-block latency across all voting validators. Stable ~-800 ms = healthy Tachyon protocol. Sudden swings = network stress, deployments, or load events.',
    // v1.2.0: dynamic subtitle — {bucketLabel} swapped for the active
    // aggregation interval ("5-min" / "10-min" / "15-min" / "30-min")
    // so users can tell which resolution they're looking at.
    chart_history_subtitle_template: 'Aggregate vote-to-block latency · {bucketLabel} buckets · stable ~-800 ms = healthy Tachyon protocol; sudden swings = stress, deployments, load events.',
    chart_history_median: 'X1 median lag',
    chart_history_stake: 'X1 stake-weighted lag',
    chart_history_sentinel: 'Sentinel offset (ms)',
    chart_axis_drift_ms: 'pipeline lag (ms)',
    chart_axis_lag_ms: 'pipeline lag (ms)',
    network_window_2d: '2d',
    network_window_4d: '4d',
    network_window_6d: '6d',
    network_window_12d: '12d',
    network_show_outliers: 'Show outliers',
    network_outlier_alert_title: '⚠️ Chain time anomalies detected:',
    chart_y_clamped_note: 'Y-axis: ±5000 ms · ▲ marker = value exceeds range',
    chart_histogram_title: 'Validator pipeline lag distribution (all tracked)',

    // v1.0.0 Foundation trend — pipeline framing
    foundation_trend_title: 'X1 Labs foundation pipeline trend (14 days)',
    foundation_trend_help: 'Tracks pipeline latency for the 12-node X1 Labs foundation cluster. Shifts >100 ms in one bucket indicate X1 Labs changed something operationally — Tachyon config, NTP source, deployment, or load test. NOT clock drift (foundation clocks are tightly synchronized — identical signature across nodes proves this).',
    foundation_current_drift: 'Current avg lag',
    foundation_current_lag: 'Current avg lag',
    foundation_drift_change_7d: 'Change vs 7d ago',
    foundation_lag_change_7d: 'Change vs 7d ago',
    foundation_active_nodes: 'Active foundation nodes',
    foundation_alert_label: '⚠️ Operational change detected',
    foundation_trend_avg: 'avg lag',
    foundation_trend_min: 'min',
    foundation_trend_max: 'max',

    // v1.0.0 Best — pipeline efficiency
    best_top_title: 'Top pipeline efficient validators (top 10)',
    best_top_subtitle: '≥100 samples · |lag|<5 s · foundation excluded · sorted by ABS(lag) ascending',
    best_top_help: 'Lowest pipeline latency means fastest vote→block round-trip. NOT necessarily the best clock — pipeline depends on network position, leader proximity, and Tachyon configuration. Foundation excluded since its baseline is the protocol baseline.',
    best_synced_title: 'Top pipeline efficient validators (top 10)',
    best_synced_help_v04: 'Lowest pipeline latency means fastest vote→block round-trip. NOT necessarily the best clock — pipeline depends on network position, leader proximity, and Tachyon configuration. Foundation excluded since its baseline is the protocol baseline.',

    // v1.0.0 Anomalies — split into two tiers
    worst_section_title: 'Anomalies & deviations',
    worst_tier1_title: 'Pipeline anomalies',
    worst_tier1_subtitle: '500 ms ≤ |lag| < 5 s · ≥20 samples · slow but not clock drift',
    worst_tier1_help: 'Validators with elevated pipeline latency. Causes: slow network, CPU saturation, geographic distance from leaders, suboptimal Tachyon config. Not a chain-time threat — but operator should investigate infra.',
    worst_tier2_title: 'Clock drift (Layer 2)',
    worst_tier2_subtitle: '|drift| ≥ 5 s · ≥20 samples · genuine clock misconfiguration',
    worst_tier2_help: 'Validators whose Clock::unix_timestamp deviates from real UTC by 5+ seconds. This IS clock drift — operator needs to fix chrony/NTP. Strontium oracle corrects this at protocol level for chain consumers.',
    worst_table_title: 'Pipeline anomalies',
    worst_table_help: 'Sorted by absolute lag (worst first). Foundation nodes flagged but appear in their natural position.',
    ranking_search_placeholder: 'search by pubkey…',
    filter_all: 'All',
    filter_critical: 'Critical only',
    filter_high: 'High and worse',
    col_rank: '#',
    col_pubkey: 'pubkey',
    col_drift: 'lag (ms)',
    col_jitter: 'jitter (ms)',
    col_n: 'n',
    col_stake: 'stake (XNT)',
    col_severity: 'tag',
    col_label: 'label',

    // v1.0.0 Severity badges (Layer 1/2 framework)
    severity_layer2: 'Layer 2 drift',
    severity_pipeline_slow: 'slow pipeline',
    severity_normal: 'normal',
    // legacy badge labels — kept for any place still binding to them
    severity_critical: 'critical',
    severity_high: 'high',
    severity_medium: 'medium',

    // v1.0.0 Layer 1/2 explainer block at page bottom
    layer_explainer_title: 'How to read this dashboard',
    layer1_title: 'Layer 1 — Pipeline latency',
    layer2_title: 'Layer 2 — Clock drift',
    layer1_explanation: 'Vote pipeline latency: ~400-850 ms inherent in Tachyon protocol. Sum of signing + gossip + block inclusion times. Identical across well-synchronized validators.',
    layer2_explanation: 'Validator system time vs real UTC. |drift| > 5 s indicates NTP/chrony misconfiguration. This is what Strontium oracle corrects.',
    methodology_link: 'methodology',
    methodology_cta: 'Full details:',

    // v1.1.0 — diagnostic snapshot widget (between Hero #2 and chart)
    diagnostic_snapshot_title: 'Network diagnostic snapshot',
    diagnostic_snapshot_subtitle: 'Side-by-side: pipeline health vs clock drift outliers',
    layer_1_label: 'Layer 1',
    layer_2_label: 'Layer 2',
    snapshot_pipeline_title: 'Vote pipeline latency',
    snapshot_pipeline_baseline: 'Baseline:',
    snapshot_pipeline_help: 'Network-wide median lag from vote creation to block inclusion. ~400-850 ms is healthy Tachyon protocol behavior.',
    snapshot_drift_title: 'Clock drift outliers',
    snapshot_drift_worst: 'Worst:',
    snapshot_drift_help: 'Validators with |drift| ≥ 5 seconds — genuine NTP/chrony misconfiguration. Full list in the Anomalies section below.',
    validators_label: 'validators',
    snapshot_status_healthy: 'Healthy — within baseline',
    snapshot_status_elevated: 'Elevated — investigate',
    snapshot_status_disrupted: 'Disrupted — check Foundation trend',
    snapshot_status_no_drift: 'No clock drift detected',
    snapshot_status_with_drift: 'validators with broken clocks',
    diagnostic_cta_text: 'Operating a validator? Find your row in the tables below, then check the ',
    diagnostic_cta_link: 'step-by-step diagnostic guide',
    diagnostic_cta_suffix: ' for fix instructions.',

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
    tagline: 'Monitor integralności czasu blockchaina X1 — Layer 1 (vote pipeline) + Layer 2 (dryf zegara).',
    updated: 'aktualizacja',
    hide_foundation: 'Ukryj X1 Labs',
    stake_filter_label: 'Min stake (XNT, opcjonalnie)',
    stake_filter_hint: 'Filtr po stronie klienta. Serwer zachowuje wszystkie poziomy stake.',

    // v1.0.0 Hero #1 — opóźnienie pipeline głosowania
    hero1_title: 'Opóźnienie pipeline głosowania X1 vs UTC',
    hero1_subtitle: 'Czas od utworzenia vote do widoczności w bloku',
    hero1_explanation: 'Pipeline głosowania Solana/Tachyon ma wbudowane opóźnienie ~400-850 ms (podpisywanie + gossip + włączenie do bloku). Wartości w tym zakresie są normalne — baseline Layer 1, nie dryf zegara. Patrz metodologia.',
    hero1_lag_label: (lagMs) => {
      const a = Math.abs(lagMs);
      const sign = lagMs >= 0 ? '+' : '−';
      const v = a >= 1000 ? `${(a / 1000).toFixed(2)} s` : `${a.toFixed(0)} ms`;
      return `${sign}${v}`;
    },
    hero1_trend_label: (mean, std, isStable) => {
      const trend = isStable ? 'stabilny' : 'fluktuujący';
      return `Trend 24h: ${trend}, średnia ${mean.toFixed(0)} ms ± ${std.toFixed(0)} ms`;
    },

    // v1.0.0 Hero #2 — stan pipeline (przebudowane progi dryfu)
    hero2_title: 'Stan pipeline walidatorów',
    hero2_subtitle: 'Rozkład odchyleń od baseline’u fundacji',
    health_normal_label: 'Pipeline normalny',
    health_normal_desc: '|odchylenie| < 500 ms',
    health_slow_label: 'Pipeline wolny',
    health_slow_desc: '500 ms – 5 s — sprawdź sieć/CPU',
    health_drift_label: 'Dryf zegara',
    health_drift_desc: '|dryf| ≥ 5 s — Layer 2, realny problem zegara',
    health_foundation_label: 'X1 Labs',
    health_foundation_desc: 'referencja',

    // v1.0.0 Wykres sieci — narracja pipeline
    chart_history_title: 'Opóźnienie pipeline w czasie (kubełki 5-minutowe)',
    chart_history_help: 'Zagregowane opóźnienie vote-do-bloku dla wszystkich głosujących walidatorów. Stabilne ~-800 ms = zdrowy protokół Tachyon. Nagłe wahania = stres sieci, deploymenty, lub load testy.',
    chart_history_subtitle_template: 'Zagregowane opóźnienie vote-do-bloku · kubełki {bucketLabel} · stabilne ~-800 ms = zdrowy protokół Tachyon; nagłe wahania = stres, deploymenty, load testy.',
    chart_history_median: 'Mediana opóźnienia X1',
    chart_history_stake: 'Opóźnienie ważone stakiem',
    chart_history_sentinel: 'Odchylenie Sentinela (ms)',
    chart_axis_drift_ms: 'opóźnienie pipeline (ms)',
    chart_axis_lag_ms: 'opóźnienie pipeline (ms)',
    network_window_2d: '2d',
    network_window_4d: '4d',
    network_window_6d: '6d',
    network_window_12d: '12d',
    network_show_outliers: 'Pokaż outlierów',
    network_outlier_alert_title: '⚠️ Wykryto anomalie chain time:',
    chart_y_clamped_note: 'Oś Y: ±5000 ms · ▲ marker = wartość przekracza zakres',
    chart_histogram_title: 'Rozkład opóźnienia pipeline walidatorów (wszyscy śledzeni)',

    // v1.0.0 Trend fundacji — narracja pipeline
    foundation_trend_title: 'Trend pipeline fundacji X1 Labs (14 dni)',
    foundation_trend_help: 'Śledzi opóźnienie pipeline klastra 12 nodów fundacji X1 Labs. Skoki >100 ms w jednym kubełku oznaczają że X1 Labs zmieniło coś operacyjnie — konfigurację Tachyona, źródło NTP, deployment, albo load test. NIE dryf zegara (zegary fundacji są ściśle zsynchronizowane — identyczna sygnatura na wszystkich nodach to potwierdza).',
    foundation_current_drift: 'Aktualne średnie opóźnienie',
    foundation_current_lag: 'Aktualne średnie opóźnienie',
    foundation_drift_change_7d: 'Zmiana vs 7 dni temu',
    foundation_lag_change_7d: 'Zmiana vs 7 dni temu',
    foundation_active_nodes: 'Aktywne nody fundacji',
    foundation_alert_label: '⚠️ Wykryto zmianę operacyjną',
    foundation_trend_avg: 'średnie opóźnienie',
    foundation_trend_min: 'min',
    foundation_trend_max: 'max',

    // v1.0.0 Best — efektywność pipeline
    best_top_title: 'Najefektywniejszy pipeline (top 10)',
    best_top_subtitle: '≥100 próbek · |opóźnienie|<5 s · bez fundacji · sortowane rosnąco po ABS(opóźnienie)',
    best_top_help: 'Najniższe opóźnienie pipeline oznacza najszybszą rundę vote→blok. NIE oznacza „najlepszego zegara" — pipeline zależy od pozycji w sieci, bliskości do leaderów, i konfiguracji Tachyona. Fundacja wykluczona bo jej baseline to baseline protokołu.',
    best_synced_title: 'Najefektywniejszy pipeline (top 10)',
    best_synced_help_v04: 'Najniższe opóźnienie pipeline oznacza najszybszą rundę vote→blok. NIE oznacza „najlepszego zegara" — pipeline zależy od pozycji w sieci, bliskości do leaderów, i konfiguracji Tachyona. Fundacja wykluczona bo jej baseline to baseline protokołu.',

    // v1.0.0 Anomalie — podział na dwa tier-y
    worst_section_title: 'Anomalie i odchylenia',
    worst_tier1_title: 'Anomalie pipeline',
    worst_tier1_subtitle: '500 ms ≤ |opóźnienie| < 5 s · ≥20 próbek · wolny pipeline, nie dryf zegara',
    worst_tier1_help: 'Walidatorzy z podwyższonym opóźnieniem pipeline. Przyczyny: wolna sieć, saturacja CPU, dystans geograficzny od leaderów, nieoptymalna konfiguracja Tachyona. Nie zagraża chain time — ale operator powinien sprawdzić infra.',
    worst_tier2_title: 'Dryf zegara (Layer 2)',
    worst_tier2_subtitle: '|dryf| ≥ 5 s · ≥20 próbek · realna błędna konfiguracja zegara',
    worst_tier2_help: 'Walidatorzy których Clock::unix_timestamp odbiega od prawdziwego UTC o 5+ sekund. TO JEST dryf zegara — operator musi naprawić chrony/NTP. Strontium oracle koryguje to na poziomie protokołu dla konsumentów chain.',
    worst_table_title: 'Anomalie pipeline',
    worst_table_help: 'Sortowanie po bezwzględnej wartości opóźnienia (najgorsze najpierw). Walidatory fundacji oznaczone, ale widoczne w naturalnej kolejności.',
    ranking_search_placeholder: 'szukaj po pubkey…',
    filter_all: 'Wszystkie',
    filter_critical: 'Tylko krytyczne',
    filter_high: 'Wysokie i gorsze',
    col_rank: '#',
    col_pubkey: 'pubkey',
    col_drift: 'opóźnienie (ms)',
    col_jitter: 'jitter (ms)',
    col_n: 'n',
    col_stake: 'stake (XNT)',
    col_severity: 'tag',
    col_label: 'label',

    // v1.0.0 Severity badges
    severity_layer2: 'Layer 2 dryf',
    severity_pipeline_slow: 'wolny pipeline',
    severity_normal: 'normalny',
    // legacy
    severity_critical: 'krytyczny',
    severity_high: 'wysoki',
    severity_medium: 'średni',

    // v1.0.0 Layer 1/2 wyjaśnienie na dole strony
    layer_explainer_title: 'Jak czytać ten dashboard',
    layer1_title: 'Layer 1 — Opóźnienie pipeline',
    layer2_title: 'Layer 2 — Dryf zegara',
    layer1_explanation: 'Opóźnienie vote pipeline: ~400-850 ms wbudowane w protokół Tachyon. Suma czasów podpisywania + gossip + włączenia do bloku. Identyczne dla dobrze zsynchronizowanych walidatorów.',
    layer2_explanation: 'Czas systemowy walidatora vs prawdziwy UTC. |dryf| > 5 s wskazuje na błędną konfigurację NTP/chrony. To jest co koryguje Strontium oracle.',
    methodology_link: 'metodologia',
    methodology_cta: 'Pełne szczegóły:',

    // v1.1.0 — diagnostyczny snapshot
    diagnostic_snapshot_title: 'Diagnostyczny snapshot sieci',
    diagnostic_snapshot_subtitle: 'Side-by-side: stan pipeline vs odchylenia zegarów',
    layer_1_label: 'Layer 1',
    layer_2_label: 'Layer 2',
    snapshot_pipeline_title: 'Opóźnienie pipeline głosowania',
    snapshot_pipeline_baseline: 'Baseline:',
    snapshot_pipeline_help: 'Mediana opóźnienia w sieci od utworzenia vote do włączenia do bloku. ~400-850 ms to zdrowe zachowanie protokołu Tachyon.',
    snapshot_drift_title: 'Odchylenia zegarów',
    snapshot_drift_worst: 'Najgorszy:',
    snapshot_drift_help: 'Walidatorzy z |dryf| ≥ 5 sekund — realna błędna konfiguracja NTP/chrony. Pełna lista w sekcji Anomalie poniżej.',
    validators_label: 'walidatorów',
    snapshot_status_healthy: 'Zdrowy — w zakresie baseline’u',
    snapshot_status_elevated: 'Podwyższony — sprawdź',
    snapshot_status_disrupted: 'Zaburzony — sprawdź Foundation trend',
    snapshot_status_no_drift: 'Brak dryfu zegarów',
    snapshot_status_with_drift: 'walidatorów z błędnymi zegarami',
    diagnostic_cta_text: 'Operujesz walidator? Znajdź swój wiersz w tabelach poniżej, a następnie zobacz ',
    diagnostic_cta_link: 'instrukcję diagnostyki krok po kroku',
    diagnostic_cta_suffix: ' z instrukcjami napraw.',

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

const state = {
  lang: 'en',
  hideFoundation: false,
  severityFilter: 'all',
  summary: null,
  validators: [],
  history: [],
  meta: null,
  best: [],
  worst: [],                  // v0.5.0: server-side filtered worst ranking
                              // (v1.0.0: mirrors pipelineAnomalies for legacy renderer)
  pipelineAnomalies: [],      // v1.0.0 Tier 1: 500ms ≤ |lag| < 5s
  clockDrift: [],             // v1.0.0 Tier 2: |drift| ≥ 5s — real Layer 2
  foundation: [],
  foundationTrend: [],        // v0.5.0: 14-day foundation drift trend
  chrony: null,
  filtered: [],
  page: 0,
  sortKey: 'mean_drift_ms_abs',
  sortDir: -1,
  query: '',
  // v0.6.0: optional client-side stake lens for best/worst tables.
  // 0 means "no filter" — server already returns all stake levels.
  bestMinStake: 0,
  worstMinStake: 0,
  // v0.7.0: network-drift chart window selector + outlier handling.
  // history.json holds 7 days; the user picks how many of those to show.
  // Outlier-clamp keeps the y-axis at ±5000 ms (or tighter via p1/p99
  // padding) when showOutliers is false, so a single 58s incident
  // doesn't squash the baseline line into a flat strip.
  networkDriftWindowDays: 6,
  showOutliers: false,
};
// v0.7.0: y-axis hard cap when outliers are clamped. Anything above this
// is rendered as a triangular marker on the chart edge + listed in the
// outlier-alert div so operators still see real chain-time anomalies.
const NETWORK_OUTLIER_THRESHOLD_MS = 5000;
const VALID_WINDOW_DAYS = [2, 4, 6, 12];

// v1.2.0: adaptive bucket aggregation — at 4d/6d/12d windows the raw
// 5-min buckets exceed the chart canvas pixel count (~1100 px), so
// consecutive lines become vertical "shading" instead of readable curves.
// We aggregate to a target of ~576 datapoints per chart by grouping
// raw 5-min buckets. 2d window stays raw (576 buckets fits cleanly).
//
// `sourceBuckets` = how many raw 5-min buckets get averaged into one
// chart point. `label` = human-readable bucket interval rendered into
// the chart subtitle so users can tell what aggregation level they're
// looking at.
const BUCKET_AGGREGATION = {
  2:  { sourceBuckets: 1, label: '5-min'  },  // 576 datapoints — raw
  4:  { sourceBuckets: 2, label: '10-min' },  // 576 datapoints
  6:  { sourceBuckets: 3, label: '15-min' },  // 576 datapoints
  12: { sourceBuckets: 6, label: '30-min' },  // 576 datapoints
};

// v1.2.0: collapse `groupSize` consecutive raw buckets into one. Each
// numeric field is averaged across the group, except `n_samples` which
// is summed (it's a count, not a rate). Nulls are skipped per field, so
// a group with one missing sentinel reading still yields a numeric
// sentinel offset for the rest of the group. Group anchor (bucket_ts /
// bucket_iso) is the FIRST raw bucket of the group — Chart.js places
// each group at that timestamp on the x-axis, which matches what
// "30-min bucket starting at 14:00" intuitively means.
function aggregateBuckets(rawBuckets, groupSize) {
  if (groupSize <= 1) return rawBuckets;
  const meanField = (group, field) => {
    const values = group
      .map((b) => b[field])
      .filter((v) => typeof v === 'number');
    return values.length > 0
      ? values.reduce((a, b) => a + b, 0) / values.length
      : null;
  };
  const sumField = (group, field) => {
    const values = group
      .map((b) => b[field])
      .filter((v) => typeof v === 'number');
    return values.length > 0 ? values.reduce((a, b) => a + b, 0) : null;
  };
  const out = [];
  for (let i = 0; i < rawBuckets.length; i += groupSize) {
    const group = rawBuckets.slice(i, i + groupSize);
    if (group.length === 0) continue;
    out.push({
      bucket_ts: group[0].bucket_ts,
      bucket_iso: group[0].bucket_iso,
      median_drift_ms: meanField(group, 'median_drift_ms'),
      mean_drift_ms: meanField(group, 'mean_drift_ms'),
      stake_weighted_drift_ms: meanField(group, 'stake_weighted_drift_ms'),
      sentinel_offset_us: meanField(group, 'sentinel_offset_us'),
      n_validators: meanField(group, 'n_validators'),
      n_samples: sumField(group, 'n_samples'),
    });
  }
  return out;
}

const el = {};
function bindElements() {
  el.updatedTs = document.getElementById('updated-ts');
  // v1.0.0: hero #1 simplified to a single pipeline-lag figure +
  // explanation. The legacy chain-time/real-utc dual readout was
  // removed because the hero was conflating "what time is X1's chain"
  // with "what's the pipeline latency" — those are different things.
  el.pipelineCurrentLag = document.getElementById('pipeline-current-lag');
  el.hero1Trend = document.getElementById('hero1-trend');
  // v1.0.0: hero #2 health bands. Drift-magnitude buckets, not legacy
  // critical/high/healthy severity strings.
  el.healthNormalCount = document.getElementById('health-normal-count');
  el.healthSlowCount = document.getElementById('health-slow-count');
  el.healthDriftCount = document.getElementById('health-drift-count');
  el.healthFoundationCount = document.getElementById('health-foundation-count');
  el.worstBody = document.getElementById('worst-body');
  el.clockDriftBody = document.getElementById('clock-drift-body');
  el.clockDriftEmpty = document.getElementById('clock-drift-empty');
  // v1.1.0: diagnostic snapshot widget (between Hero #2 and chart-block).
  el.snapshotPipelineCurrent = document.getElementById('snapshot-pipeline-current');
  el.snapshotPipelineStatus = document.getElementById('snapshot-pipeline-status');
  el.snapshotPipelineBaseline = document.getElementById('snapshot-pipeline-baseline');
  el.snapshotDriftCount = document.getElementById('snapshot-drift-count');
  el.snapshotDriftStatus = document.getElementById('snapshot-drift-status');
  el.snapshotDriftWorst = document.getElementById('snapshot-drift-worst');
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
  el.bestMinStake = document.getElementById('best-min-stake');
  el.worstMinStake = document.getElementById('worst-min-stake');
  // v0.7.0: network-drift chart controls
  el.windowButtons = Array.from(document.querySelectorAll('.window-btn'));
  el.showOutliers = document.getElementById('show-outliers');
  el.chartYClampedNote = document.getElementById('chart-y-clamped-note');
  el.networkOutlierAlert = document.getElementById('network-outlier-alert');
  el.networkOutlierAlertList = document.getElementById('network-outlier-alert-list');
  // v1.2.0: dynamic chart subtitle (bucket-aggregation interval)
  el.chartHistorySubtitle = document.getElementById('chart-history-subtitle');
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
  el.hideFoundation.checked = state.hideFoundation;
  // v0.6.0: optional client-side stake lens. Persist user choice across
  // reloads so the lens sticks. 0 / NaN / negative all mean "no filter".
  const savedBest = parseFloat(localStorage.getItem('bestMinStake') || '0');
  const savedWorst = parseFloat(localStorage.getItem('worstMinStake') || '0');
  state.bestMinStake = isFinite(savedBest) && savedBest > 0 ? savedBest : 0;
  state.worstMinStake = isFinite(savedWorst) && savedWorst > 0 ? savedWorst : 0;
  if (el.bestMinStake) {
    el.bestMinStake.value = state.bestMinStake > 0 ? String(state.bestMinStake) : '';
  }
  if (el.worstMinStake) {
    el.worstMinStake.value = state.worstMinStake > 0 ? String(state.worstMinStake) : '';
  }
  // v0.7.0: network-drift chart window + outlier toggle. Persist both,
  // default to 6d/clamped view (typically the most readable for a 7-day
  // history with the rare multi-second incident in it).
  const savedWindow = parseInt(localStorage.getItem('networkDriftWindowDays') || '', 10);
  state.networkDriftWindowDays = VALID_WINDOW_DAYS.includes(savedWindow) ? savedWindow : 6;
  state.showOutliers = localStorage.getItem('showOutliers') === '1';
  if (el.showOutliers) el.showOutliers.checked = state.showOutliers;
  syncWindowButtons();
}

// v0.7.0: visually mark the active window button (segmented control look).
function syncWindowButtons() {
  if (!Array.isArray(el.windowButtons)) return;
  el.windowButtons.forEach((btn) => {
    const days = parseInt(btn.dataset.windowDays || '', 10);
    btn.classList.toggle('active', days === state.networkDriftWindowDays);
  });
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
    // v1.0.0: pipeline_anomalies.json (Tier 1) + clock_drift.json (Tier 2)
    // are the canonical worst-tables. worst_validators.json is fetched as
    // a fallback for the first cycle after a daemon upgrade where the
    // new files aren't on the data branch yet — once they exist, Tier 1
    // takes over and the fallback path becomes a no-op.
    const [
      summary, validators, history, meta,
      best, pipelineAnomalies, clockDrift, worstLegacy,
      foundation, foundationTrend, chrony,
    ] = await Promise.all([
      fetchJSON('data/summary.json'),
      fetchJSON('data/validators.json'),
      fetchJSON('data/history.json'),
      fetchJSON('data/meta.json'),
      fetchJSONOptional('data/best_validators.json'),
      fetchJSONOptional('data/pipeline_anomalies.json'),
      fetchJSONOptional('data/clock_drift.json'),
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
    // Prefer the v1.0.0 split exports; fall back to the legacy combined
    // ranking only if the split files are missing (pre-upgrade snapshot).
    if (pipelineAnomalies != null || clockDrift != null) {
      state.pipelineAnomalies = pipelineAnomalies || [];
      state.clockDrift = clockDrift || [];
      // Mirror Tier 1 into state.worst so the legacy worst-table render
      // path keeps working without a parallel rewrite.
      state.worst = state.pipelineAnomalies;
    } else {
      const legacy = worstLegacy || [];
      // Split the legacy combined ranking into the two tiers client-side
      // so the new UI still renders correctly during the transition.
      state.pipelineAnomalies = legacy.filter((v) => {
        const a = Math.abs(v.mean_drift_ms || 0);
        return a >= 500 && a < 5000;
      });
      state.clockDrift = legacy.filter(
        (v) => Math.abs(v.mean_drift_ms || 0) >= 5000,
      );
      state.worst = state.pipelineAnomalies;
    }
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
    return true;
  });
}

function renderAll() {
  renderHeader();
  renderHero1();
  renderHero2();
  renderDiagnosticSnapshot();  // v1.1.0
  renderClock();
  renderHistoryChart();
  renderFoundationTrend();    // v0.5.0
  applyWorstFilter();
  renderWorstTable();        // v1.0.0 Tier 1: pipeline anomalies
  renderClockDriftTable();   // v1.0.0 Tier 2: Layer 2 clock drift
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

// v1.0.0: hero #1 — single pipeline-lag figure with explanation. The
// legacy chain-time / real-utc dual readout was misframing the signal
// as "X1 chain consensus vs UTC" when it's actually pipeline latency.
function renderHero1() {
  if (!state.summary) return;
  const t = I18N[state.lang];
  const lag = state.summary.drift_ms_now ?? 0;
  if (el.pipelineCurrentLag) {
    el.pipelineCurrentLag.textContent = t.hero1_lag_label(lag);
    el.pipelineCurrentLag.className =
      'mono ' + pipelineLagColorClass(Math.abs(lag));
  }
  if (el.hero1Trend) {
    const mean = state.summary.drift_24h_mean_ms ?? 0;
    const std = state.summary.drift_24h_stddev_ms ?? 0;
    const isStable = std < STABILITY_THRESHOLD_MS;
    el.hero1Trend.textContent = t.hero1_trend_label(mean, std, isStable);
  }
}

// v1.0.0: pipeline-lag colour bands match the Layer 1/2 framework.
//   < 500 ms  → normal pipeline (green)
//   < 5000 ms → slow pipeline   (warn)
//   ≥ 5000 ms → Layer 2 drift   (bad)
function pipelineLagColorClass(absMs) {
  if (absMs < 500) return 'good';
  if (absMs < 5000) return 'warn';
  return 'bad';
}

// v1.0.0: hero #2 health bands. Drift-magnitude buckets per the Layer
// 1/Layer 2 framework. Computed from the live validators[] array, not
// from a cached server-side severity field, so toggles like
// hide-foundation immediately reflect in the prominent counts.
function renderHero2() {
  const visible = visibleValidators();
  let normal = 0;
  let slow = 0;
  let drift = 0;
  let foundation = 0;
  for (const v of visible) {
    if (v.is_foundation) {
      foundation += 1;
      continue;
    }
    const a = Math.abs(v.mean_drift_ms || 0);
    if (a >= 5000) drift += 1;
    else if (a >= 500) slow += 1;
    else normal += 1;
  }
  if (el.healthNormalCount) el.healthNormalCount.textContent = formatInt(normal);
  if (el.healthSlowCount) el.healthSlowCount.textContent = formatInt(slow);
  if (el.healthDriftCount) el.healthDriftCount.textContent = formatInt(drift);
  if (el.healthFoundationCount) el.healthFoundationCount.textContent = formatInt(foundation);
}

// v1.0.0: severity classification for badges, replacing the legacy
// critical/high/healthy strings. Returns the i18n key for the badge
// label plus a CSS class and an icon. Foundation rows are tagged
// upstream by render code (the 🏛️ marker) and don't pass through here.
function severityFor(driftMs) {
  const abs = Math.abs(driftMs || 0);
  if (abs >= 5000) {
    return { cssClass: 'severity-layer2', labelKey: 'severity_layer2', icon: '🔴' };
  }
  if (abs >= 500) {
    return {
      cssClass: 'severity-pipeline-slow',
      labelKey: 'severity_pipeline_slow',
      icon: '⚠️',
    };
  }
  return { cssClass: 'severity-normal', labelKey: 'severity_normal', icon: '✅' };
}

// v1.1.0: diagnostic snapshot — at-a-glance side-by-side health for
// Layer 1 (current pipeline lag vs foundation baseline) and Layer 2
// (count of validators with |drift| ≥ 5 s, plus the worst entry).
//
// Design constraints:
//   * Pipeline current is the most recent network bucket from history.json
//     (5-minute resolution), not summary.json's drift_ms_now — the latter
//     is a single instantaneous reading; the bucket median absorbs jitter.
//   * Baseline is the foundation cluster's 7-day pipeline average from
//     foundation_drift_trend.json, with extreme spikes (|x| ≥ 5 s)
//     filtered out. Falls back to -800 ms (Theo's documented Layer 1
//     baseline) if foundation data is unavailable, so the widget still
//     renders something useful on a cold-start snapshot.
//   * Status thresholds (200 ms / 500 ms deviation from baseline) match
//     the methodology guide's Tier 1 boundary.
const PIPELINE_BASELINE_FALLBACK_MS = -800;
const PIPELINE_STATUS_HEALTHY_DEV_MS = 200;
const PIPELINE_STATUS_ELEVATED_DEV_MS = 500;
const FOUNDATION_BASELINE_LOOKBACK_BUCKETS = 168; // 7 d × 24 h × 1 bucket/h

function renderDiagnosticSnapshot() {
  const t = I18N[state.lang];

  // ---- LEFT card: Layer 1 pipeline lag ----
  // Latest history bucket gives the freshest aggregate signal. Empty
  // history means we just started up — leave the dashes alone.
  const history = Array.isArray(state.history) ? state.history : [];
  const latestBucket = history[history.length - 1];
  const pipelineCurrentMs = latestBucket
    ? latestBucket.median_drift_ms
    : null;

  // Foundation cluster as the protocol baseline. Slice the last week of
  // hourly buckets and average the avg_drift_ms field, dropping any
  // bucket where |x| ≥ 5 s (those would be Layer-2-style anomalies and
  // shouldn't define "normal").
  const foundationTrend = Array.isArray(state.foundationTrend)
    ? state.foundationTrend
    : [];
  const recentFoundation = foundationTrend.slice(-FOUNDATION_BASELINE_LOOKBACK_BUCKETS);
  const validBaseline = recentFoundation
    .map((b) => b.avg_drift_ms)
    .filter((v) => typeof v === 'number' && Math.abs(v) < 5000);
  const baselineMs = validBaseline.length > 0
    ? validBaseline.reduce((a, b) => a + b, 0) / validBaseline.length
    : PIPELINE_BASELINE_FALLBACK_MS;

  if (el.snapshotPipelineCurrent && el.snapshotPipelineStatus && el.snapshotPipelineBaseline) {
    el.snapshotPipelineBaseline.textContent = baselineMs.toFixed(0);

    if (typeof pipelineCurrentMs === 'number') {
      el.snapshotPipelineCurrent.textContent = pipelineCurrentMs.toFixed(0);
      el.snapshotPipelineCurrent.className = 'metric-value mono';
      const deviation = Math.abs(pipelineCurrentMs - baselineMs);
      let statusClass;
      let statusText;
      if (deviation < PIPELINE_STATUS_HEALTHY_DEV_MS) {
        statusClass = 'status-healthy';
        statusText = `✅ ${t.snapshot_status_healthy}`;
      } else if (deviation < PIPELINE_STATUS_ELEVATED_DEV_MS) {
        statusClass = 'status-elevated';
        statusText = `⚠️ ${t.snapshot_status_elevated}`;
      } else {
        statusClass = 'status-disrupted';
        statusText = `🔴 ${t.snapshot_status_disrupted}`;
      }
      el.snapshotPipelineCurrent.classList.add(statusClass);
      el.snapshotPipelineStatus.textContent = statusText;
    } else {
      el.snapshotPipelineCurrent.textContent = '—';
      el.snapshotPipelineCurrent.className = 'metric-value mono';
      el.snapshotPipelineStatus.textContent = '—';
    }
  }

  // ---- RIGHT card: Layer 2 clock-drift outliers ----
  // clock_drift.json is already filtered to |drift| ≥ 5 s and sorted by
  // ABS(drift) DESC, so the count and worst entry are direct reads.
  const driftRows = Array.isArray(state.clockDrift) ? state.clockDrift : [];
  if (el.snapshotDriftCount && el.snapshotDriftStatus && el.snapshotDriftWorst) {
    el.snapshotDriftCount.textContent = String(driftRows.length);
    el.snapshotDriftCount.className = 'metric-value mono';

    if (driftRows.length === 0) {
      el.snapshotDriftCount.classList.add('status-clean');
      el.snapshotDriftStatus.textContent = `✅ ${t.snapshot_status_no_drift}`;
      el.snapshotDriftWorst.textContent = '—';
    } else {
      el.snapshotDriftCount.classList.add('status-warning');
      el.snapshotDriftStatus.textContent =
        `🔴 ${driftRows.length} ${t.snapshot_status_with_drift}`;
      const worst = driftRows[0];
      const pubkey = worst.vote_account || worst.pubkey || '';
      const driftSec = ((worst.mean_drift_ms || 0) / 1000).toFixed(1);
      const sign = (worst.mean_drift_ms || 0) >= 0 ? '+' : '−';
      el.snapshotDriftWorst.textContent =
        `${sign}${Math.abs(parseFloat(driftSec)).toFixed(1)} s · ${shorten(pubkey)}`;
    }
  }
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
  // v1.0.0: hero #1 no longer has a chain-time / real-utc dual readout
  // (replaced by a single pipeline-lag figure that updates per refresh
  // cycle, not per wall-clock tick), so no per-tick hero #1 work here.
}

let chartHistory = null;

// v0.7.0: percentile over a numeric array. Returns null on empty input.
// Used to clamp the y-axis to a "typical-data" band (p1..p99 with 20%
// padding) so a single 58s incident doesn't squash the baseline.
function percentile(values, p) {
  if (!values.length) return null;
  const sorted = values.slice().sort((a, b) => a - b);
  const idx = Math.max(0, Math.min(sorted.length - 1, Math.floor((p / 100) * (sorted.length - 1))));
  return sorted[idx];
}

// v0.7.0: human-readable absolute-drift formatter for outlier list and
// the on-chart marker label. Keeps "ms" for sub-second values, switches
// to "s" with one decimal for >=1000 ms (the regime that motivated this).
function formatDriftMagnitude(ms) {
  const a = Math.abs(ms);
  if (a >= 1000) return `${(a / 1000).toFixed(1)} s`;
  return `${a.toFixed(0)} ms`;
}

function renderHistoryChart() {
  const ctx = document.getElementById('chart-history');
  if (!ctx || !window.Chart) return;
  const t = I18N[state.lang];

  // v0.7.0: filter by selected window. history.json holds 7 days; the
  // user picks how much of that to show. We compare against the most
  // recent bucket's timestamp rather than Date.now() so a stale export
  // still renders something useful (don't blank the chart on a paused
  // exporter).
  const all = Array.isArray(state.history) ? state.history : [];
  const days = state.networkDriftWindowDays || 6;
  let data = all;
  if (all.length > 0) {
    const latestTs = all[all.length - 1].bucket_ts || 0;
    const cutoffTs = latestTs - days * 86400;
    data = all.filter((d) => (d.bucket_ts || 0) >= cutoffTs);
  }

  // v1.2.0: client-side bucket aggregation. The wider the window, the
  // more raw 5-min buckets we'd render — at 12d that's 3456 datapoints
  // for a ~1100 px canvas. Group them into wider buckets so each
  // datapoint gets ~2 px of horizontal space. Outlier detection +
  // y-axis clamping all work on the aggregated series, which keeps the
  // outlier-alert list and the chart consistent with each other.
  const aggregation = BUCKET_AGGREGATION[days] || { sourceBuckets: 1, label: '5-min' };
  data = aggregateBuckets(data, aggregation.sourceBuckets);

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

  // v0.7.0: outlier handling — when showOutliers is OFF, clamp y-axis
  // to a "typical" band so a single 58s spike doesn't flatten the
  // baseline into a one-pixel strip. Outlier buckets aren't dropped:
  // their values stay in the dataset (the line clips at the axis edge),
  // and we annotate them in the alert panel below the chart.
  // Per spec: clamp = (p1, p99) with 20% padding, hard-capped at ±5000 ms.
  const xnt = [];
  for (const v of median) if (typeof v === 'number') xnt.push(v);
  for (const v of stakeW) if (typeof v === 'number') xnt.push(v);
  let yMin = null;
  let yMax = null;
  if (!state.showOutliers && xnt.length > 0) {
    const p1 = percentile(xnt, 1);
    const p99 = percentile(xnt, 99);
    if (p1 != null && p99 != null) {
      const span = p99 - p1;
      const pad = Math.max(span * 0.2, 50); // 20% pad, never less than 50 ms
      let lo = p1 - pad;
      let hi = p99 + pad;
      // Hard cap: outlier-clamped view never spans more than ±5000 ms.
      lo = Math.max(lo, -NETWORK_OUTLIER_THRESHOLD_MS);
      hi = Math.min(hi, NETWORK_OUTLIER_THRESHOLD_MS);
      // Always include 0 — the "real UTC" reference line is a useful anchor.
      if (lo > 0) lo = 0;
      if (hi < 0) hi = 0;
      yMin = lo;
      yMax = hi;
    }
  }

  // v0.7.0: detect outlier buckets in the visible window for the alert
  // list below the chart. Threshold mirrors the y-axis hard cap so the
  // list and the chart agree on "what's an outlier here."
  const outliers = [];
  for (let i = 0; i < data.length; i += 1) {
    const m = median[i];
    const s = stakeW[i];
    const candidates = [];
    if (typeof m === 'number' && Math.abs(m) >= NETWORK_OUTLIER_THRESHOLD_MS) {
      candidates.push(m);
    }
    if (typeof s === 'number' && Math.abs(s) >= NETWORK_OUTLIER_THRESHOLD_MS) {
      candidates.push(s);
    }
    if (candidates.length === 0) continue;
    // Use the larger-magnitude value so the alert reflects the worst signal.
    const worst = candidates.reduce(
      (acc, v) => (Math.abs(v) > Math.abs(acc) ? v : acc),
      candidates[0],
    );
    outliers.push({
      bucket_iso: data[i].bucket_iso,
      bucket_ts: data[i].bucket_ts,
      drift_ms: worst,
      index: i,
    });
  }

  // v0.7.0: triangular markers on the median series at outlier indices,
  // pinned to the y-axis edge so they're visible even when the actual
  // value is clipped. Default pointRadius is 0, so non-outlier points
  // remain invisible — only spikes get the ▲.
  const outlierIndexSet = new Set(outliers.map((o) => o.index));
  const markerPointRadius = data.map((_, i) => (outlierIndexSet.has(i) ? 6 : 0));
  const markerPointStyle = data.map((_, i) =>
    outlierIndexSet.has(i) ? 'triangle' : 'circle',
  );
  const markerPointColor = data.map((_, i) =>
    outlierIndexSet.has(i) ? '#f85149' : 'rgba(0,0,0,0)',
  );

  // v1.2.0: lighter alpha on median + stake-weighted creates a "band"
  // effect where the eye reads the two X1 series as a single envelope
  // around the typical pipeline lag, rather than as two competing lines
  // fighting for attention. Sentinel stays at full alpha + slightly
  // thicker stroke since it's the reference (atomic-disciplined) line
  // operators trust as ground truth.
  const sentinelLabel = t.chart_history_sentinel;
  const datasets = [
    {
      label: t.chart_history_median,
      data: median,
      borderColor: 'rgba(88, 166, 255, 0.95)',
      backgroundColor: 'rgba(88, 166, 255, 0.10)',
      pointRadius: markerPointRadius,
      pointStyle: markerPointStyle,
      pointBackgroundColor: markerPointColor,
      pointBorderColor: markerPointColor,
      pointHoverRadius: 6,
      borderWidth: 1.2,
      tension: 0.15,
      yAxisID: 'yLeft',
    },
    {
      label: t.chart_history_stake,
      data: stakeW,
      borderColor: 'rgba(63, 185, 80, 0.85)',
      backgroundColor: 'rgba(63, 185, 80, 0.08)',
      pointRadius: 0,
      borderWidth: 1.2,
      tension: 0.15,
      yAxisID: 'yLeft',
    },
    {
      label: sentinelLabel,
      data: sentinel,
      borderColor: 'rgba(210, 153, 34, 0.95)',
      backgroundColor: 'transparent',
      pointRadius: 0,
      borderWidth: 1.5,
      tension: 0,
      yAxisID: 'yLeft',
      spanGaps: true,
    },
  ];

  // v0.7.0: y-axis scale config — explicit min/max only when clamping.
  // When showOutliers=true we let Chart.js auto-fit (natural range,
  // 60s spike will be visible).
  const yLeftScale = {
    type: 'linear',
    position: 'left',
    ticks: { color: '#8b949e' },
    grid: { color: '#21262d' },
    title: { display: true, text: t.chart_axis_drift_ms, color: '#8b949e' },
  };
  if (yMin != null && yMax != null) {
    yLeftScale.min = yMin;
    yLeftScale.max = yMax;
  }

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
        yLeft: yLeftScale,
      },
    },
  });

  renderNetworkOutlierAlert(outliers);
  // The "±5000 ms · ▲ marker" hint is only meaningful when clamping is
  // active. Hide it in the natural-range view so the chart-controls row
  // doesn't carry stale text.
  if (el.chartYClampedNote) {
    el.chartYClampedNote.hidden = state.showOutliers;
  }
  // v1.2.0: surface the current bucket aggregation interval in the
  // chart subtitle so the user can tell at a glance whether they're
  // looking at raw 5-min buckets (2d) or 30-min buckets (12d). Replaces
  // the placeholder text the i18n applier puts in.
  if (el.chartHistorySubtitle && t.chart_history_subtitle_template) {
    el.chartHistorySubtitle.textContent =
      t.chart_history_subtitle_template.replace('{bucketLabel}', aggregation.label);
  }
}

// v0.7.0: render the "Chain time anomalies detected" panel below the
// chart. Pattern matches the existing #foundation-alert-card (yellow
// border, hidden when 0 entries). One <li> per outlier bucket, sorted
// chronologically.
function renderNetworkOutlierAlert(outliers) {
  if (!el.networkOutlierAlert || !el.networkOutlierAlertList) return;
  if (!outliers || outliers.length === 0) {
    el.networkOutlierAlert.hidden = true;
    el.networkOutlierAlertList.innerHTML = '';
    return;
  }
  el.networkOutlierAlert.hidden = false;
  el.networkOutlierAlertList.innerHTML = '';
  const sorted = outliers.slice().sort((a, b) => (a.bucket_ts || 0) - (b.bucket_ts || 0));
  for (const o of sorted) {
    const li = document.createElement('li');
    const ts = document.createElement('span');
    ts.className = 'mono';
    ts.textContent = o.bucket_iso || '—';
    const sep = document.createElement('span');
    sep.textContent = ': ';
    const drift = document.createElement('span');
    drift.className = 'mono';
    const sign = o.drift_ms >= 0 ? '+' : '−';
    drift.textContent = `${sign}${formatDriftMagnitude(o.drift_ms)} drift`;
    li.appendChild(ts);
    li.appendChild(sep);
    li.appendChild(drift);
    el.networkOutlierAlertList.appendChild(li);
  }
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
  // v0.6.0: optional client-side stake lens. Server returns all stake
  // levels — user can narrow the view to e.g. ≥1000 XNT here without
  // affecting other dashboards or the backend filter logic.
  const minXnt = state.bestMinStake || 0;
  const rows = minXnt > 0
    ? state.best.filter((b) => (b.stake_xnt || 0) >= minXnt)
    : state.best;
  rows.forEach((b) => {
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

// v1.0.0 Tier 2: Layer 2 clock drift table. Source data is
// `clock_drift.json` (or, on a pre-v1.0 snapshot, the |drift| ≥ 5s
// slice of worst_validators.json — see loadAll). Rendered compact:
// no pagination, no severity filter, no stake-floor lens — Layer 2
// drift entries are operationally rare (typically 0–3 validators).
function renderClockDriftTable() {
  if (!el.clockDriftBody) return;
  el.clockDriftBody.innerHTML = '';
  const rows = Array.isArray(state.clockDrift) ? state.clockDrift : [];
  if (rows.length === 0) {
    if (el.clockDriftEmpty) el.clockDriftEmpty.hidden = false;
    return;
  }
  if (el.clockDriftEmpty) el.clockDriftEmpty.hidden = true;
  rows.forEach((v, i) => {
    const tr = document.createElement('tr');
    tr.classList.add('row-severity-layer2');
    const pubkey = v.vote_account || v.pubkey;
    const modalData = { ...v, pubkey };
    tr.appendChild(td(String(v.rank ?? i + 1)));
    tr.appendChild(pubkeyCell(pubkey, modalData));
    tr.appendChild(driftTd(v.mean_drift_ms));
    tr.appendChild(td(formatMsRaw(v.stddev_drift_ms), { num: true }));
    tr.appendChild(td(formatInt(v.n_samples), { num: true }));
    tr.appendChild(td(formatNum(v.stake_xnt, 0), { num: true }));
    el.clockDriftBody.appendChild(tr);
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
/// v0.6.0: server-side stake gate removed; an optional client-side
/// `worstMinStake` lens lets the user narrow the table to a chosen stake
/// floor without affecting other dashboards.
/// Frontend filters: search by pubkey, severity dropdown, hide-foundation
/// + optional min-stake input.
function applyWorstFilter() {
  const q = state.query;
  const sevFilter = state.severityFilter;
  const minXnt = state.worstMinStake || 0;
  state.filtered = (state.worst || []).filter((v) => {
    if (state.hideFoundation && v.is_foundation) return false;
    if (minXnt > 0 && (v.stake_xnt || 0) < minXnt) return false;
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
      // v1.0.0: severity ordering follows the drift-magnitude bands
      // used by severityFor() (Layer 2 > slow pipeline > normal),
      // not the legacy critical/high/healthy strings.
      av = Math.abs(a.mean_drift_ms || 0);
      bv = Math.abs(b.mean_drift_ms || 0);
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
    // v1.0.0: row tint follows the severity band assigned by severityFor()
    // so the table styling agrees with the badge in the same row. Tier 1
    // (this table) only contains 500 ms ≤ |lag| < 5 s entries, so the
    // tint is dominated by the slow-pipeline class.
    tr.classList.add(`row-${severityFor(v.mean_drift_ms).cssClass}`);
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
  // v1.0.0: Layer 1/Layer 2 framework. Classification comes from the
  // drift magnitude directly so the badge agrees with how the dashboard
  // bins validators in hero #2 and the Anomalies section. Foundation
  // status is rendered as an independent 🏛️ badge — a foundation node
  // with Layer 2 drift (extremely unlikely) would show both 🔴 and 🏛️.
  const e = document.createElement('td');
  e.classList.add('severity-cell');
  const sev = severityFor(v.mean_drift_ms);
  const t = I18N[state.lang];
  const sevIcon = document.createElement('span');
  sevIcon.classList.add('sev-icon', sev.cssClass);
  sevIcon.textContent = sev.icon;
  sevIcon.title = t[sev.labelKey] || sev.labelKey;
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
  // v0.6.0: optional client-side stake lens for best/worst tables.
  // The server returns all stake levels; these inputs purely re-filter
  // what is already in memory. Empty/NaN/<=0 means "no filter".
  if (el.bestMinStake) {
    el.bestMinStake.addEventListener('input', () => {
      const v = parseFloat(el.bestMinStake.value);
      state.bestMinStake = isFinite(v) && v > 0 ? v : 0;
      localStorage.setItem('bestMinStake', String(state.bestMinStake));
      renderBestSynced();
    });
  }
  if (el.worstMinStake) {
    el.worstMinStake.addEventListener('input', () => {
      const v = parseFloat(el.worstMinStake.value);
      state.worstMinStake = isFinite(v) && v > 0 ? v : 0;
      localStorage.setItem('worstMinStake', String(state.worstMinStake));
      state.page = 0;
      applyWorstFilter();
      renderWorstTable();
    });
  }
  // v0.7.0: network-drift chart window selector + outlier toggle. Both
  // operate purely client-side over the existing 7-day history.json —
  // no extra fetches, no server round-trip. Re-render is cheap because
  // the chart already destroys+rebuilds on every refresh tick.
  if (Array.isArray(el.windowButtons)) {
    el.windowButtons.forEach((btn) => {
      btn.addEventListener('click', () => {
        const days = parseInt(btn.dataset.windowDays || '', 10);
        if (!VALID_WINDOW_DAYS.includes(days)) return;
        state.networkDriftWindowDays = days;
        localStorage.setItem('networkDriftWindowDays', String(days));
        syncWindowButtons();
        renderHistoryChart();
      });
    });
  }
  if (el.showOutliers) {
    el.showOutliers.addEventListener('change', () => {
      state.showOutliers = el.showOutliers.checked;
      localStorage.setItem('showOutliers', state.showOutliers ? '1' : '0');
      renderHistoryChart();
    });
  }
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
