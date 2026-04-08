---
name: google-search-architecture
description: Design multi-stage search ranking pipelines using patterns confirmed by the May 2024 Google Content Warehouse API leak (Mustang, Ascorer, NavBoost, Twiddlers, NSR, CRAPS, siteAuthority, hostAge). Use when building retrieval or ranking systems, adding a new ranking signal, debugging ranking regressions, implementing click-based re-ranking, designing index tiers, partitioning quality scoring across a domain, or deciding which pipeline stage a given signal belongs in. Triggers include "ranking pipeline", "retrieval vs ranking", "click signals", "twiddler", "re-ranker", "search quality", "ranking regression", "index tier", "freshness boost", "search architecture", "NavBoost", "CRAPS", "sitechunk", "NSR", "siteAuthority", "pogo sticking", "last longest click", "search sandbox".
---

# Google Search Architecture

## What The Leak Actually Forces You To Rethink

The May 2024 Content Warehouse leak (2,596 modules, 14,014 attributes) deliberately excludes weights and thresholds. What it exposes is Google's **pipeline shape** and the *categories* of signals at each stage. If you build a search system and ignore the shape, you will rebuild mistakes Google abandoned a decade ago.

Before writing any scoring code, internalize this ordering:

```
Trawler → Alexandria / SegIndexer / TeraGoogle → Mustang(Ascorer) → SuperRoot(Twiddlers) → Tangram → GWS
          (tiered index, physical hardware)    (dense scoring)    (isolated re-rankers)   (universal packer)
```

Every stage has a **different latency budget, a different data access pattern, and a different failure mode**. Putting a signal in the wrong stage is the #1 architectural error in search — and the leak's real contribution is telling you which stage each class of signal lives in.

## Five Axioms

### Axiom 1 — NavBoost only operates on what Mustang already retrieved
Pandu Nayak testified under oath: "you get NavBoost only after they're retrieved in the first place." **Click signals cannot rescue a document that failed retrieval.** If your system buries good content at the candidate-generation stage, no downstream re-ranker fixes it. Spend retrieval budget on *recall*, not quality — quality is the next stage's job.

### Axiom 2 — Twiddlers are isolated by design, not by accident
Superroot runs **>65 twiddlers in WebMixer alone** (a 2018 number; likely hundreds today). Twiddlers **cannot see each other's decisions** — they only emit *constraints* (`Boost`, `BoostAboveResult`, `Filter`, `SetRelativeOrder`, `max_total`, `max_position`, `stride`, `Annotating`), and the framework reconciles them. Write every re-ranker as if it were the only one. Any coordination logic across re-rankers is a concurrency bug waiting to happen at query time.

### Axiom 3 — Signals belong where their data is already loaded
Twiddlers have two execution phases: **predoc** runs on several hundred "thin" results *before* docinfo (snippets/body) is fetched; **lazy** runs on a top prefix *after* an RPC fetches docinfo. Predoc sees breadth; lazy sees depth. **If your re-ranker needs body content, it MUST be lazy** — forcing it predoc either corrupts results or doubles the latency budget from a second fetch pass. If it only needs PerDocData (pagerank, NSR, hostAge), go predoc.

### Axiom 4 — Quality is site-chunked, and silence is averaged
NSR ("Normalized Site Rank") is computed per **sitechunk** — subsections of a domain. The field `nsrdataFromFallbackPatternKey` is the smoking gun: **when a chunk has no computed NSR, Google averages the chunks it does have and applies that mean.** One spammy subdirectory pulls down every editorial page through fallback inheritance. Partition your quality scoring at the same granularity you partition content, or accept the contamination.

### Axiom 5 — The ground-truth click is the one that ends the session
CRAPS stores `goodClicks`, `badClicks`, `lastLongestClicks`, `squashedClicks`, `unsquashedLastLongestClicks`, `impressions`, and `voterTokenCount`. **`lastLongestClicks` — the click that terminated the user's search session — is the strongest positive signal in the taxonomy.** Optimize evaluation around "did the user stop searching?", not around CTR. A pogo-stick back to the SERP is not a null event; it is logged as a `badClick` and actively demotes you.

## Where To Put A New Signal — Decision Tree

When someone says "let's rank X higher when Y":

```
Is Y a per-doc property knowable at index time?
├── Yes → indexer computes it, store on PerDocData, consume in Ascorer
└── No → Does Y require the full candidate body?
         ├── Yes → lazy twiddler (budget for the docinfo fetch)
         └── No  → Is Y a query-class / user-behavior signal?
                  ├── Yes → predoc twiddler over thin results
                  └── No  → Is Y a cross-corpus composition signal?
                           ├── Yes → Tangram (universal packer), not a twiddler
                           └── No  → it's site-level → NSR sitechunk feature
```

**Stop-check**: If your answer was "multiply it into Ascorer", ask one more question — *how often will this need to change?* Ascorer modifications touch billions of documents at index-build time and have long dev cycles. Twiddlers exist specifically so you can experiment without a reindex. **Rapid iteration → twiddler. Massive-data offline feature → Ascorer.** Getting this wrong means you'll ship one experiment per quarter instead of per week.

## NEVER

- **NEVER put a freshness boost in Ascorer.** Seductive because it feels like "scoring". Consequence: you bake the query-class decision into the index and cannot respond to trending events until the next rebuild. Instead: `FreshnessTwiddler` with `should_apply()` gated by runtime freshness-intent detection on the query.

