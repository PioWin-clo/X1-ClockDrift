# X1 ClockDrift

Pomiar driftu `Clock::unix_timestamp` na blockchainie X1, na żywo.

**Dashboard na żywo**: https://piowin-clo.github.io/x1-clockdrift/

[English README](README.md) · [Metodologia](docs/methodology.md)

## Co mierzymy

X1 (SVM-fork Solany, klient walidatora Tachyon) generuje per-slot wartość
`unix_timestamp`, liczoną jako stake-ważona mediana timestampów
raportowanych przez wszystkich aktywnych walidatorów w instrukcjach vote.
Ten projekt mierzy, ile każdy walidator z osobna odbiega od realnego UTC
mierzonego lokalnie.

Dla każdego slotu zamrażanego na hostującym walidatorze zapisujemy lokalny
zegar (mikrosekundy). Z kolejnych bloków zbieramy każdą instrukcję vote —
każda zawiera timestamp raportowany przez głosującego walidatora dla
pewnego wcześniejszego slotu. Łącząc te dane wyznaczamy
`drift = chain_timestamp − local_clock` dla każdej pary (walidator, slot),
którą udało się dopasować.

## Architektura

- Daemon działa na jednym walidatorze X1 (host).
- Czyta `validator.log` żeby pobrać czas lokalny zdarzeń `bank frozen`
  dla **każdego** slotu, z rozdzielczością mikrosekundową.
- Odpytuje **publiczne RPC X1** (`https://rpc.mainnet.x1.xyz`) o vote
  instructions, samplując **jeden blok na ~500 slotów (~3 min)** —
  to około **600 wywołań `getBlock` na dobę**, plus jedno
  `getVoteAccounts` na godzinę dla snapshotów stake. Używamy publicznego
  RPC ponieważ lokalny walidator jest uruchomiony bez `--full-rpc-api`
  i nie eksponuje `getBlock`; nie mamy możliwości zmiany jego konfiguracji.
  Vote instructions są dekodowane po stronie RPC przez
  `encoding=jsonParsed`, więc daemon nie wykonuje żadnego parsowania
  na poziomie bajtów.
- Baza SQLite akumuluje surowe obserwacje + odświeżane snapshoty stake.
- Co 5 minut daemon przelicza agregaty per-validator i per-network,
  eksportuje JSON-y do klona tego repozytorium i pushuje na gałąź `data`.
- Dashboard jest zawartością gałęzi `data` serwowaną przez GitHub Pages.

## Struktura repo

```
.
├── daemon/                Daemon w Rust (binarka x1cd)
├── frontend/              Dashboard — vanilla HTML + JS
├── install/               Skrypty instalacyjne + systemd unit
├── docs/                  Metodologia
└── .github/workflows/     CI + deploy Pages
```

## Build ze źródeł

```bash
cargo build --release
```

Binarka pojawi się jako `target/release/x1cd`.

## Uruchomienie lokalne (dev)

```bash
cargo run --bin x1cd -- --config ./config.toml run
```

API HTTP i dashboard dostępne na `127.0.0.1:8088` (konfigurowalne).
Daemon nie wystartuje jeśli istnieje plik kill-switch.

## Instalacja na walidatorze

Patrz [install/install.sh](install/install.sh). W skrócie:

```bash
sudo -u x1pio bash install/install.sh
sudo systemctl start x1cd
journalctl -u x1cd -f
```

Installer generuje deploy key SSH, prosi o zarejestrowanie go w GitHub
z uprawnieniami do zapisu, klonuje gałąź `data`, pisze `config.toml`
i instaluje unit systemd.

## Gwarancje operacyjne

- `CPUQuota=20%`, `MemoryMax=512M`, `Nice=19`, `IOSchedulingClass=idle` —
  walidator ma absolutny priorytet.
- `Type=notify` + `WatchdogSec=120` — systemd zabija daemon jeśli przestał
  pingować.
- `touch /home/x1pio/strontium-meter/STOP` powoduje czysty exit w ≤ 5 s.

## Licencja

Apache-2.0. Zobacz [LICENSE](LICENSE).
