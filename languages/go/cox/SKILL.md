---
name: cox-tooling-excellence
description: Apply Russ Cox style to exported Go APIs, module graphs, and toolchain policy. Use when designing public packages, changing go.mod or toolchain settings, publishing releases, or debugging compatibility, MVS, retract, replace, GODEBUG, go mod tidy, go fix, or semantic import versioning issues. Triggers: backward compatibility, minimal version selection, /v2, private proxy, checksum, gorelease, toolchain, go line.
---

# Cox Tooling Excellence

Optimize for boring releases. In this style, "works for my build" is not success; success is that users get a predictable build graph, a stable API surface, and an obvious migration path when change is unavoidable.

This skill is self-contained for normal package and module work. Do not go hunting style guides or generic Go advice. Only fetch exact official Go docs when the task specifically hinges on `go`, `toolchain`, `retract`, private proxy, or `GODEBUG` semantics.

## Core stance

- Same import path means same semantics. If old and new behavior cannot coexist under one name, you do not have a minor change; you need a new name or a new major version.
- Minimal version selection is intentionally conservative: builds use the max of declared minimums, not the newest allowed version. New upstream tags should not perturb a user build until someone asks for an upgrade.
- `replace`, `exclude`, and `toolchain` are build-owner controls. They are local policy, not a published downstream contract.
- Since Go 1.21, the `go` line is an enforced minimum toolchain requirement, not a hint. Treat every `go` bump as a consumer-facing floor raise.

## Before you change anything

Before touching a public API, ask yourself:

- Am I changing type identity or assignability, or only call convenience? Function signatures and interface method sets are hard contracts; call-site compatibility alone is irrelevant.
- Is this module a library or the top-level program? Libraries publish minimums. Programs own the full build and may use `replace`, vendoring, or `GODEBUG` compatibility shims locally.
- Am I changing syntax, semantics, or build selection? Those map to different levers: new symbol, old behavior behind a flag, or module/toolchain metadata.
- Will a newer toolchain merely help maintainers, or is it truly required for consumers? If only maintainers need it, prefer `toolchain` over raising `go`.

Before reviewing a release candidate, read the target module's `go.mod`, exported package docs, and the previous tagged release or API snapshot. Do not start in the implementation: Russ Cox style treats the public contract as the source of truth.

## Decision tree for public API changes

- Need more inputs on a stable function or method? Add a sibling API (`FooContext`, `QueryContext`) or an options carrier. There is no backward-compatible way to edit a function signature.
- Need future extensibility with defaults? Prefer a config struct when zero or `nil` can preserve old behavior and duplicate-setting semantics must be impossible.
- Need behavior injection that is genuinely open-ended? Functional options are acceptable only if order and duplicate-handling rules are explicit. Otherwise you create namespace bloat and ambiguous "last one wins?" bugs.
- Need richer behavior from callers without breaking them? Define a small extension interface and probe dynamically at the use site. Do not add methods to an exported interface.
- Need room to evolve a value type? Add only fields whose zero value preserves previous behavior. If value equality is not part of the intended contract, make the type intentionally non-comparable now rather than accidentally later.
- Need incompatible semantics? First try coexistence by adding a new name. If the conceptual API itself changed, cut `/v2` and keep `/v1` boring.

## Heuristics that save real pain

- Variadic "optional arguments" are seductive because existing calls keep compiling, but wrappers, function variables, method expressions, and interface matches break immediately. The public contract is the type, not the syntax at one call site.
- Struct options and functional options are not equivalent. Struct options make duplicate settings impossible and keep the namespace tight; functional options are better only when behavior itself is extensible or construction-time effects matter.
- If an exported struct is currently comparable, adding a slice, map, or function field later breaks `==` and map-key use. If you want freedom to evolve a value type that should never be compared, add a zero-size non-comparable sentinel up front.
- Unkeyed composite literals for foreign structs are a latent compatibility bug. Go explicitly allows new exported fields to be added; keyed literals survive, positional ones do not.
- Embedding non-interface exported types in public structs buys short-term convenience at the cost of future method-conflict risk when the embedded type grows new methods.
- The `go` line and `GODEBUG` interact: a newer toolchain can preserve older defaults when the module's `go` line stays older. Use that deliberately during migrations instead of surprising users with behavior flips.
- If you routinely build and test against dependency versions newer than the minimums in `go.mod`, your published minimums become fiction. Limited upgrades will fail later in ways CI never saw, especially when you accidentally relied on a bug fix rather than a new symbol.

## Module graph rules most people learn late

