# Methodology / Metodologia

EN below — Polska wersja niżej.

---

## EN: What is `Clock::unix_timestamp` and how is it produced?

Every slot in a Solana-style cluster (X1 included) carries a sysvar
`Clock` whose `unix_timestamp` field is intended to approximate the
real-world UTC time at which the slot was confirmed. The chain-wide
value is computed as the **stake-weighted median** of timestamps that
each active validator includes in its vote instructions for that slot.
Validators take their own host clock when signing the vote.

The chain timestamp is therefore only as accurate as the consensus of
validator host clocks. Drift in any one validator's clock — for
instance NTP misconfigurations, kernel bugs, or virtualization quirks —
shows up as drift in the chain timestamp once that validator has enough
stake to influence the median.

## EN: Why per-validator drift is observable

For every block we read, the validator's vote instruction carries:
- the slot it is voting on (`slot_voted_for`)
- the timestamp the voting validator believed to be UTC at the moment
  it signed (`ts_chain`, integer Unix seconds)

Independently, our host validator records the local clock at the moment
each slot was *frozen* locally (`ts_local`, microseconds since epoch).
The two readings refer to the same physical event (the same slot)
observed from two different vantage points: the voting validator and
us. Their difference,

```
drift_ms(validator, slot) = ts_chain * 1000 − ts_local_us / 1000
```

is the offset of that validator's clock relative to ours, modulo network
propagation. Aggregated across many slots and many validators, this
isolates persistent clock skew.

## EN: Sources of measurement error

- **Vote-timestamp resolution is 1 second.** Per single sample the
  irreducible noise is ±0.5 s. Aggregating N independent samples
  reduces this to roughly 0.5 / √N seconds, so 100 samples gives ~50 ms
  resolution and 1000 samples gives ~16 ms.
- **Host clock accuracy of the measurement node.** All drift is reported
  *relative to the host validator's clock*. An NTP-disciplined host
  with a chrony or systemd-timesyncd daemon is typically accurate to
  ≤ 10 ms vs UTC, so absolute interpretations of the numbers should
  budget that.
- **Slot freeze vs. vote-issuance time.** The vote is signed slightly
  *after* the bank freezes; in well-functioning validators the gap is
  a few hundred milliseconds. This shows up as a small positive bias
  for fast voters and negative for slow voters; ranking is meaningful
  but absolute drift has a per-validator pipeline-dependent component.
- **Network propagation.** Our host hears about a slot after the
  producer has already moved on. This adds ≤ 100 ms typical noise to
  the local-clock side.
- **Vote variants.** Different versions of the vote instruction encode
  the timestamp in slightly different layouts; the parser handles the
  current set (Vote, VoteSwitch, UpdateVoteState[Switch],
  CompactUpdateVoteState[Switch], TowerSync[Switch]). Unknown variants
  are logged and skipped, which under-counts samples for affected
  validators rather than producing false readings.

## EN: How to interpret the dashboard

- **Hero number** — median across all observed validators of each
  validator's median drift over the last 24 h. Good = green (|drift|
  < 200 ms), warn = amber (< 1 s), bad = red (≥ 1 s).
- **Stake-weighted drift** — same idea but weighted by activated stake.
  Closer to the real chain timestamp behaviour.
- **History chart** — the network median and stake-weighted drift,
  bucketed in 5-minute windows over the last 7 days.
- **Histogram** — how the per-validator means are distributed.
  A healthy network is a tight peak around zero; long tails indicate
  poor clock hygiene at some validators.
- **Validator ranking** — sortable table; default sort is by absolute
  mean drift, descending. The `impact` column is `mean_drift × stake`,
  expressed in milliseconds × XNT, and is the rough magnitude by
  which that single validator pulls the network median (good for
  prioritising who to contact).

---

## PL: Czym jest `Clock::unix_timestamp` i jak powstaje?

Każdy slot w klastrze typu Solana (w tym X1) niesie ze sobą sysvar
`Clock`, którego pole `unix_timestamp` ma przybliżać realny czas UTC
zatwierdzenia slotu. Wartość chain-wide jest obliczana jako
**stake-ważona mediana** timestampów, które każdy aktywny walidator
zawiera w swoich vote instructions dla tego slotu. Walidator bierze
swój własny zegar hosta podczas podpisywania.

