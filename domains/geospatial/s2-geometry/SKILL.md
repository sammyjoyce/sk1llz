---
name: s2-geometry-spatial-indexing
description: Expert guidance for Google S2 Geometry — cell ID design, region coverings, proximity search, geofencing, and sharding. Use when building location-based services, ride-sharing dispatch, delivery zones, geofences, spatial joins, geographic sharding, or any system that indexes points/regions on a sphere. Triggers on keywords s2, s2sphere, s2geometry, S2CellId, S2RegionCoverer, S2Polygon, S2Cap, S2LatLngRect, cell covering, Hilbert curve spatial index, DGGS, and comparisons to H3/geohash.
tags: spatial-indexing, geospatial, s2, s2cell, regioncoverer, hilbert-curve, geofencing, proximity-search, sharding, dggs
---

# S2 Geometry

S2 is deceptively easy to start with and punishingly sharp in the details.
Most bugs come from defaults that are tuned for one shape (circular caps) being
applied to another (thin polygons, large rects), and from assuming properties
that S2 does not actually guarantee.

## What Claude Probably Gets Wrong About S2

Before writing any S2 code, internalize these expert-only facts. Each one has
bitten real production systems.

1. **S2 cells at the same level are NOT equal area.** They vary by up to ~2× at
   every level because of the cube projection. Level 16 spans 11,880–24,909 m².
   Never say "level N cells are X m²" — say "average X, worst case 2X".

2. **`max_cells` is a work budget, not just an output cap.** Raising it makes
   the coverer itself slower. And `max_cells < 4` is catastrophic: the
   worst-case area ratio at `max_cells = 3` is **215,518×** (yes, that's the
   real published number). Never go below 4. The library default of 8 is
   tuned for circular caps and nothing else.

3. **`min_level` silently overrides `max_cells`.** If `min_level` is too high
   for the region, you get an arbitrary number of cells back. Setting
   `min_level == max_level` does NOT give a fixed-level covering — you'll get
   coarser cells whose children fit. Use `GetSimpleCovering` or post-process.

4. **RegionCoverer output is not stable across library versions.** Never use
   a covering as a canonical hash/fingerprint. Store the `S2Polygon`, not its
   covering, and recompute when the library updates.

5. **`S2Polygon` requires CCW orientation.** A clockwise loop is interpreted
   as its complement — your "delivery zone in downtown SF" becomes
   "everywhere on Earth except a tiny hole in SF." Always validate.

6. **Close points don't always have close cell IDs** (despite what most
   tutorials say). The reverse is true — close IDs mean close cells — but
   two points 5 m apart across a cube face boundary can have wildly different
   IDs. Never do nearest-neighbor by `abs(id_a - id_b)`.

7. **`s2sphere` (pure Python) is incomplete.** It lacks S2Polygon boolean ops,
   `S2ShapeIndex`, and closest-edge queries. For anything beyond point-to-cell
   and cap coverings, use the SWIG-bound `s2geometry` package instead. If you
   start on `s2sphere` and later need polygon intersection, you will rewrite.

8. **`CellId.from_lat_lng(ll).to_lat_lng() != ll`.** You get the leaf cell
   center, not the original point. Never store a cell ID as a lossless
   substitute for coordinates — store both if you need roundtrip fidelity.

## Before Writing S2 Code, Ask Yourself

- **What shape am I covering?** Caps (circles) are what the defaults assume.
  Long thin polygons (roads, rivers), large rectangles, and polygons that
  straddle cube faces all need hand-tuned `max_cells` (often 30–200).
- **What's my query level vs. my storage level?** They should match within
  ±2. A mismatch turns a range query into a parent scan.
- **Is my covering computed once (offline) or per-request?** Offline
  coverings can afford `max_cells = 100–500`. Per-request should stay ≤20.
- **Do I need interior, exterior, or both?** Geofence containment usually
  needs both — interior cells are instant-accept, boundary cells need the
  full polygon check.
- **Is this data CCW?** If it came from a GeoJSON source, check. GeoJSON
  doesn't enforce winding order; S2 does, silently and catastrophically.
- **Am I about to use S2 for hexagonal aggregation or visual heatmaps?**
  Don't. Use H3. S2's quadrilateral cells at face boundaries look ugly,
  and H3's hexagonal neighbors have uniform distances, which matters for
  visualization. S2 wins at backend indexing; H3 wins at the frontend.

## NEVER List

**NEVER set `max_cells < 4`.** The coverer has no combinatorial freedom and
worst-case area blowup is 215,518×. Even `max_cells = 4` can hit 14.4× at
cube face corners. Start at 8, move up to 20 if post-filter cost hurts.
*Instead*: use 8 for quick caps, 16–20 for polygons, 50–200 for offline
precomputed geofences.

**NEVER use S2 tokens as prefixes for parent lookup.** Unlike geohash,
chopping characters off a token produces an invalid token because the
trailing `1` sentinel bit moves. Convert to `CellId → .parent(level) →
token` instead. *Consequence of the wrong approach*: silent wrong answers
that look plausible because the bad token is often still a valid cell.

