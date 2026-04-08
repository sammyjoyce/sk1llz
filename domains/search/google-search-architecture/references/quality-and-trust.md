# Quality & Trust — NSR, Site Authority, Topical Geometry, Demotions

Load this file before modifying site-level authority computations, topical scoring, or building a demotion system. Do not load for twiddler-only or click-only work.

## NSR = Normalized Site Rank

NSR is a **site-level signal** — specifically, a site-wide quality signal stored in `QualityNsrNsrData` and used throughout the `Q*` quality system. Mike King's guess (plausible) is that NSR stands for *Neural Semantic Retrieval*, but internally it is treated as *Normalized Site Rank* when composed into scoring.

### NSR is computed per **sitechunk**, not per site

A **sitechunk** is a subsection of a domain — sometimes a subdirectory, sometimes a topical cluster, sometimes a site partition from embeddings. NSR is computed for each chunk independently, and the scoring system blends the chunk NSR with page-level signals.

### The fallback-inheritance trap

The single most dangerous field in `QualityNsrNsrData`:

> `nsrdataFromFallbackPatternKey` *(boolean)* — indicates if the NSR data is from a fallback pattern key.

When a document lives in a chunk for which NSR has **not** been computed, the system applies the **mean of other chunks' NSR values**. Practical consequence:

- A brand-new section of your site inherits the average of existing sections.
- A low-quality section pulls down the mean, which then pulls down the un-scored sections.
- Removing a bad chunk from the index can *raise* the scores of other chunks, because they stop pulling against its pull on the mean.

**If you build a similar system**, decide explicitly whether un-scored partitions should inherit the global mean, the parent partition's score, or a neutral prior. All three are defensible. The Google choice — inherit the mean — is the most contamination-prone.

### Important NSR fields

| Field | What it means | Operational meaning |
|---|---|---|
| `nsr` | The primary NSR score | Site-level quality input to Q* |
| `priorAdjustedNsr` | NSR adjusted by prior (above/below average in its slice) | Used for comparative scoring |
| `chardEncoded` | Site-level CHARD (content quality predictor) | Predicts page quality from content |
| `chardVariance`, `chardScoreVariance` | Distribution of CHARD across chunks | **Inconsistency is penalized** |
| `tofu` | Site-level quality predictor based on content | Composite quality proxy |
| `healthScore` | Categorical site health | Used as gate in composite scores |
| `clutterScore` | Penalty for distracting resources (ads, popups, heavy JS) | **Site-wide deduction** |
| `siteAutopilotScore` | Aggregate URL autopilot scores | Sitechunk composition |
| `siteFocusScore` | How tightly focused the site is on one topic | Topical geometry |
| `siteRadius` | How far a given page's topic deviates from the site centroid | Topical geometry |
| `site2vecEmbedding` | A vector representation of the site (like word2vec, but for sites) | Used for similarity, clustering |
| `smallPersonalSite` | Binary flag for "small personal site" | Can trigger specific handling |
| `chromeInTotal` | Total Chrome views for the site | Engagement signal from Chrome |
| `pnav`, `pnavClicks` | Navigational clicks / intent | Brand-query signal |
| `exactMatchDomainDemotion` | Demotion when the domain name is a suspicious exact keyword match | Anti-spam |
| `nsrOverrideBid` | Override NSR as a bid in Q* when > 0.001 | Escape hatch for specific cases |

## Topical Geometry: siteFocusScore and siteRadius

`siteFocusScore` measures how concentrated a site is on a single topic. `siteRadius` measures how far an individual page's topic sits from the site's topical centroid.

The geometric reading:

- A site is a **point cloud** in topic-embedding space.
- `siteFocusScore` is the inverse of the cloud's diameter.
- `siteRadius` for a page is its Euclidean-ish distance from the cloud's centroid.
- Adding off-topic content expands the cloud → reduces focus → reduces authority in the original topic.

**Counterintuitive consequence**: publishing an excellent article outside your niche can **dilute** your authority within your niche, because it moves your centroid and grows your radius. The page itself may do well, but the site-wide authority halo weakens.

**Design rule for multi-topic sites**: partition into distinct hostnames or distinct sitechunks so each has its own tight centroid. Don't run a cooking blog and a crypto blog on the same domain and expect either to rank.

## `hostAge` — The Sandbox Is Real And It Is Hostile

`hostAge` lives in `PerDocData`. The field's documentation explicitly says it is used "to sandbox fresh spam in serving time." Two things follow:

1. **The sandbox exists.** Google spokespeople denied this for a decade.
2. **It is an anti-spam filter, not a grace period.** The framing matters: new hosts are treated as *suspect* until they accumulate enough evidence to be trusted. They are not being eased in gently.

### Operational implications

- New domains face intentional ranking suppression, roughly aligned with a ~6-month floor (not explicitly stated in the leak; observed community behavior).
- `RegistrationInfo` includes `createdDate` and `expiredDate`, so Google tracks domain lifecycle via WHOIS equivalents.
- Re-registering an expired domain does not reset hostAge to zero — the continuity is tracked.
- Content that would trust-establish a natural site (author bios, About, verified business info, consistent publishing) also helps dampening lift.

**Design rule if you build your own sandbox**: mirror the shape, not just the fact. A ramp that depends on *diverse evidence* (distinct visitors over time, distinct referring sources, topical consistency, no spam-pattern tripwires) is robust. A pure age-based ramp is gameable by buying old domains.

## The Panda Family Of Demotions

Panda was not retired. It was decomposed into modular demotion attributes that live in `CompressedQualitySignals`:

| Demotion | What it targets |
|---|---|
| `pandaDemotion` | The classic Panda low-quality site penalty |
| `babyPandaDemotion` | A lighter / earlier-stage Panda variant |
| `babyPandaV2Demotion` | A second-generation variant |
| `navDemotion` | The inverse of NavBoost — demotion based on poor user engagement |
| `exactMatchDomainDemotion` | Anti-spam for keyword-stuffed exact-match domains |
| `serpDemotion` | Demotion based on SERP experience measurement (SDS — SERP Demotion Score) |
| `productReviewDemotion` | Low-quality product review penalty |
| `anchorMismatchDemotion` | Penalty when anchor text does not match the target page's topic |
| `unauthoritativeDemotion` | Generic authority-deficit penalty |
| `lowQuality` | NSR-derived flag; when the normalized site rank is too low, the site is flagged |

**The implied architecture**: demotions are **additive in the log domain**. A site can be hit by multiple demotions at once, and each has its own trigger logic. A page can pass Ascorer, pass NavBoost, and still be suppressed by a product-review-quality twiddler.

**Design rule**: if you build a demotion system, make demotions *orthogonal* and *composable*. A single "quality score" conflating ten concerns is unmaintainable. Google's approach is many named demotions, each addressing one failure mode.

## `siteAuthority` — The Long-Denied Metric

`siteAuthority` is stored in `CompressedQualitySignals`. Google spokespeople, including John Mueller and Gary Illyes, denied having a "website authority score" for years. The leak has it as a named field. It is an input to the `Q*` system alongside NSR, chardScores, clutterScore, and others.

The leak does **not** reveal how it is computed. Reasonable inferences from surrounding fields:

- Aggregates link-graph authority (multiple PageRank variants: `pagerank`, `pagerank2`, `toolbarPageRank`, `pageRankNS` / "nearest seed").
- Blends with Chrome engagement (`chromeInTotal`).
- Includes brand-query signals (`pnav`, `pnavClicks`, `navBrandWeight`, `siteNavBrandingScore`).
- Moderated by topical focus (`siteFocusScore`, `siteRadius`).
- Historical quality consistency (`chardVariance`).

**Design rule**: a well-designed site authority signal is not a number; it's a composition. Build it from multiple independent inputs, and expose each input for debugging — otherwise, when it moves, no one will know why.

## Quality Rater Data — `golden` Documents

The field `golden` on `NlpSaftDocument`: *"Flag for indicating that the document is a gold-standard document. This can be used for putting additional weight on human-labeled documents in contrast to automatically labeled annotations."*

Combined with references to EWOK (Google's human rater platform) throughout the leak, this contradicts years of statements that quality raters "don't directly influence rankings." They can — at least for gold-standard labeled documents, which receive extra weight.

**Design rule**: human-in-the-loop is a legitimate, documented mechanism. If you build a quality system, reserve an explicit "gold-standard" class for human-labeled documents and let it override model outputs with higher weight. Don't pretend your ML pipeline is fully autonomous.

## Content Length And Originality

| Signal | What it scores | Notes |
|---|---|---|
| `OriginalContentScore` | Originality of **short** content (0–512) | Applied specifically to short docs |
| `ContentChecksum96` | Fingerprint of page content | Used for exact-duplicate detection |
| `shingleInfo` | Overlapping content chunks | Used for near-duplicate detection |
| `numTokens` | Token count | Subject to Mustang's max-cap truncation |
| `lastSignificantUpdate` | Date of last **significant** change | Distinct from byline date / trivial changes |
| `richcontentData`, `semanticDate` | Semantic freshness signals | Updating a timestamp is not enough |

**Two things to internalize**:

1. **Short content is not thin content.** Short content has its own scoring path centered on `OriginalContentScore`. A crisp 400-word page can outrank a 4,000-word page in the same query.
2. **Freshness is semantic, not chronological.** Updating the byline without changing facts is detected and does not produce a freshness lift. The system looks at what actually changed.

## The Site-Level Halo

A page's ranking potential is bounded by site-level signals above it and below it:

- **Ceiling**: `siteAuthority`, `nsr`, `chardEncoded`, `tofu`, `siteFocusScore`, brand signals, `chromeInTotal`.
- **Floor**: `exactMatchDomainDemotion`, `unauthoritativeDemotion`, `clutterScore`, `navDemotion`, `pandaDemotion`, `smallPersonalSite` flag.
- **Inheritance**: `nsrdataFromFallbackPatternKey` causes un-scored chunks to inherit the chunk mean.

A perfectly optimized page on a low-authority site cannot break through the ceiling. A low-effort page on a high-authority site can still be dragged down by the floor. The interaction is multiplicative, not additive.

## Design Checklist For Your Own Quality System

1. [ ] Quality is computed per partition (chunk), not per site.
2. [ ] Un-scored partitions have an explicit fallback policy (mean / parent / neutral) — not an implicit one.
3. [ ] Demotions are orthogonal and composable; each targets one failure mode.
4. [ ] Topical focus is measured (centroid + radius or equivalent) and off-topic drift is penalized.
5. [ ] New-entity suppression is evidence-based (diverse behavior), not pure age.
6. [ ] Consistency (variance) is a signal, not just mean quality.
7. [ ] Human review can override model output for gold-standard documents.
8. [ ] Site-level signals are composable from independently-debuggable inputs.
9. [ ] Short and long content have distinct quality models.
10. [ ] Freshness is detected semantically (content diff), not from metadata timestamps.
