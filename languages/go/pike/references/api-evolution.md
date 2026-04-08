# API Evolution⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​​‌‌​​‍‌‌​‌​‌​​‍‌​​​‌‌​‌‍‌​​​​​​‌‍​​​​‌​‌​‍​‌​​​‌‌​⁠‍⁠

Load this before touching any exported Go surface.

## Decision tree

- Need cancellation or deadlines on a stable function? Add a `FooContext` sibling and let the old function delegate with `context.Background()`.
- Need more optional behavior? Prefer a config struct or config receiver over new positional parameters.
- Need future methods? Return a concrete type, not an interface.
- Need an interface but do not want outside implementations? Seal it with an unexported method.
- Need copyable opaque values? Return a struct with unexported fields instead of forcing pointers or interfaces.

## Compatibility heuristics

- Function signatures are brittle. Struct fields are more evolvable, but only if the new field's zero value preserves the old behavior.
- Comparability is part of a value type's API. Adding a slice, map, or function field later breaks `==` and map-key callers.
- If you do not want a value type to become part of equality semantics, make that explicit early with a non-comparable field. Do this only when preserving future evolution matters more than map-key usability.
- Returning concrete types preserves your ability to add methods later. Returning interfaces hands that control to every downstream implementer.
- If you already exported an interface and need a new method, prefer a new sibling interface plus runtime type check over breaking the original interface in place.

## Error-shape rule

- Keep the mainline obvious. If the operation is "iterate until done," prefer a `Scan` plus `Err` shape over forcing error handling inside every loop body.
- Pike's own survey point is useful here: the feared `if err != nil { return err }` pattern showed up only about once every page or two in open source. When you see it every few lines, question the API shape before blaming Go.
