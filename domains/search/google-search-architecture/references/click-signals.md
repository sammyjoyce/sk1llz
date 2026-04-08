# Click Signals — CRAPS, Squashing, Voter Tokens, IP Priors

Load this file before building or debugging any click-based re-ranker. Do not load for retrieval, indexing, or quality-only work.

## What "NavBoost" Actually Is

NavBoost is a **re-ranker**, not a retrieval system. Pandu Nayak, under oath at the DOJ antitrust trial: *"you get NavBoost only after they're retrieved in the first place."* This matters more than it sounds:

- NavBoost cannot pull a page into the candidate set.
- NavBoost cannot compensate for a catastrophic Ascorer score.
- NavBoost is trained on a **rolling 13-month window** (~395 days) of aggregated click and impression data.
- NavBoost runs at query time, inside SuperRoot, typically as a predoc twiddler.
- Internal Google comms (2019, from DOJ evidence): one engineer claimed NavBoost was "likely more powerful than the rest of ranking combined on click and precision metrics." Take that as an upper bound but not as gospel.

## The CRAPS Module — Full Click Taxonomy

CRAPS = **C**lick and **R**esults **P**rediction **S**ystem (per ex-Googler, unconfirmed officially). It is the storage substrate for NavBoost. Every click signal is a **float**, not an integer — Google stores continuous, normalized values, not raw counts.

### The attributes

| Attribute | What it measures | Signal polarity |
|---|---|---|
| `impressions` | Times a URL was shown | Neutral — denominator |
| `clicks` | Total clicks from SERP | Neutral — raw count |
| `goodClicks` | Clicks followed by a long dwell / no immediate return to SERP | **Strong positive** |
| `badClicks` | Clicks followed by fast return to SERP (the pogo stick) | **Strong negative** |
| `lastLongestClicks` | Clicks that were the **last** and **longest** of the session — the click that ended the search | **Strongest positive** |
| `squashedClicks` | Post-squashing-function transformed click count | Non-linear, dampened |
| `unsquashedClicks` | Pre-transformation click count | Raw, for comparison |
| `unsquashedLastLongestClicks` | Raw last-longest before squashing | Raw comparison |
| `sliceTag` | Segmentation key (device / country / locale / other) | **Partitioning key** |
| `voterTokenCount` | Number of distinct "voter tokens" (unique users) contributing | **Confidence gate** |
| `packedIpAddress` | Packed net-byte-order IP for craps-ip-prior lookup | Anti-manipulation |
| `unscaledIpPriorBadFraction` | IP-level historical bad-click fraction (pre-scaling) | Anti-manipulation |

### Key insight: session-termination beats click volume

`lastLongestClicks` is the click that made the user **stop searching**. A page with 100 clicks and 5 last-longest is worse than a page with 30 clicks and 20 last-longest. If you're modeling click signals in your own system, the primary target variable should be "did this click end the session?", not "was there a click?"

### Pogo-sticking is explicitly penalized

`badClicks` is defined as a click where the user quickly returned to SERP. Google spent years publicly claiming pogo-sticking was "made up crap" (Gary Illyes, 2019). The leak has it as a named, stored, float-valued attribute.

## The Squashing Function — Why Linear Scaling Fails

The CRAPS module distinguishes `squashed` and `unsquashed` variants of click metrics. The squashing function is a **non-linear compression** applied before the signal feeds re-ranking. The design intent is clear from the schema:

1. Prevents linear click manipulation — 10× the clicks does **not** mean 10× the impact.
2. Applies diminishing returns above a threshold.
3. Makes low-volume genuine signals comparable to high-volume popular signals.

**Practical consequence for your own design**: if you're building a click-based re-ranker, use a sigmoid, log, or Box-Cox transform on click counts *before* they enter the scoring function. Otherwise a single viral event will dominate your rankings for weeks.

```
raw_clicks → squash(raw_clicks) → ratio_with_impressions → scoring
```

Not:

```
raw_clicks → scoring  ← anti-pattern
```

## Voter Tokens — The Confidence Gate

`voterTokenCount` is the number of **distinct users** contributing click data for a URL/query pair. It exists because:

1. **Privacy filtering**: below a minimum distinct-user threshold, the data is too small to anonymize safely and the signal is suppressed entirely.
2. **Anti-manipulation**: raw click volume from few users is ignored in favor of distributed signal from many users.
3. **Low-traffic tension**: a highly specialist page with 14 distinct users all giving long clicks has an exceptional ratio but may still be suppressed until it accumulates more voters. The ratio alone is not enough.

**Design rule for your own system**: every click aggregator should key by `(url, query, distinct_user_id)` and refuse to emit a signal until a minimum-user threshold is met. The exact threshold isn't leaked; pick one based on your traffic distribution and tune it until bot bursts stop moving the needle.

```python
def emit_click_signal(url, query):
    distinct_users = count_distinct(clicks_for(url, query))
    if distinct_users < MIN_DISTINCT_USERS:
        return None  # suppression
    ratio = long_clicks / total_clicks
    return squash(ratio, distinct_users)
```

## IP Prior — Why CTR Manipulation Degrades

The `craps-ip-prior.h` and `craps-penalty.cc` references in the leaked schema describe an **IP-level reputation system**:

- `packedIpAddress`: populated only when the system looked up the IP prior at retrieval time.
- `unscaledIpPriorBadFraction`: the raw "historical bad-click fraction" for the IP, **before** a "linear scaling / offset / min / max" transformation is applied.

Interpreted: every IP has a reputation score based on how often clicks from it have been classified as spam, bot, or manipulation. The scaling transform implies a **threshold system** — clicks from "clean" IPs count fully, clicks from moderately suspect IPs are progressively devalued, and clicks from the worst offenders are **discarded entirely**.

**Failure mode of CTR manipulation services**: they route through residential proxies, which look clean initially. As clicks accumulate from those IPs without natural browsing patterns (varied dwell time, natural follow-up queries, diverse destinations), the bad-fraction climbs. Manipulation degrades over time even if the proxy pool is large.

**Design implication**: if you build your own click-based ranker, carry an IP reputation prior alongside the click signal, and blend it multiplicatively into the weight. Do not rely on IP filtering alone — a slow, continuous discounting based on behavioral reputation is more robust than a hard block list.

## `sliceTag` — Signals Are Not Interchangeable

Every CRAPS signal is stored alongside a `sliceTag` string: "This field can be used by the craps pipeline to slice up signals by various attributes such as device type, country, locale etc."

What this means:

- Desktop clicks do not boost mobile rankings (and vice versa).
- US clicks do not boost UK rankings.
- English clicks do not boost Spanish rankings.
- Signals are aggregated **separately per slice**, plus a default unsliced bucket.

Speculatively, the leak also suggests Google can create **ad-hoc slice tags** for temporary events (elections, pandemics) — consistent with the DOJ antitrust evidence about COVID and election whitelists.

**Design rule**: partition your click store at **write time** by slice, not at read time by filter. A single flat click log cannot be retrofitted into sliced signals correctly once the slicing dimensions have drifted.

## What The Leak Does *Not* Tell You

- Exact squashing curve parameters.
- Exact voter-token minimum.
- Weights assigned to `goodClicks` vs `lastLongestClicks` vs `badClicks`.
- IP bad-fraction scaling constants.
- Whether `sliceTag` segmentation is strict or blended.

Do not invent numbers and present them as from the leak. The architecture is confirmed; the coefficients are not.

## Design Checklist For Your Own Click-Based Re-Ranker

Before shipping, verify each item:

1. [ ] Signal is computed per `(url, query, slice)` tuple, not per URL alone.
2. [ ] Distinct-user count is tracked and below-threshold URL/query pairs are suppressed.
3. [ ] Raw counts are squashed (log / sigmoid) before entering scoring.
4. [ ] An IP-level reputation prior discounts contributions from historically bad sources.
5. [ ] "Last long click" (session-terminating satisfaction) is a distinct, tracked, higher-weight signal than "any click."
6. [ ] Pogo-stick events (fast return to SERP) are logged as explicit negative signals, not as null events.
7. [ ] The re-ranker runs **after** retrieval, not inside it.
8. [ ] The signal's rolling window is bounded (Google uses 13 months; pick a window that matches your content freshness).
