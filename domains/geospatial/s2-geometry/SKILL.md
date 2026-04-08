---
name: s2-geometry-spatial-indexing
description: >
  Use when designing or debugging S2-based spatial indexes, coverings, term
  indexes, geofences, proximity search, or cell-id/token interoperability.
  Helps choose max_cells/min_level/max_level/level_mod, avoid covering
  explosions and false-negative query bugs, decide when to use interior
  coverings or mixed-level indexes, and debug antimeridian/degenerate
  geometry/token issues. Trigger keywords: s2, s2cellid, s2regioncoverer,
  s2regiontermindexer, interior covering, cell union, token, geofence,
  proximity, sharding, antimeridian.
---

# S2 Spatial Indexing

This skill is for the parts of S2 that fail only after you ship: approximation
budgets, index/query compatibility, seam-heavy workloads, and geometry that
degrades under simplification.

## Load only what matters
- Before touching `max_cells`, `min_level`, `max_level`, or `level_mod`, READ `references/covering-tuning.md`.
- Before designing proximity search, geofence acceptance tiers, or shard layout, READ `references/proximity-and-sharding.md`.
- Before debugging tokens, parent math, signedness, or cross-library drift, READ `references/cell-id-internals.md`.
- Do NOT load `references/cell-id-internals.md` when the task is only coverer tuning.
- Do NOT load `references/proximity-and-sharding.md` when the task is only token parsing or parent derivation.

## Before you choose anything, ask yourself
- Is the real budget query fanout, post-filter CPU, index size, or false negatives? S2 only lets you optimize one pressure at a time.
- Are you indexing points, regions, or both? `S2RegionTermIndexer` has a different winning configuration for points-only corpora.
- Is the geometry authoritative, or is the covering authoritative? If the covering is the only stored artifact, approximation error became product behavior.
- Is the query mostly caps and rects, or long narrow polylines and thin polygons? The "good" covering strategy changes.
- Will index-time and query-time code share `min_level`, `max_level`, and `level_mod` exactly? If not, stop: that is a correctness bug, not tuning.
- Are you near cube-face seams, the antimeridian, or degenerate geometry? These are where synthetic benchmarks lie.

## Operating rules experts actually use
- Treat `max_cells` as a search budget, not just an output-size cap. Raising it makes cover generation slower before it makes queries better.
- Never choose `max_cells < 4`. Official S2 data puts the worst-case area ratio at `215,518x` for `3`; below `4` the coverer has too little freedom to avoid pathological fits.
- For request-path coverings, start around `8-20`. `8` is the default inflection point; `20` is the last big accuracy win before diminishing returns.
- `min_level` is not a harmless precision floor. If it is too high, exterior coverings can explode in cell count even when `max_cells` is small.
- Interior coverings need an explicit `max_level`. Without it, thin or tiny regions can recurse to level 30 searching for contained cells and still return empty.
- Same-level cells are not uniform buckets. Official level-16 area ranges are about `11,880-24,910 m^2`; never bill, shard, or capacity-plan from "level N means fixed area."
- Covering output is not stable across library versions. Never use an exact returned cell list as a persistent region fingerprint.
- If you truly have no scale prior for `S2RegionTermIndexer`, the official defaults (`min_level=4`, `max_level=16`) are meant for query regions from about `100 m` to `3000 km`. Treat that as a safe baseline, not as a claim that your workload fits it.
- `optimize_for_space` is not free compression. Official comments say it typically cuts index terms by about `1.3x` and can approach `2x` as `max_cells` grows, while query terms grow by the same factor. It is a storage-vs-serving trade, not a pure win.

## Decision tree
1. If the corpus is points only:
- Use `S2RegionTermIndexer`.
- Set `index_contains_points_only = true`; official docs say this usually cuts query terms by about `2x`.
- Keep `min_level`, `max_level`, and `level_mod` identical between indexing and querying.
- Vary `max_cells` per workload if needed: larger at ingest for tighter index terms, smaller at query time for faster serving.

2. If you need exact containment or exact distance:
- Store original geometry as well as the covering.
- Use the covering only as a prefilter or fast-accept tier.
- For polygon containment, maintain exterior plus bounded interior coverings; boundary cells still need exact polygon checks, and interior coverings may be empty for thin shapes.

3. If the region is long and narrow:
- Prefer ordinary `GetCovering()` unless you are deliberately doing recursive subdivision.
- `GetSimpleCovering()` / `FloodFill()` only win for seeded, same-level flood fills over narrow regions; the official header warns they are often slower than regular coverings for caps and polygons.

4. If you are choosing sharding:
- Shard contiguous `S2CellId` ranges, not `cell_id % N`.
- Range partitions preserve locality; modulo destroys it and turns every neighborhood query into global shard fanout.
- Expect skew near face seams and around real population centers. Move to traffic-weighted boundaries once real QPS exists.

5. If you are debugging weird misses:
- First confirm index and query sides use the same `min_level`, `max_level`, and `level_mod`.
- Then canonicalize the covering with the query-side options.
- Only after that inspect geometry validity, token canonicalization, and antimeridian behavior.

