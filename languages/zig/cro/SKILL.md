---
name: cro-practical-zig
description: "Practical Zig decision guide in Loris Cro's style: use Zig as a toolchain and integration layer first, keep build.zig and build.zig.zon as the source of truth, migrate C/C++ gradually, and express I/O and concurrency requirements precisely. Use when writing production Zig, designing build.zig, packaging cross-platform apps, wrapping C libraries, or evaluating async and event-loop code. Triggers: zig, build.zig, build.zig.zon, zig cc, zig translate-c, @cImport, std.Build, cross-compilation, system library, Io, asyncConcurrent, snapshot testing."
tags: zig, build-system, c-interop, async, cross-compilation, package-management, testing
---

# Cro Practical Zig

## Start Here

- Before editing `build.zig`, read the official build-system guide for the exact Zig version in use and scan that version's release notes for build API removals. Zig examples age fast; build API drift is not theoretical.
- Before binding a C library, read the official `@cImport` / `zig translate-c` section for the exact Zig version. If target triple or `-cflags` are not pinned, stop: you do not yet know what ABI you are binding.
- Before adopting `std.Io` / `io.async*` patterns from blog posts, verify whether the project tracks stable Zig or roadmap/master. Cro often writes about the direction of the language before those APIs land in a release.
- Do not load the full language reference for routine work. Load the smallest relevant build-system, C interop, or testing section first.

## Mindset

- Treat Zig first as a toolchain for making a codebase operable: build graph, packaging, cross-compilation, tests, and C interop. Rewriting everything in Zig is the last move, not the first.
- The build graph is part of the product. If a user cannot `zig build` the project reproducibly on macOS, Linux, and Windows, the code is not yet practical no matter how elegant the internals are.
- Prefer boundaries that surface intent. An allocator parameter says "this code may allocate"; an `Io` parameter says "this code performs I/O"; `asyncConcurrent` says "correctness requires overlap", not merely "parallelism would be nice".

## Before You Change Anything, Ask

- Is the pain semantic or operational?
  - If the bug is build, release, dependency, or cross-target friction, fix `build.zig`, `build.zig.zon`, or `zig cc` first.
  - If the bug is at a C boundary, fix ABI and translation discipline before rewriting the library.
- Is concurrency required for correctness or only a performance opportunity?
  - If correctness requires overlap, encode that explicitly.
  - If not, keep the code correct under blocking I/O first and let the runtime exploit concurrency when available.
- Am I copying a blog-era API into a stable codebase?
  - If yes, verify against the codebase's exact Zig version before writing any code.
- Does this task belong in the build graph or in application code?
  - If it coordinates artifacts, fixtures, tests, assets, or packaging, it probably belongs in `build.zig`.

## Decision Tree

- Need to integrate upstream C/C++?
  - If upstream has no `build.zig`, depend on it as files and keep the Zig layer as a clean adapter.
  - If the integration burden is mostly compiler flags, headers, libraries, or cross targets, use `zig cc` / `zig build` and stop there.
  - Only rewrite the dependency in Zig after the boundary itself becomes the bottleneck.
- Need C bindings?
  - Use `@cImport` for quick access to constants, typedefs, and record shapes when no custom `-cflags` or manual cleanup is needed.
  - Use `zig translate-c` when flags matter, when you need to inspect output, or when you plan to tighten pointer types and edit macro fallout.
  - If translation hits bitfields, token-pasting macros, or `goto`, keep the boundary `extern` / `opaque` and do not force fake purity.
- Need tests?
  - For pure logic, use normal `test {}` blocks with `std.testing.allocator`.
  - For tools that emit directories, websites, codegen, or other file trees, make the build graph drive integration tests and compare snapshots with `git diff`.
  - For cross-target test matrices, compile widely but only run on targets you can execute; use `skip_foreign_checks` instead of pretending host execution is free.

## Procedures That Matter

### Build Graph Discipline

- Keep dependency metadata in `build.zig.zon` and build logic in `build.zig`. That separation exists so tools can inspect and fetch the graph without executing arbitrary build code.
- Model reusable units as modules first. In Zig 0.14+, artifact constructors moved toward explicit `root_module`; if you keep stuffing configuration directly into executable or test creation, later migrations hurt more.
- Default to Zig-managed dependencies for developer builds. Link against system libraries only in an explicit packager mode; distro builders need system libs, normal users need reproducibility.
- On macOS, `zig build --watch` became usable again in 0.15.x after the file-watching rewrite. If team lore says "watch is broken on macOS", re-check that assumption before building wrapper scripts around stale pain.
- When Debug compile time on x86_64 is the bottleneck, try the default self-hosted backend first. Zig 0.15.1 reports roughly 5x faster Debug compilation than LLVM in many cases. If you hit backend-specific issues, fall back to `-fllvm` or `.use_llvm = true` for that target/configuration rather than globally.

### C Interop Without Footguns

