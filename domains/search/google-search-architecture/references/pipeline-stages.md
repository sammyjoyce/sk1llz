# Pipeline Stages — Responsibilities & Twiddler Methods

Load this file before adding a new pipeline stage, modifying stage boundaries, implementing a twiddler, or auditing where signals live.

## The Full Stage Map (Leaked Names)

| Stage | Leaked name(s) | Responsibility | Latency class |
|---|---|---|---|
| Crawl | **Trawler** | Manages crawl scheduling, change-rate estimation, fetch politeness | Hours–days |
| Render | **HtmlrenderWebkitHeadless** | Renders JavaScript pages (was WebKit, now Headless Chrome) | Seconds |
| Dedup / canonicalize | **WebMirror** | Canonicalization, duplicate detection, cluster selection | Offline |
| Link extraction | **LinkExtractor** | Extracts anchors, anchor text, context tokens | Offline |
| Primary index | **Alexandria** | Core document index | Offline build, online read |
| Tier placement | **SegIndexer** | Assigns documents into index tiers | Offline |
| Long-term storage | **TeraGoogle** | Cold / archival document store on disk | Disk |
| Retrieval + scoring | **Mustang** (with **Ascorer**) | Primary scoring, ranking, serving. Ascorer is the main scoring pass. | ~sub-100 ms |
| Query brain | **SuperRoot** | Dispatches query to indexes, runs twiddler framework on returned results | Hard latency budget |
| Re-ranker layer | **Twiddlers** | ≥ hundreds of isolated re-rankers inside SuperRoot | Per-query |
| Universal packing | **Tangram** (formerly Tetris) | Assembles cross-corpus SERP (web + images + news + video + maps) | Per-query |
| SERP frontend | **GWS** (Google Web Server) | Receives the final payload, renders the page to the user | Per-query |
| Snippets | **SnippetBrain**, **WebChooserScorer** | Generates the result snippet | Per-query |
| Universal behavior | **Glue** | Pulls together universal results using user behavior (distinct from web NavBoost) | Per-query |
| Signal generation | **Cookbook** | Generates signals; some values computed at runtime rather than stored | Mixed |

**The key insight**: Mustang and Superroot are *separate* systems. Mustang returns an initial ranked list. Superroot then calls the twiddler framework on that list. If you think of "Google ranking" as one thing, you will put code in the wrong place.

## Index Tiers = Physical Hardware Placement

`SegIndexer` places documents into tiers by a combination of quality and expected access frequency. The leak makes clear these tiers correspond to **different physical storage media**:

| Tier | Storage | Access | Intended for |
|---|---|---|---|
| Base | Flash memory | Fast, frequent | High-quality, frequently updated, frequently served |
| Zeppelins | SSD | Moderate | Standard content |
| Landfills / TeraGoogle | HDD / long-term disk | Rare | Archive, rarely-updated, low-quality |

A page's tier is not just a scoring input — it determines the **physical latency** of retrieval. Demoted pages load slower, which alone changes how they interact with query-time budgets. When you design your own tiered index, treat tier assignment as a *caching* decision, not just a quality decision.

## Mustang + Ascorer Internals

- **Mustang** is the primary scoring, ranking, and serving system. It determines who wins top spots on a daily basis.
- **Ascorer** is the primary ranking algorithm *inside* Mustang — it runs before any downstream re-ranking.
- Ascorer is downstream of **DeepRank** / **RankBrain** (neural scoring) and upstream of the twiddler layer.
- **UDR** is the named successor to certain Mustang data structures. Mustang attachments for WebRef entities and IQL expressions are deprecated in favor of UDR.

### Document truncation
PerDocData's `numTokens` comment: *"we drop some tokens in mustang and also truncate docs at a max cap."* The exact cap was not leaked, but the behavior is confirmed. Practical design rule: **the first N tokens of a document are the document, from the scorer's perspective**. If you build a long-form document store, either split documents into sub-docs at indexing time or ensure the highest-signal passages are extracted into the first N tokens.

## The Twiddler Framework

### Two execution phases

| Phase | Runs on | Has snippets/body? | Typical cardinality |
|---|---|---|---|
| **predoc** | "thin" results (PerDocData + score only) | No | Several hundred |
| **lazy** | "fat" results with docinfo | Yes | Top prefix (tens) |

