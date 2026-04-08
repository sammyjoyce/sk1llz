# Surface Area⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​​‌​‌‍‌​​‌​‌‌​‍​‌‌​‌‌‌‌‍​​‌‌​​‌‌‍​​​​‌​​‌‍​‌​‌​‌​‌⁠‍⁠

Load this before renaming packages, adding dependencies, or choosing config and option patterns.

## Package boundary heuristics

- The package name is part of every selector. If it does not improve the call site, it is the wrong package name.
- Avoid `util`, `common`, `helpers`, and `base`. Those names mean the code was grouped by leftovers instead of by responsibility.
- Avoid stutter between module path, package name, and exported identifiers.

## Dependency restraint

- Every dependency is a trust relationship, not just a convenience. Copying a tiny, obvious helper is often cheaper than taking on review, upgrade, and supply-chain cost forever.
- In Go, the checked-in module files are supposed to be the full build truth. Adding one dependency also adds its transitive review surface, checksum entries, and update pressure.
- Prefer standard-library solutions when they keep the surface obvious. A smaller dependency tree is both simpler and safer.

## Option-shape heuristics

- Start with plain arguments when there are only one or two required choices.
- Use a config struct when you need growth and sensible zero defaults.
- Use functional options only when options will genuinely proliferate or when temporary reversible state matters.
- Option structs look simple, but they become clumsy when zero values carry real meaning and callers need temporary overrides. That is the narrow case where Pike's self-referential options earn their keep.
- If temporary state matters, consider Pike's self-referential options: an option can return the inverse so callers can `defer` restoration cleanly.

## Smell test

- If the diff adds more naming ceremony than behavior, you are probably designing around hypothetical future problems.
- If the package becomes easier for tests but harder for normal callers, you optimized the wrong client.
