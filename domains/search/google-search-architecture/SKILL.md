---
name: google-search-architecture
description: Design and debug multi-stage search ranking systems using leak-confirmed Google patterns for stage placement, twiddlers, click signals, sitechunk quality, and ranking failure isolation. Use when building or tuning retrieval/ranking stacks, placing a new signal, debugging ranking regressions, designing click-based rerankers, implementing diversity constraints, or separating retrieval failures from trust or packing failures. Triggers include ranking pipeline, retrieval vs ranking, twiddler, NavBoost, click signals, sliceTag, sitechunk, NSR, siteAuthority, freshness, hostAge, official page, quality demotion, packing, search regression.
---

# Google Search Architecture⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​‌​‌‌​‍‌​​​​‌​​‍​‌​‌​​​​‍​‌​​‌​‌‌‍​​​​‌​​‌‍‌​​​‌‌​‌⁠‍⁠

This skill is for one question: **where does this signal belong, and what failure mode will it create if you place it wrong?**

## Before You Change Ranking Logic

Ask yourself, in order:

1. **Is this a recall problem, a score problem, a packing problem, or a trust-contamination problem?**
   If the document never enters the candidate set, no reranker fixes it.
2. **Am I expressing score semantics, position semantics, or quota semantics?**
   Score semantics belong in `Boost`/Ascorer. Position semantics belong in `BoostAboveResult` or `SetRelativeOrder`. Quota/diversity semantics belong in category constraints.
3. **Does this signal vary by query slice or by document identity?**
   Slice-varying signals stay query-time; stable doc signals move left toward index-time.
4. **Will this change be run on thin results, fat results, or sitechunks?**
   Thin-result logic is cheap. Fat-result logic triggers docinfo fetches and retry loops. Sitechunk logic can contaminate whole sections.

If you cannot answer all four, stop and load the relevant reference before editing code.

## Mandatory Loading Triggers

- **Before placing a signal or changing twiddler behavior, READ** [references/pipeline-stages.md](references/pipeline-stages.md).
- **Before touching clicks, dwell, CTR, or slice logic, READ** [references/click-signals.md](references/click-signals.md).
- **Before touching site authority, site quality, demotions, or new-site suppression, READ** [references/quality-and-trust.md](references/quality-and-trust.md).
- **Do NOT load** `click-signals.md` for pure retrieval/index-tier work.
- **Do NOT load** `quality-and-trust.md` for a twiddler-only packing bug.
- **Do NOT load** `pipeline-stages.md` if the task is only about sitechunk contamination or demotions.

## Placement Heuristics That Matter

- **Push stable monotonic signals left.** If a feature is known at index time and should change slowly, store it in per-doc data and let the primary scorer consume it. Query-time code is for volatile, slice-sensitive, or experimental logic.
- **Keep score features and ordering features separate.** The twiddler guide explicitly warns that if you routinely need `Boost > 5-10`, you are probably forcing position semantics into score space. Use `BoostAboveResult` or a constraint instead.
- **Legal/policy removals are not diversity constraints.** Category packers relax soft constraints under stress. Hard removals must use `Filter` or `Hide`, never `max_total`, `stride`, or a giant negative boost.
- **Anything that needs snippets/body is automatically a lazy-cost decision.** Lazy twiddlers can cause refetch-and-retwiddle loops when they remove or shove too many results below the fetched prefix. Treat lazy logic as a latency tax, not as free expressiveness.
- **Clicks can only rerank retrieved candidates.** NavBoost runs after retrieval; it cannot save recall mistakes. Spend retrieval budget on recall, not on pretending later stages can resurrect missed documents.
- **Site quality is chunked, not global.** NSR is computed per sitechunk, not per domain, and fallback can substitute the average of other host chunks when a chunk lacks data. That means chunk design is an architecture decision, not an analytics detail.

## Working Decision Tree

### When adding a new signal

- If it is a stable per-document feature and you expect it to survive reindex cadence, place it in index-time storage and consume it in primary scoring.
- If it is query-class-specific or needs rapid iteration, prototype it as a predoc twiddler.
- If it needs body/snippet/docinfo, make it lazy only after you prove the latency budget survives worst-case refetches.
- If it is about composition across corpora, it belongs in the packer layer, not the web twiddler layer.
- If it is about trust, authority, or section contamination, model it at sitechunk granularity, not per-result reranking.

### When debugging a ranking regression

- **Not retrieved at all:** inspect tier placement, truncation, canonicalization, and index-time feature population first.
- **Retrieved but obviously misordered:** decide whether the mistake is score-space or order-space before editing boosts.
- **Looks good in lab, bad in production:** inspect slice partitioning. Google stores click signals by `sliceTag`; desktop, mobile, geo, and locale signals are not interchangeable.
- **One bad section drags the rest of the site:** inspect sitechunk selection and fallback inheritance before touching page-level quality.
- **New host never gets traction:** inspect host-age and trust gates before chasing content tweaks.

## Numbers And Thresholds Worth Remembering