## Non-obvious procedures
### Tuning a new index
1. Pick the smallest meaningful query scale and set `max_level` from that edge length, not from a memorized level table.
2. Set `min_level` about `4-6` levels coarser so the coverer can use coarse interior cells and fine boundary cells.
3. Benchmark `max_cells` at `8`, `20`, and one workload-specific higher value. If `20` does not materially reduce post-filter cost, stop there.
4. Merge adjacent `range_min()/range_max()` intervals before hitting storage; sibling cells frequently collapse into far fewer index probes.
5. Re-run the benchmark on seam-heavy inputs: antimeridian, polar cases, and shapes spanning multiple faces.

### Designing fast containment
1. Compute an exterior covering for completeness.
2. Compute an interior covering with an explicit `max_level` for fast accepts.
3. Treat `interior == empty` as a normal outcome for thin regions.
4. Route boundary hits to the exact predicate (`contains`, exact distance, or point-in-region) before pagination or ranking.

### Handling interoperability
1. Persist `uint64` cell IDs or canonical lowercase tokens.
2. Derive parents from `S2CellId.parent(level)`, never from token truncation.
3. If a datastore is signed, verify sort and range semantics on raw `uint64` bit patterns before using lexical or signed comparisons.
4. When libraries disagree, compare final cell IDs, not intermediate ST/UV coordinates or library-specific polygon builders.

## Anti-patterns
- NEVER force a fixed-level worldview onto `GetCovering()` because `set_fixed_level()` looks like "give me only level N cells." The seductive part is the API name. The consequence is mixed-level output or bad fanout assumptions. Instead use `GetSimpleCovering()` or denormalize children explicitly when you truly need every intersecting level-N cell.
- NEVER raise `min_level` to "keep precision high" because it feels like the cleanest way to avoid coarse cells. The consequence is cell explosion that ignores `max_cells` on exterior coverings. Instead cap precision with `max_level` and leave room for mixed-level coverings.
- NEVER use an interior covering without `max_level` because "contained cells only" sounds safer than exterior approximation. The consequence is deep recursion on small or zero-area shapes, then empty results after expensive work. Instead bound `max_level` and pair interior coverage with exterior or exact boundary checks.
- NEVER change `level_mod`, `min_level`, or `max_level` on only one side of an `S2RegionTermIndexer` deployment because each knob feels like an independent tuning lever. The consequence is false negatives, not just different performance. Instead lock those three values as schema and only vary `max_cells`.
- NEVER shard by `cell_id % N` because it gives beautifully even write distribution in synthetic tests. The consequence is that every geographically local query fans out across most shards. Instead shard contiguous ID ranges and rebalance with workload-weighted boundaries.
- NEVER treat an S2 covering as the geofence itself because it makes point tests look O(1). The consequence is silent false positives on boundary cells and product bugs that only appear near edges. Instead store authoritative geometry and use coverings only for reject/accept tiers.
- NEVER truncate tokens to compute parents because tokens look geohash-like. The consequence is invalid or wrong ancestors, broken cache keys, and holes in shard membership. Instead convert token -> `S2CellId` -> parent -> token.
- NEVER feed potentially degenerate loops into legacy `S2Polygon` / `S2Polyline` because they work on clean demo data. The consequence is intermittent importer failures or topology changes after simplification. Instead use the lax shape path and `S2Builder` when geometry can collapse, self-touch, or mix dimensions.

## Edge cases that change the answer
- Points on cell edges are assigned to one containing leaf cell because `S2CellId(point)` behaves as a closed-set choice. If you need every point to belong to exactly one polygon in a tiling, use the semi-open polygon model, not ad-hoc tie-breaking.
- Antimeridian and polar shapes are where area ratios and range counts look worst. Always include seam-heavy cases in benchmarks before locking levels.
- Degeneracies are not just validity annoyances: in S2 they affect boundary semantics and distance results. If simplification must preserve distance guarantees, use `S2Builder` with snapping; the official guarantee is max edge deviation at most about `1.1x` the snap radius.
- If you enable `level_mod > 1`, the returned `S2CellUnion` may not be normalized. Normalize before relying on set equality or dedup logic.

## Freedom calibration
- Use low freedom for schema, persistence, and query compatibility rules. `min_level`, `max_level`, `level_mod`, token canonicalization, and signed/unsigned handling are contract choices.
- Use medium freedom for `max_cells` and level spread. Those are workload knobs and should be justified with candidate counts, range counts, and p99 latency.
- Use high freedom only for higher-level strategy: whether to pre-materialize mixed levels, keep leaf-only storage, or add separate interior fast-accept indexes.

## Fallbacks when the first plan fails
- If coverings are too loose at `8`, test `20` before inventing a new index design.
- If `20` is still too loose, add a stronger exact post-filter before exploding index cardinality.
- If shard hotspots cluster near seams or cities, keep the contiguous-range strategy and move to weighted boundaries; do not abandon locality-preserving partitions.
- If cross-language results diverge, serialize canonical tokens or `uint64` IDs and re-run from authoritative geometry rather than diffing cover lists.
- If thin polygons yield empty interiors, treat that as expected and fall back to exterior plus exact boundary evaluation.