- Pin the same `-target` and `-cflags` for translation that the final build will use. Enum width, `long` size, packing, and calling convention drift can compile cleanly and still be ABI-wrong.
- After `zig translate-c`, spend effort where it pays:
  - Replace overly permissive `[*c]T` with `*T` or `[*]T` where the boundary guarantees it.
  - Collapse `anytype` macro artifacts into concrete types only after you understand the originating macro family.
  - Use `--verbose-cimport` when `@cImport` behaves strangely; inspect cached `cimport.h` and translated output before blaming Zig.
- If you cannot explain the header search path, the macro environment, and the target ABI in one sentence, you are not ready to publish bindings.

### Async and I/O Boundaries

- Write libraries so the caller chooses the I/O implementation. This keeps blocking, evented, and future runtimes from forking your API surface.
- Separate "can run out of order" from "must progress concurrently". `io.async` expresses the first; `asyncConcurrent` expresses the second. Conflating them is how code passes tests under a generous runtime and fails under a stricter one.
- Futures are resources. If multiple futures are live, either await all results before propagating errors or install `defer future.cancel(io) catch {};` before any `try` that can escape early.
- Assume `io.async` may legally run inline when resources are scarce. If inline execution breaks correctness, you chose the wrong primitive.

### Testing Like a Toolchain Engineer

- `std.testing.allocator` is not just a convenience; it is a leak oracle. If a test allocates, default to it until the ownership story is obviously stable.
- For snapshot-heavy tools, stage snapshot paths before diffing. Cro's pattern uses `git add` plus `git diff --cached --exit-code` specifically so new files fail the test instead of slipping past as untracked output.
- Do not recurse into `zig build` from `build.zig` unless you truly have separate build roots. For normal artifact orchestration, use the build graph directly; subprocess builds forfeit graph semantics, cache visibility, and better failure reporting.

## NEVER

- NEVER start by rewriting working C/C++ code because Zig makes rewriting tempting. The fast win is usually replacing the build and integration layer; a rewrite trades operational certainty for semantic risk. Instead, prove the boundary with `zig build` or `zig cc` first.
- NEVER treat `build.zig.zon` like Cargo or npm because Zig package management does not do version resolution for you. It is seductive because the file looks familiar. The consequence is surprise dependency conflicts that are yours to solve. Instead, keep the graph small, pinned, and explicit.
- NEVER copy stable-looking `build.zig` snippets from old blog posts or gists because minor Zig releases remove deprecated build APIs aggressively. The seductive part is that the code looks official. The consequence is hard compile breaks during upgrades. Instead, verify against the exact-version build-system docs before editing.
- NEVER use `@cImport` when flags or target ABI differ across environments because the host build may "work" while cross-target binaries become subtly wrong. Instead, translate with the exact `-target` and `-cflags`, inspect the generated Zig, and then wrap it.
- NEVER shell out to `zig build` from inside `build.zig` for ordinary composition because it bypasses the build graph's dependency model. It is seductive because it feels like "just run the other thing". The consequence is worse caching, worse diagnostics, and invisible dependencies. Instead, use `b.addRunArtifact` / `b.installArtifact`; recurse only for genuinely separate workspaces or fixture roots.
- NEVER write `try future.await(io)` on the first live future when later futures still need awaiting. It is seductive because it reads like ordinary Zig error propagation. The consequence is an un-awaited future and a resource leak or programming error. Instead, capture results first or install cancellation defers.
- NEVER encode correctness-critical overlap with plain `io.async` because some `Io` implementations may legitimately run the function inline. It is seductive because it often works under thread-pool runtimes. The consequence is code that silently serializes or deadlocks under non-concurrent implementations. Instead, use the API that explicitly requires concurrency.
- NEVER default to system libraries in user-facing builds because packager constraints are not end-user constraints. It is seductive if you come from distro tooling. The consequence is unreproducible cross-platform support and fragile onboarding. Instead, make Zig-managed dependencies the default and gate system linking behind an explicit packaging option.

## Fallbacks

- If a Cro-style frontier API is not available on the project's Zig version, keep the interface shape explicit anyway: pass allocators, isolate I/O at call boundaries, and hide version-specific build churn behind small helper functions in `build.zig`.
- If `translate-c` output is ugly but correct, prefer a thin handwritten wrapper over "fixing" generated code beyond recognition. The goal is a stable ABI boundary, not aesthetic purity.
- If cross-target execution is blocked, still compile every target in CI. Practical Zig means catching build drift even when you cannot run the binary on every host.
- If the build script starts to look like application code, stop and split responsibilities: build graph here, runtime logic in normal Zig modules.

## Freedom Level

- High freedom: data structures, internal APIs, and how much Zig to introduce into an existing codebase.
- Low freedom: ABI boundaries, build graph wiring, cross-target flags, package sourcing, and future cancellation or await discipline. Small mistakes here create failures that look random later.