Flow:
1. SuperRoot calls the corpus backend; gets thin results.
2. **All predoc twiddlers run** over the full thin result set.
3. Framework reorders thin results by reconciled twiddler constraints.
4. SuperRoot issues an RPC to fetch docinfo for the top *prefix*.
5. **Lazy twiddlers run** on the prefix.
6. Framework tries to **pack** the response.
7. If packing fails (filters removed too many, twiddlers pushed results below the prefix boundary), the framework **fetches more docinfo and re-runs lazy twiddlers on the new prefix**.

**Design consequence**: a lazy twiddler that aggressively filters or demotes can cause cascading re-fetches and blow the latency budget. Predoc is "free" (already fetched); lazy is expensive. Pay only when you need body/snippet data.

### Isolation

The framework's core invariant: **no twiddler sees another twiddler's output**. Each twiddler receives the context (query, results, PerDocData) and emits constraints. The framework reconciles. This is how >65 twiddlers can evolve independently without turning into a coupled mess.

*If you find yourself writing "if twiddler X fired, then Y should also fire" — you are working against the framework. Split it into two twiddlers that independently look at the same underlying signal.*

### Methods — Use Semantically, Not Operationally

The design doc explicitly warns: **implement twiddler methods by semantic intent, not by mechanical effect.** A single lookup table:

| Method | Semantic meaning | Use when |
|---|---|---|
| `Boost(delta)` | "This result deserves more weight on signal X" | You have a per-result confidence score that shifts ranking smoothly |
| `BoostAboveResult(ref)` | "This result should rank above that other specific one" | You have a *relative* judgement, not an absolute score |
| `Filter()` | "This result must not appear at all" | Hard policy (DMCA, spam confirmed, safe-search) |
| `SetRelativeOrder(a, b)` | "A must come before B" | Canonicalization (original before duplicate) |
| `max_position(n)` | "This result must not appear above position n" | Demotion with a hard ceiling |
| `max_total(n)` | "At most n results from this group" | Diversity (e.g. host diversification) |
| `stride(k)` | "Enforce a spacing of k between items of this group" | Interleaving, pacing, diversification |
| `Annotating(tag)` | "Attach metadata for downstream twiddlers, Tangram, or GWS" | Cross-stage communication without affecting rank |

**Anti-pattern from the design doc**: using a large `Boost` to push a result to the second page. It works in isolation, breaks in composition. Use `Filter` (if the result shouldn't show) or `max_position` (if you want a ceiling).

### Canonical twiddlers

The 2019 leak and the 2024 leak together confirm these named twiddlers (partial list):

- `FreshnessTwiddler` — boost for freshness-intent queries
- `QualityBoost` — boost for high-quality signals
- `RealTimeBoost` — breaking news and emerging trends
- `NavBoost` — click-based re-ranker (the heavyweight; often cited as ~more influential than the rest of ranking combined)
- `WebMixer` — the web corpus re-ranker host (ran >65 twiddlers in 2018)
- `ImageHostCategorizer` — host diversification for images
- `OfficialPageTwiddler` — forces position #1 for high-confidence official entity pages
- `DMCAFilter` — removes DMCA'd results and annotates for GWS
- `EmptySnippetFilter` — removes results with no usable snippet
- `YoutubeDuplicatesRemovalTwiddler` — `SetRelativeOrder` to put originals before reuploads
- `SymptomSearchTwiddler` — flags medical-symptom queries for downstream handling
- `SocialLikesAnnotator` — attaches +1 counts via `Annotating`

## Ascorer vs. Twiddler — When To Put Logic Where

| Concern | Ascorer | Twiddler |
|---|---|---|
| Number of signals | Few, complex | Many, simple |
| Data access | Full corpus, heavy data | Top-N results only |
| Dev cycle | Long — touches indexer | Short — SuperRoot job only |
| Experiment cost | Bring up ~1,400 jobs | Bring up a few SuperRoot jobs |
| Query-specific logic | No (runs offline or per-doc) | Yes — can gate on query class |
| Cross-corpus composition | No | No — use Tangram |

**Rule of thumb from the design doc**: *"If you need huge amounts of data, Ascorer is a better choice. If you need fast experimentation, twiddler is better."*

## Tangram (Universal Packing)

Tangram — formerly codenamed Tetris — is the universal packer. It sits *after* Superroot's twiddler framework and assembles the final SERP from results across corpora (web, images, news, video, maps, knowledge panels, featured snippets, ads). It uses **Glue** (a comprehensive query log / behavior store) and NavBoost to decide which verticals to surface for a given query.

Key architectural implication: **cross-corpus composition is not a twiddler problem**. A twiddler sees one corpus. If your question is "when should the video block appear above the text block?", that is Tangram logic, not web-twiddler logic.