Tym samym chain timestamp jest dokładny tylko o tyle, o ile zegary
walidatorów są skonsensusowane. Drift jednego walidatora — np.
zła konfiguracja NTP, bug jądra czy quirky wirtualizacji — pokaże
się w chain timestamp jeśli ten walidator ma odpowiednio duży stake
żeby wpływać na medianę.

## PL: Dlaczego drift per-validator jest mierzalny

Dla każdego bloku, vote instruction walidatora zawiera:
- slot, na który głosuje (`slot_voted_for`)
- timestamp uznany przez głosującego za UTC w momencie podpisywania
  (`ts_chain`, sekundy unix)

Niezależnie nasz host zapisuje lokalny zegar w momencie *zamrożenia*
każdego slotu (`ts_local`, mikrosekundy od epoki). Oba odczyty dotyczą
tego samego fizycznego zdarzenia (tego samego slotu), obserwowanego
z dwóch punktów widzenia. Ich różnica:

```
drift_ms(validator, slot) = ts_chain * 1000 − ts_local_us / 1000
```

to przesunięcie zegara walidatora względem naszego, modulo propagacja
sieciowa. Po agregacji po wielu slotach i walidatorach izoluje się
trwałe odchylenie zegara.

## PL: Źródła błędu pomiaru

- **Rozdzielczość timestampu vote = 1 sekunda.** Per pojedynczy pomiar,
  nieredukowalny szum to ±0.5 s. Agregacja N niezależnych pomiarów
  redukuje to do ~0.5 / √N s, więc 100 pomiarów = ~50 ms, 1000 = ~16 ms.
- **Dokładność zegara naszego hosta.** Wszystko jest mierzone
  *względem zegara hosta*. NTP-zsynchronizowany host (chrony,
  systemd-timesyncd) jest typowo dokładny do ~10 ms względem UTC.
  Interpretacje absolutne powinny tę rezerwę uwzględniać.
- **Czas zamrożenia slotu vs. moment podpisania votu.** Vote jest
  podpisywany nieco *po* zamrożeniu banku; u zdrowych walidatorów
  to kilkaset ms. Daje to niewielki dodatni bias dla szybkich,
  ujemny dla wolnych — ranking pozostaje sensowny, ale wartość
  bezwzględna driftu zawiera komponentę pipeline'u.
- **Propagacja sieciowa.** Nasz host dowiaduje się o slocie po
  producencie. To dodaje ~100 ms typowego szumu po stronie zegara
  lokalnego.
- **Warianty vote.** Różne wersje instrukcji vote kodują timestamp
  w trochę innych layoutach; parser obsługuje aktualny zestaw (Vote,
  VoteSwitch, UpdateVoteState[Switch], CompactUpdateVoteState[Switch],
  TowerSync[Switch]). Nieznane warianty są logowane i pomijane —
  zaniża to liczność próbki dla dotkniętych walidatorów, ale nie
  wprowadza fałszywych odczytów.

## PL: Jak interpretować dashboard

- **Liczba w nagłówku** — mediana po walidatorach z każdej mediany
  driftu z ostatnich 24 h. Zielony = |drift| < 200 ms, żółty < 1 s,
  czerwony ≥ 1 s.
- **Stake-weighted drift** — to samo, ale ważone aktywnym stake.
  Bliższe rzeczywistemu zachowaniu chain timestampa.
- **Wykres historii** — sieciowa mediana i stake-weighted drift w
  oknach 5-minutowych przez ostatnie 7 dni.
- **Histogram** — rozkład średnich per-validator. Zdrowa sieć to
  wąski pik przy zerze; długie ogony oznaczają złą higienę zegarów
  części walidatorów.
- **Ranking** — sortowalna tabela; domyślnie po wartości bezwzględnej
  średniego driftu, malejąco. Kolumna `impact` = `mean_drift × stake`,
  w milisekundach × XNT — wskazuje, który walidator najmocniej
  pociąga sieciową medianę (przydatne do ustalania priorytetów
  kontaktów).
