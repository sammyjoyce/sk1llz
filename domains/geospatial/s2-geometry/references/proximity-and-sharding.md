# S2 Proximity Search, Geofencing, and Sharding Patterns

Concrete patterns that have survived production. Each comes with the gotcha
that made the prior version wrong.

## Pattern 1: Radius Proximity Search

**The naive version everyone writes first** builds a cap and requests a
covering with `max_cells = 8`. It works until somebody queries a 2 km radius
near a cube face boundary and the covering balloons to 8 cells that together
enclose an area 4× the cap. Post-filter cost dominates.

**The right version** tunes `max_cells` to the *fan-out* you can tolerate in
your storage layer, not to some abstract accuracy target.

```python
from s2sphere import LatLng, Cap, Angle, RegionCoverer

EARTH_M = 6371008.8  # WGS84 mean; matches S2Earth::kRadius

def proximity_covering(lat, lng, radius_m, max_cells=16, min_level=None, max_level=None):
    center = LatLng.from_degrees(lat, lng).to_point()
    cap = Cap.from_axis_angle(center, Angle.from_radians(radius_m / EARTH_M))
    coverer = RegionCoverer()
    coverer.max_cells = max_cells
    if min_level is not None: coverer.min_level = min_level
    if max_level is not None: coverer.max_level = max_level
    return coverer.get_covering(cap)

def candidate_ranges(covering):
    """Turn a covering into contiguous [min, max] ranges for SQL BETWEEN."""
    # Always merge adjacent cell ranges before issuing queries; a naive
    # per-cell BETWEEN creates 16 index probes where 3-4 would suffice.
    ranges = sorted((c.range_min().id(), c.range_max().id()) for c in covering)
    merged = [ranges[0]]
    for lo, hi in ranges[1:]:
        if lo <= merged[-1][1] + 1:
            merged[-1] = (merged[-1][0], max(merged[-1][1], hi))
        else:
            merged.append((lo, hi))
    return merged
```

**Production tuning that matters:**

- `max_cells = 16` for typical mobile proximity (balance false positives vs
  query count). `max_cells = 8` is too loose near face boundaries.
- Set `max_level` to one level **finer** than your index storage level.
  Mismatched levels turn a range query into a full-parent scan.
- **Always merge adjacent ranges** before hitting the DB. S2 coverings
  commonly include 4 sibling cells whose ranges are consecutive — one query
  is always better than four.
- Post-filter with **chord distance** (`S1ChordAngle`), not great-circle
  Haversine: chord distance is 5–10× faster and monotonic with great-circle
  distance, so sorting/thresholding gives identical results.

## Pattern 2: Geofence Containment

The mistake: storing a covering as "the geofence" and doing point-in-covering
checks. This is fast but **the covering is an approximation** — a point just
outside the true polygon but inside a boundary cell is a false positive.

The right pattern: store **both** the polygon and its covering, use the
covering as a fast reject/accept index, and only run the expensive
`S2Polygon.contains()` on boundary cells.

```python
# Three-tier check, cheapest to most expensive:
#
#   1. Point's leaf cell ID not in covering range             → reject
#   2. Point is in an INTERIOR covering cell                   → accept (no polygon call)
#   3. Point is in an EXTERIOR-only cell (boundary)            → call S2Polygon.contains()

class Geofence:
    def __init__(self, polygon):
        self.polygon = polygon
        exterior = RegionCoverer(); exterior.max_cells = 64
        interior = RegionCoverer(); interior.max_cells = 64; interior.max_level = 18
        self.exterior = set(c.id() for c in exterior.get_covering(polygon))
        self.interior = set(c.id() for c in interior.get_interior_covering(polygon))
        # Boundary cells need the expensive containment check:
        self.boundary = self.exterior - self.interior

    def contains(self, lat, lng):
        point_cell = CellId.from_lat_lng(LatLng.from_degrees(lat, lng))
        # Walk up ancestors to find match in either set
        c = point_cell
        while c.is_valid():
            if c.id() in self.interior: return True
            if c.id() in self.boundary:
                return self.polygon.contains(point_cell.to_point())
            c = c.parent()
        return False
```

**Gotchas:**

- `get_interior_covering` on a small or thin polygon may return the empty set.
  Your code must handle that; don't assume `interior` is non-empty.
- Always set `max_level` on the interior coverer (see `covering-tuning.md`).
- `S2Polygon` requires **CCW orientation**. A clockwise loop is interpreted as
  the *complement* — the entire sphere minus a tiny hole. This silently
  corrupts geofencing: your "pizza delivery zone in downtown SF" becomes
  "everywhere except a tiny hole in SF." Always call `S2Polygon.is_valid()`
  and `s2polygon.Normalize()` on input.

## Pattern 3: Geographic Sharding

Do **not** shard on raw 64-bit cell IDs modulo shard count. The Hilbert curve
gives you geographic locality *precisely because* contiguous ID ranges are
geographically contiguous. `id % N` destroys that and gives you uniform
global load — which sounds good until every proximity query fans out to N
shards.

**Correct pattern**: partition the cell-ID space into contiguous ranges, one
per shard. Because the curve is space-filling, each shard owns a (roughly)
contiguous geographic region.

```python
def shard_boundaries(num_shards):
    # Cell IDs span roughly [0, 2^64). Contiguous ranges preserve locality.
    return [(i * (2**64)) // num_shards for i in range(num_shards)]

def shard_for_cell(cell_id, boundaries):
    import bisect
    return bisect.bisect_right(boundaries, cell_id) - 1

def shards_for_query(covering, boundaries):
    shards = set()
    for c in covering:
        lo = shard_for_cell(c.range_min().id(), boundaries)
        hi = shard_for_cell(c.range_max().id(), boundaries)
        shards.update(range(lo, hi + 1))
    return shards
```

**Real-world considerations:**

- The Hilbert curve crosses cube faces at discontinuities. Shards near face
  boundaries may get unexpectedly large geographic territories — expect load
  skew. Measure actual QPS per shard; don't assume uniform.
- For global services with uneven population, use **weighted boundaries**:
  sample real traffic, then choose boundaries so each shard sees ~equal QPS.
  The Hilbert property still preserves locality within each shard.
- For US-only services, sharding on the **8 level-2 cells** that overlap
  CONUS is a clean natural partition. Don't over-engineer.
- Replication for reliability is orthogonal and should use a second hash
  over the primary shard ID, not geography.

## Pattern 4: Level Selection Cheat Sheet

Levels the author has actually seen used in production. "Cell edge" is
approximate — same level varies up to ~2× in area.

| Use case                                    | Level   | ~Edge      | Notes                                |
|---------------------------------------------|---------|------------|--------------------------------------|
| Country-level aggregation / weather tiles   | 4–6     | 100–500 km | Stable at country granularity        |
| City / metro sharding                       | 8–10    | 10–40 km   | Typical "region server" partition    |
| Neighborhood search, ride-sharing dispatch  | 12–14   | 500 m–2 km | Index level for large mobile apps    |
| Street-level geofence, delivery zones       | 15–16   | 100–300 m  | Sweet spot for urban containment     |
| Building footprint, POI clustering          | 17–18   | 30–75 m    | Post-filter territory                |
| Room / asset-level (AR, indoor)             | 20–22   | 2–10 m     | Rarely needed; usually overkill      |
| Centimeter — sensor fusion                  | 28–30   | 1–5 cm     | Only for geodetic / survey use       |

**Rule of thumb**: pick the index level where ~90% of your queries return
between 1 and 10 cells in their covering. Too coarse → post-filter dominates.
Too fine → covering fan-out dominates. Measure both.