- **NEVER use `Boost` to push a result to the second page.** The Superroot twiddler design doc explicitly warns against it. Seductive because a large negative `Boost` works in isolation. Consequence: another twiddler's positive `Boost` can cancel yours, and because twiddlers run in isolation the interaction is undefined. Instead: use `Filter`, `max_position`, or `SetRelativeOrder` — methods that encode semantic intent the framework can reconcile.

- **NEVER dedupe domain results in a lazy twiddler when you could do it predoc.** Seductive because snippets make dedup easier to reason about. Consequence: pagination bugs — results that appear on page 1 vanish from page 2 because reordering a lazy prefix causes the framework to refetch and the prefix boundary shifts. Use predoc; if you must go lazy, return `stride` or `max_total` constraints, not manual reorders.

- **NEVER read clicks without weighting by `voterTokenCount`.** Seductive because raw CTR is one line of code. Consequence: 20 clicks from one IP range move your signal identically to 20 clicks from 20 users, which is exactly what `craps-ip-prior` was built to catch. Below a minimum distinct-user threshold NavBoost suppresses the signal *entirely* (privacy filter). Always: distinct-user count first, then ratio, then squash.

- **NEVER store clicks unsliced.** Seductive because a flat store is simpler. Consequence: desktop clicks boost mobile rankings and vice versa. Google stores signals per `sliceTag` (device / country / locale); clicks are not interchangeable across segments. Partition at write time, not read time.

- **NEVER average `OriginalContentScore` across long and short documents.** Seductive because you want one unified quality score. Consequence: `OriginalContentScore` is a 0–512 originality signal tuned for *short* content; long documents have different proxies (`shingleInfo`, `chardScore`). A single model biases your ranker against both regimes at once.

- **NEVER treat the sandbox as a grace period.** Seductive interpretation: "new sites are eased in." Reality: the `hostAge` field in PerDocData is documented as being used "to *sandbox fresh spam in serving time*" — it is an **anti-spam filter**, not training wheels. Your new domain is treated as hostile until proven otherwise. Plan for ~6 months of intentionally depressed rankings; don't chase "fixes" that aren't.

- **NEVER let a bad subdirectory stay indexed because "it's a small part of the site".** Via `nsrdataFromFallbackPatternKey`, an un-scored chunk inherits the *mean* of scored chunks. One neglected UGC corner drags down your editorial pages through fallback inheritance. Fix it, `noindex` it, or move it to a separate host.

- **NEVER treat a high CTR on one query as a site-wide quality proof.** CRAPS stores signals keyed by query *and* slice. `chardVariance` and `chardScoreVariance` explicitly penalize **inconsistency** — one viral page can't carry a site whose average chunk score is low. The system rewards distributional consistency over peaks.

- **NEVER rely on `robots.txt` + disavow assumptions from pre-leak SEO lore.** The leak has no runtime disavow field — disavow data is almost certainly crowd-sourced training input for spam classifiers, not a live ranking input. Disavow decisions do not propagate at query time.

## Common Debugging Scenarios

| Symptom | Most likely stage | What to check |
|---|---|---|
| Good content absent from candidate set | Retrieval (Alexandria/Mustang) | Index tier assignment, token-truncation cap, `titlematchScore` |
| Ranks #50, should be top 10 | Ascorer | `siteAuthority`, `hostNsr`, length-dependent quality model mismatch |
| Top 10 but wrong order on news queries | Twiddler (predoc) | `FreshnessTwiddler.should_apply()` query-class gate |
| Ranks well then decays over weeks | NavBoost | `lastLongestClicks` ratio, distinct `voterTokenCount`, IP prior scaling |
| New site gets no traction for months | `hostAge` sandbox | Expected behavior. Serve consistent quality and wait. |
| One section's decline drags the whole site | NSR chunk contamination | Audit sitechunk fallback inheritance, isolate or remove the bad chunk |
| SERP looks fine in lab, bad in prod | Twiddler interaction | Check `sliceTag` segmentation; lab likely tests single-slice |

## Document Truncation Is Real

Mustang truncates tokens at a hard cap. The leaked `DocProperties.numTokens` documentation literally states: *"we drop some tokens in mustang and also truncate docs at a max cap"*. Content past the cap is invisible to the ranker. Frontload. For long-form indexes you operate, extract the most important passages to the first N tokens or index sub-documents.

## Reference Files — Load On Demand

- **READ `references/pipeline-stages.md`** before adding any new stage, modifying stage boundaries, or implementing a twiddler. Contains exact responsibilities of Trawler / Alexandria / SegIndexer / TeraGoogle / Mustang / Ascorer / SuperRoot / Tangram / GWS, and the full twiddler method catalog (`Boost`, `BoostAboveResult`, `Filter`, `SetRelativeOrder`, `max_total`, `max_position`, `stride`, `Annotating`) with when each is semantically correct.

- **READ `references/click-signals.md`** before building or debugging any click-based re-ranker. Contains the full CRAPS taxonomy, the squashing function semantics (why linear scaling fails), voter-token suppression rules, `craps-ip-prior` scaling stages, and `sliceTag` segmentation design.

- **READ `references/quality-and-trust.md`** before touching site-level authority, topical scoring, or demotions. Contains NSR sitechunk mechanics and fallback inheritance, `siteFocusScore` / `siteRadius` topical geometry, `hostAge` sandbox enforcement, the Panda / babyPanda / exactMatchDomainDemotion modular demotions, and `chardVariance` consistency penalties.

**Do NOT load** `click-signals.md` for pure retrieval or index-structure questions.
**Do NOT load** `quality-and-trust.md` for twiddler-only design work.
**Do NOT load** `pipeline-stages.md` when the task is purely about per-site quality tuning.