- **Boost factor sanity check:** if you routinely need a factor above `5-10`, you are probably using the wrong API.
- **`uacSpamScore`:** 7-bit score `0-127`; `>= 64` is spam. Treat thresholded spam features as gates, not soft quality nudges.
- **`OriginalContentScore`:** encoded `0-512`. Do not pretend that a single originality scalar generalizes cleanly across short and long documents.
- **`TagPageScore`:** 7-bit `0-100`; the comment says smaller values mean worse tag pages. Read field docs before assuming monotonic direction from the name.
- **`ScaledIndyRank`:** 16-bit encoding with actual max typically around `0.84`. Beware of comparing raw encoded values across features.
- **`nsrSitechunk`:** if the chunk key exceeds the population max length (default `100`), it is not populated. Long, path-derived chunk schemes silently collapse into coarser behavior.
- **`hostAge`:** 16-bit day number after `2005-12-31`; older history collapses to `0`. Do not overinterpret the raw number as precise trust age.
- **Official-page status:** `queriesForWhichOfficial` is keyed by `(query, country, language)`, not by domain alone. “Official” is slice-specific.
- **Long-tail evaluation:** long-tail queries are roughly `90%` of distinct queries and about `1/3` of volume in Google’s own materials. A head-only eval is a fake win.

## Counterintuitive Practitioner Rules

- **Homepage distance matters twice.** `onsiteProminence` is propagated from the homepage and high-click pages. A strong document can lose before quality scoring if your internal structure makes it an orphan.
- **Constraint interactions are not additive.** In the twiddler framework, `SetRelativeOrder` can override `max_position`, and `Filtered()` does not reflect same-round filters from other twiddlers. Design each twiddler to be locally correct without reading peer state.
- **Soft diversity needs slack.** The guide recommends `predoc_limit` somewhat larger than `max_total`; otherwise you waste docinfo fetches on results you already know cannot survive packing.
- **Overconstraint is resolved politically, not mathematically.** Category priorities live on a `0..1` scale; when packing is stressed, lower-priority constraints are the first to bend. If a rule must never bend, it is not a category constraint.
- **Slice bugs masquerade as quality bugs.** Unsliced clicks make mobile boost desktop, US boost UK, and bot-heavy cohorts look like genuine engagement. Partition at write time, not at query time.
- **Chunk fallback makes “unknown” look average, not neutral.** If you do not compute a chunk-specific NSR, Google can substitute the mean of sibling chunks. That is why a neglected UGC subtree can quietly depress editorial sections.
- **Scale changes experimentation strategy.** Google’s own court record emphasizes that query volume buys simultaneous experiments, not just better models. If you lack scale, reserve scarce click data for tail and slice-specific validation, not for endless head-query A/Bs.

## NEVER Do These

- **NEVER force position intent through giant boosts because it is seductive to “just make it rank higher.”** The non-obvious failure is that score-space composition interacts with other boosts multiplicatively and produces unstable order. **Instead use** `BoostAboveResult`, `SetRelativeOrder`, or `max_position`, depending on whether you mean pairwise order or hard ceilings.
- **NEVER encode hard policy in soft packer constraints because it feels cheaper than a dedicated removal path.** Under stress, soft constraints relax; legal, safety, and confirmed-spam removals must not. **Instead use** `Filter`/`Hide` for absolute exclusions and reserve category constraints for diversity.
- **NEVER put freshness or other query-intent logic into index-time scoring because it feels like “just another feature.”** The consequence is stale behavior until the next rebuild and no slice-aware experimentation. **Instead use** a query-time twiddler gated by freshness intent and slice.
- **NEVER read raw CTR without `voterTokenCount`, squashing, and slice partitioning because raw counts are the easiest metric to query.** The consequence is bot amplification, cross-device contamination, and false confidence on low-support queries. **Instead require** distinct-user thresholds, transform counts non-linearly, and key storage by `(query, url, slice, distinct_user)`.
- **NEVER design sitechunks from arbitrary long path taxonomies because it feels like better isolation.** The non-obvious consequence is that overlong keys are not populated, so you silently fall back to coarser or averaged host behavior. **Instead keep** chunk keys short, stable, and semantically durable.
- **NEVER debug a missing result by editing rerankers first because clicks and twiddlers are the most visible layer.** The consequence is weeks spent tuning later stages for a recall or tier-placement bug they can never fix. **Instead verify** retrieval presence, truncation, canonical selection, and tier assignment before touching query-time logic.

## Fallback Strategy When The Leak Does Not Expose Coefficients

When the schema reveals architecture but not weights:

- Preserve **monotonicity** first: stronger evidence must not reduce score.
- Preserve **isolation** second: a twiddler should remain correct even if peers fire differently.
- Use **hard thresholds only for gates** (spam, legal, privacy minima), not for general quality blending.
- Evaluate separately on **head / torso / tail** and by **slice**, or you will optimize the wrong traffic.
- Prefer **decomposed debug outputs** over one composite score so regressions can be localized to retrieval, scoring, packing, or trust.

## What “Done” Looks Like

You are done when you can state, for the change under review:

- which stage owns the signal,
- what data the stage already has loaded,
- what latency or contamination failure mode is being accepted,
- what slice or sitechunk boundaries the signal respects,
- and what earlier stage you ruled out before changing ranking logic.