- `replace` does not add a module to the graph. Without a matching `require`, it is inert. This is a common false sense of safety during local hotfixes.
- `replace` and `exclude` only apply in the main module or `go.work`. Dependencies cannot force them downstream, by design. If your "fix" depends on consumers inheriting `replace`, you have not shipped a fix.
- `retract` is the right answer for a bad release. Publish a higher version containing the `retract` directive and rationale comments. Do not delete or retag the bad version: proxies and the checksum database preserve history, and mutable tags turn mistakes into integrity failures.
- `go mod tidy` is broader than it looks: it loads packages, tools, tests in other modules, and all build tags except `ignore`. Unexpected graph churn often comes from test-only or platform-specific imports, not a real runtime dependency.
- `go mod tidy -go=...` changes more than metadata: it can enable or disable module graph pruning and lazy loading. Separate `go`-line bumps from ordinary dependency maintenance so reviewers can see the semantic change.
- The default compatibility check for `go mod tidy` targets the Go version immediately before the module's `go` line. Use `-compat` deliberately when you are preserving an older consumer floor.
- MVS gives high-fidelity builds: users get the versions some module author actually asked for, not the latest satisfying versions. Treat any workflow that silently builds against newer versions than you declared as corrupting your minimum requirements.
- `GODEBUG` compatibility switches are a migration window, not a permanent architecture. Go keeps these compatibility settings for at least 2 years, or 4 releases, so use them to stage a transition and then delete the dependency on old behavior.

## Toolchain and migration workflow

1. Snapshot the public surface with `gorelease` or an equivalent API diff. If that tooling is unavailable, compare exported symbols, signatures, comparability, and method sets against the last tag manually. Unit tests are not enough.
2. Review `go.mod` as policy. Every `go`, `toolchain`, `replace`, `exclude`, and `retract` line should have an owner and a reason.
3. Test with `GOTOOLCHAIN` at the declared `go` floor and at the current preferred toolchain. If behavior differs only because of newer defaults, decide whether to keep the older `go` line or encode the migration explicitly with `GODEBUG` or `//go:debug`.
4. Run `go mod tidy -diff` before mutating files. If the diff is surprising, prove which package, test, tool, or build tag pulled it in.
5. For language or standard-library migrations, run `go fix -diff ./...` from a clean git state first. It intentionally skips generated files; if fixes belong there, repair the generator rather than patching generated code by hand.
6. If the repo is build-tag heavy, rerun `go fix` or validation under representative `GOOS` and `GOARCH` combinations. The active build configuration limits what the tool can see.

## Private module and proxy edge cases

- If one trusted corporate proxy serves all modules, do not set `GONOPROXY`; let that proxy handle both public and private traffic and set `GONOSUMDB` or `GOPRIVATE` as needed.
- If a private proxy sits before a public fallback, a 404 or 410 on a mistyped private path causes the `go` command to fall through to the public proxy, leaking the private module path. Return a non-fallback error when path secrecy matters.
- Turning `GOSUMDB=off` stops integrity verification for newly downloaded modules unless hashes are already in `go.sum`. Use it only in tightly controlled environments, not as a convenience workaround.

## NEVER do these

- NEVER add a variadic or extra parameter to a stable function because existing calls still compile and the diff looks small. Function values, wrappers, and interface matches break. Instead add a sibling API or an options carrier.
- NEVER bump the `go` line just because CI uses a newer compiler because it feels like harmless housekeeping. Since Go 1.21 it excludes older consumers by contract. Instead keep the lowest truthful `go` line and use `toolchain` for maintainer preference.
- NEVER publish with a `replace` that you expect consumers to honor because local tests pass and the graph looks patched. Downstream builds ignore it. Instead tag the real dependency, add the right `require`, or keep the replacement strictly local to the main module.
- NEVER delete or retag a bad release because it feels cleaner than admitting a mistake. Proxies and the checksum database keep the old bits, and users can hit checksum failures or irreproducible rebuilds. Instead publish a higher version with `retract` and a reason.
- NEVER accept `go mod tidy` churn blindly because the command looks like formatting. It loads tests, tools, and nearly all tagged files, so you can accidentally widen the graph or raise minimums. Instead review `-diff` and isolate the importer before committing.
- NEVER add a non-comparable field to an exported comparable value type because slices or maps are convenient state holders. You silently break `==` and map-key users. Instead preserve comparability or intentionally forbid comparison before users depend on it.
- NEVER use unkeyed composite literals for types from another package because they are shorter. Future field additions are allowed and will break callers. Instead use keyed fields or constructors.
- NEVER extend an exported interface in place because it is the shortest patch. Every external implementer becomes uncompilable immediately. Instead define an extension interface and probe for it dynamically.

## When to escalate to `/v2`

Cut a new major version when you cannot preserve the old name and old type relationships at the same time: changed function signatures, redefined exported semantics, removed symbols, or a package that has accreted mutually incompatible responsibilities. Keep the old line stable, document the migration, and let users move incrementally.
