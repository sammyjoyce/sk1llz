# S2 RegionCoverer Tuning — The Expert Reference

This is the calibration reference for `S2RegionCoverer`. Read this **before**
picking `max_cells`, `min_level`, `max_level`, or `level_mod` for any production
use of S2. The defaults are reasonable for circular caps and nothing else.

## The Authoritative max_cells Table

From the S2 C++ source (`s2region_coverer.h`), measured on 100,000 random caps:

```
max_cells:        3        4     5     6     8    12    20   100   1000
median ratio:  5.33     3.32  2.73  2.34  1.98  1.66  1.42  1.11  1.01
worst case:  215518    14.41  9.72  5.26  3.91  2.75  1.92  1.20  1.02
```

**Read that first number carefully**: `max_cells = 3` has a worst-case area
ratio of **215,518×**. This is not a typo. Below 4 cells, the coverer has no
combinatorial freedom to avoid catastrophically bad fits at cube-face boundaries.

**Rules that fall out of this table:**

1. **NEVER set `max_cells < 4`**. Ever. The worst case isn't "bad", it's
   "queries that should touch one shard touch 215,000 of them."
2. `max_cells = 8` is the library default because it is the inflection point:
   median 1.98×, worst 3.91×. Sweet spot for circular caps.
3. Going from 8 → 20 cuts worst case from 3.91× to 1.92×. This is the single
   best "I need tighter fit" upgrade.
4. Going from 20 → 100 only cuts median from 1.42× to 1.11×. Diminishing
   returns. Pay the query cost only if your post-filter is very expensive.
5. Going beyond 100 is almost never justified by accuracy alone.

## max_cells Is Also a Work Budget

`max_cells` does not just cap the output size — it caps **how much work the
coverer is allowed to do while searching**. Raising it makes the coverer
itself slower, not just the downstream query. If you precompute coverings
offline (e.g., for geofences), use a generous budget (50–500). If you compute
them per-request, stay tight (≤20).

## The Floor: Cells Returned Regardless of max_cells

These are non-negotiable from the algorithm:

- **Up to 6 cells** may be returned even with `max_cells=1` if the region
  intersects all six face cells (e.g., a polygon that wraps Antarctica).
- **Up to 3 cells** may be returned for a region so tiny it fits inside a
  sub-millimeter square — *if* that square happens to land on a cube vertex
  where three faces meet. Yes, this happens in the wild.
- **Arbitrary cells** may be returned if `min_level` is too high for the region
  to be covered with coarser cells. `min_level` silently overrides `max_cells`.

## The setMinLevel == setMaxLevel Trap

```python
coverer.min_level = 14
coverer.max_level = 14
coverer.max_cells = 100
covering = coverer.get_covering(rect)
# Expectation: all cells at level 14
# Reality: cells at levels 11, 12, 13, AND 14
```

`get_covering()` does **not** guarantee a fixed-level output even when you
constrain `min_level == max_level`. It will emit coarser cells whose children
are fully inside the region. If you genuinely need "give me every level-N cell
that intersects this region", use `GetSimpleCovering(region, start_point, level, out)`
in C++/Java, or post-process by calling `.children_at_level(N)` on each returned
cell. In Python `s2sphere`, use `CellUnion.denormalize(N, 1)`.

## Interior Coverings: The Subtle Bomb

```python
coverer.get_interior_covering(tiny_polygon)   # NO max_level set
```

For a region with small or zero area, an interior coverer with no `max_level`
will recursively subdivide all the way to level 30 searching for cells that fit.
This takes measurable seconds per call and produces an empty or tiny result.

**Rule**: when computing interior coverings, always set `max_level` to something
reasonable (typically 2–4 levels below your exterior covering's max_level).
Interior coverings are also legitimately empty for thin regions — your code
must handle that case or combine interior + boundary.

## Output Is Not Stable Across Versions

Quoting the S2 docs directly: *"one should not rely on the stability of the
output. In particular, the output of the covering algorithm may change across
different versions of the library."*

**Consequences:**

- Never use a covering as a canonical hash/fingerprint for a region across
  library versions or languages (C++, Go, Java, and Python implementations
  disagree).
- Never store the **exact cell list** as the "ID" of a geofence and expect
  later versions to reconstruct it byte-for-byte.
- Instead store the underlying `S2Polygon` (WKT or S2 binary) and re-compute
  the covering whenever the library updates.

## level_mod: Branching Factor Control

`level_mod` values of 1, 2, or 3 correspond to branching factors 4, 16, 64.
Setting `level_mod = 2` means only even levels (0, 2, 4, …) can appear in the
covering. This is useful when:

- You want a sharding scheme with fewer distinct levels (simpler routing).
- You want to bound the heterogeneity of cell sizes in a covering (factor of 4
  per step instead of 4× across every level).

Caveat: with `level_mod > 1`, the returned `S2CellUnion` may not be
**normalized** — groups of four siblings at the same non-aligned level won't
be merged into a parent. Call `Normalize()` if you rely on that invariant, or
be aware that containment checks still work but set equality does not.

## Choosing min_level / max_level

The official way is via the `s2metrics.h` helpers. In Python:

```python
from s2sphere import Cell, CellId

# Find level whose average edge is closest to a given distance
def level_for_edge_m(meters):
    EARTH_M = 6371000.0
    angle_rad = meters / EARTH_M
    # S2::kAvgEdge.GetClosestLevel(angle_rad) — Python s2sphere ports this
    # via CellId.level_for_edge or manual table lookup.
    for level in range(31):
        if Cell(CellId.from_face_pos_level(0, 0, level)).exact_area() ** 0.5 <= angle_rad:
            return level
    return 30
```

**Heuristic**: set `max_level` to the level matching your tightest meaningful
precision and `min_level` 4–6 levels coarser. Wider spread lets the coverer
use coarse cells for interior, fine cells at edges.

## Accuracy of Area Methods

Three methods, three trade-offs:

| Method       | Accuracy            | Cost       | When to use                |
|--------------|---------------------|------------|----------------------------|
| `AverageArea(level)` | Factor of 1.7  | ~1 ns      | Rough sharding stats       |
| `ApproxArea()`       | 3% (0.1% ≥ L5) | ~50 ns     | Default for most code      |
| `ExactArea()`        | 6 digits       | ~microsec  | Billing, compliance, audit |

`AverageArea` is a level-only lookup — it does not even look at which cell.
For `ApproxArea` and `ExactArea`, the cell shape matters because cells at the
same level vary in area by up to **~2×** (see the S2 Cell Statistics page:
at level 16, min = 11,880 m², max = 24,909 m²). Never assume "level N" means
"uniform N square meters".
