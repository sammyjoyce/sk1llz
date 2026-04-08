# S2 Cell ID Internals, Tokens, and Cross-Library Gotchas

Read this only when you need to: (a) parse/construct cell IDs manually,
(b) convert between tokens and IDs, (c) debug why your IDs don't match
another team's output, or (d) choose a Python/Go/Java/C++ S2 library.

## The 64-bit Cell ID Layout

```
 bit:  63  62  61  60 59  ...  2k+1 2k  2k-1 ...  1  0
       [ face ]  [ child  child  ... child ]  1   0 0 0 0
          3 bits     2 bits per level          sentinel  padding
```

- **3 face bits** (positions 63–61): 0–5 for the six cube faces.
- **2k child bits**: quadrant selection (0–3) at each of k subdivisions.
- **Trailing 1 bit**: sentinel marking "end of hierarchy information". Its
  position implicitly encodes the level: `level = (63 - trailing_zero_count) / 2`.
- **Padding zeros**: pad to 64 bits.

**Consequence**: the bit representation of a parent cell is literally a prefix
of all its descendants' bit representations, with the sentinel moved left.
This is why `parent.range_min() <= descendant.id() <= parent.range_max()`
becomes a single SQL `BETWEEN` clause.

**Parent computation is a pure bitmask**:

```python
def parent_id(cell_id, target_level):
    lsb = 1 << (2 * (30 - target_level))
    return (cell_id & -lsb) | lsb
```

No iteration, no lookups. Linear in nothing.

## Cell ID Validity Check

A cell ID is valid iff:

1. The top 3 bits (face) encode 0–5 (not 6 or 7).
2. The lowest set bit is in an **even** position (0, 2, 4, ..., 60). That is,
   `lowest_set_bit & 0x1555555555555555 != 0`.

```python
def is_valid_cell_id(cell_id):
    if (cell_id >> 61) > 5:
        return False
    lowest = cell_id & (-cell_id & 0xFFFFFFFFFFFFFFFF)
    return bool(lowest & 0x1555555555555555)
```

The S2 library reserves `cell_id == 0` and `cell_id == ~0` as *invalid* /
*sentinel* values — don't use them as "null point", use a separate flag.

## Cell IDs Are NOT Symmetric With Geographic Closeness

One of the most misunderstood properties:

> *If two cell IDs are close numerically, then their cells are close
> geographically.* ✅
>
> *If two points are close geographically, then their cell IDs are close
> numerically.* ❌

The reverse fails at cube-face boundaries and near the Hilbert curve
discontinuities. Two points 5 meters apart across a cube edge can have cell
IDs that differ in the top bits.

**Consequences:**

- Range queries (covering → `BETWEEN` ranges) are correct — they scan all
  cells in the range, including geographically-close cells that happen to have
  "far" IDs, because the covering itself includes them.
- Nearest-neighbor by `|cell_id_a - cell_id_b|` is **wrong**. Always use
  geometric distance on the underlying points.
- "Show me everything within 50 IDs of this cell" is meaningless as a spatial
  query. Use a proper covering.

## S2 Tokens: Compact String Form

Tokens are hex-encoded cell IDs with trailing zeros stripped:

```
cell_id = 0x87283472E8000000         → token = "87283472e8"
cell_id = 0x3000000000000000 (face 1) → token = "3"
cell_id = 0                          → token = "X"   (special-cased)
```

### Token Gotchas (Most Code Gets These Wrong)

1. **You cannot truncate a token to get the parent token.** Unlike geohash,
   chopping characters off an S2 token usually produces an invalid token
   because the trailing `1` sentinel bit moves. To get a parent token, convert
   to cell ID → `.parent(level)` → back to token.
2. **Tokens are case-insensitive by value but case-sensitive by string
   comparison.** If one system emits `2ef59b` and another emits `2EF59B`,
   `==` returns false while they represent the same cell. **Canonicalize to
   lowercase on ingest.**