**NEVER store a covering as the canonical form of a geofence.** The
RegionCoverer algorithm's output changes across library versions and
between language implementations. Your "golden" covering will silently
diverge after a library upgrade. *Instead*: store the `S2Polygon` (WKT or
S2 binary) and recompute the covering at load time.

**NEVER compute interior coverings without setting `max_level`.** For a
tiny or thin region with no `max_level` set, the coverer recursively
subdivides to level 30 searching for contained cells. This is seconds of
CPU per call for an empty or near-empty result. *Instead*: set `max_level`
to 2–4 levels below your exterior coverer's `max_level`.

**NEVER shard on `cell_id % num_shards`.** This destroys the locality that
is the entire point of S2. Every proximity query will fan out to all shards.
*Instead*: partition the 64-bit ID space into contiguous ranges
(`boundaries[i] = i * 2^64 / num_shards`) so each shard owns a geographically
contiguous region.

**NEVER use Haversine for post-filter distance.** It's 5–10× slower than
`S1ChordAngle` and gives identical rankings and thresholds. *Instead*:
`S1ChordAngle.from_length2(dist_squared)` for comparisons, convert to meters
only for display.

**NEVER trust that a `LatLngRect` covering handles the antimeridian the
way you expect.** A rect with `west > east` in longitude is interpreted as
crossing ±180°. Rects where the bounds look "normal" but should cross the
antimeridian (e.g., Fiji) will silently cover the wrong half of the planet.
*Instead*: for antimeridian-crossing regions, build an `S2Polygon` directly
from points, which handles wrapping naturally.

## The max_cells Calibration Cheat Sheet

From the S2 C++ source, measured on 100,000 random caps:

```
max_cells:        3        4     5     6     8    12    20   100   1000
median ratio:  5.33     3.32  2.73  2.34  1.98  1.66  1.42  1.11  1.01
worst case:  215518    14.41  9.72  5.26  3.91  2.75  1.92  1.20  1.02
```

- `8` = library default, inflection point for caps.
- `16–20` = best upgrade for polygons or thin regions (worst case drops to ~2×).
- `100` = diminishing returns; only worth it when post-filter is very expensive.
- `>100` = almost never justified by accuracy; only by query engine limits.

## When You Actually Hit the Details

Load these references when the task requires their specific territory. Do NOT
load all three for a simple task.

- **Tuning a RegionCoverer, interior vs exterior coverings, `level_mod`, or
  area method selection** → **MANDATORY read** `references/covering-tuning.md`
- **Implementing proximity search, geofence containment, or geographic
  sharding** → **MANDATORY read** `references/proximity-and-sharding.md`
- **Parsing cell IDs by hand, token canonicalization, cross-language
  interop, or choosing a Python/Go/Java/R S2 library** →
  **MANDATORY read** `references/cell-id-internals.md`

Do NOT load `cell-id-internals.md` for a proximity-search task — it adds no
value. Do NOT load `covering-tuning.md` for a pure cell-ID encoding question.

## Fallback Strategies

- **Covering too loose (false positive rate too high)**: first double
  `max_cells` (8→16→20). If still bad, raise `max_level` by 1. Only then
  consider `level_mod = 1` with a tighter `[min_level, max_level]` window.
- **Covering compute too slow**: halve `max_cells`, or precompute offline and
  cache by polygon hash. Never compute coverings inside a hot request path
  for polygons you control.
- **Python `s2sphere` missing an API you need**: switch to `s2geometry` (SWIG
  bindings). It is the full C++ library. No other Python option is complete.
- **Antimeridian-crossing region covering wrong**: drop `LatLngRect`, build
  an `S2Polygon` from explicit CCW points. Polygons handle wrap naturally;
  rectangles do not.
- **Cell IDs drifting between two services**: one is almost certainly using
  `s2sphere` (which matches C++) and the other Java (which has a slightly
  different ST coordinate system in intermediate values). Final cell IDs
  should still match — if they don't, check CCW orientation of your input
  polygons first.

## When to Use Something Other Than S2

S2 is the right answer for **backend indexing, spatial joins, and geographic
sharding at scale**. It is the wrong answer for:

- **Hexagonal aggregation, heatmaps, visualization** — use H3. Hexagonal
  neighbors have uniform distances (critical for heatmaps), and H3's 16
  resolutions are easier to reason about than S2's 30 levels.
- **Simple systems where any database will do** — use a geohash column and
  `LIKE 'dr5ru%'`. Not elegant, but zero new dependencies.
- **Strictly equal-area statistical analysis** — use an equal-area DGGS
  (A5, ISEA4H). S2's 2× area variation at the same level breaks naive
  aggregation by cell count.
- **Geodetic survey, aviation, sub-meter absolute accuracy** — S2 treats
  lat/lng as spherical, introducing ≤0.3% error from ignoring WGS84's
  flattening. Fine for consumer apps, not for compliance or survey work.

## References

- S2 Cell Statistics (the official variation-by-level table):
  https://s2geometry.io/resources/s2cell_statistics
- S2 Developer Guide: https://s2geometry.io/devguide/
- S2RegionCoverer source (where the `max_cells` calibration table lives):
  https://github.com/google/s2geometry/blob/master/src/s2/s2region_coverer.h
- H3 vs S2 comparison from Uber: https://h3geo.org/docs/comparisons/s2/