3. **Tokens with trailing zeros are non-canonical but valid.** `2ef59b00` and
   `2ef59b` are the same cell. Canonicalize by stripping trailing `0` (but
   keep the string "X" for cell_id 0).
4. **Empty string is not a valid token.** The special "X" exists because
   empty strings are falsy in most languages. Never store `""` to mean
   "the zero cell".

### Canonical form:

```python
def canonicalize_token(token: str) -> str:
    t = token.strip().lower().rstrip("0")
    return "X" if t == "" or t == "x" else t
```

## The Corner Case at Cube Vertices

The Hilbert curve visits the cube center `(0.5, 0.5)` on each face **three
times** with different parameter values. When decoding a cell ID back to a
`LatLng`, the library must pick one canonical parameter. This is why:

- `CellId.from_lat_lng(ll).to_lat_lng()` may not round-trip to the same
  `ll` — you get the center of the containing leaf cell, not the original
  point. Expect ~1 cm error at level 30 and proportionally more at coarser
  levels.
- **Never** store a cell ID as a lossless proxy for a lat/lng. If you need
  exact coordinates, store both.

## C++ vs Java vs Python vs Go: What Differs

All implementations produce the **same cell IDs** for the same input (modulo
floating-point determinism in the boundary cases). But intermediate values
and available APIs differ:

| Feature / Library             | C++ `s2geometry` | Go `golang/geo` | Java `s2-geometry-library-java` | Python `s2sphere`    | Python `s2geometry` (SWIG) | R `s2`             |
|-------------------------------|------------------|-----------------|----------------------------------|----------------------|----------------------------|--------------------|
| S2Polygon full ops            | ✅                | ✅               | ✅                                | ⚠️ partial           | ✅                          | ✅                  |
| Boolean polygon ops (union/intersect) | ✅        | ✅               | ✅                                | ❌ missing           | ✅                          | ✅                  |
| MutableS2ShapeIndex           | ✅                | ⚠️ limited      | ⚠️ limited                       | ❌                    | ✅                          | ❌                  |
| S2ClosestEdgeQuery            | ✅                | ✅               | ⚠️                               | ❌                    | ✅                          | ✅                  |
| Exact predicates              | ✅                | ✅               | ⚠️ some                          | ❌                    | ✅                          | ✅                  |
| `ST` coord system exact match | reference        | matches C++     | **differs slightly**             | matches C++          | matches C++                | matches C++        |

**Recommendations:**

- **Python proximity/geofence work**: use `s2geometry` (the official SWIG
  bindings), not `s2sphere`. `s2sphere` is pure Python, more portable, but
  missing full polygon boolean operations, `S2ShapeIndex`, and closest-edge
  queries. If you start on `s2sphere` and later need polygon intersection,
  you will rewrite.
- **Cross-language systems**: store cell IDs as **uint64** or canonical
  hex tokens. Never store intermediate ST/UV coordinates or WKB polygons
  that went through different languages' builders — they'll drift.
- **Java interop**: be aware the ST coordinate system diverges slightly
  from C++. Final cell IDs are the same, but if you're debugging at the ST
  level, you'll go insane comparing values.

## Spheroid Convention

S2 specifies **no** spheroid model. The library treats lat/lng as angles on a
unit sphere and does not convert WGS84 → geocentric or anything else. If you
pass in WGS84 latitudes, you get WGS84-consistent distances out. If you pass
in geocentric latitudes (rare), you get geocentric distances. **Pick one
convention (WGS84) and document it**; the S2 docs compare this to "character
encoding — be consistent, S2 doesn't know or care."

Distance errors from treating WGS84 as spherical are ≤0.3% (the Earth's
flattening is 1/298). Acceptable for almost all location-based services;
unacceptable for geodetic survey, aviation, or anything measuring absolute
distance to better than meters over tens of kilometers.
