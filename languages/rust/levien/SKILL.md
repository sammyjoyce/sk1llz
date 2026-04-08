---
name: levien-native-ui-mastery
description: "Design Rust native UI stacks in the style of Raph Levien across Xilem, Masonry, Vello, Parley, AccessKit, and Kurbo. Use when building reactive retained or hybrid UIs, GPU vector renderers, virtualized editors or lists, accessibility-sensitive widgets, or multilingual text systems where stable identity, incremental diffing, clip culling, scene encoding, smooth resize, font fallback, or compute-shader tradeoffs matter. Triggers: xilem, vello, masonry, accesskit, parley, kurbo, retained vs immediate, stable widget id, virtualized scroll, text shaping, font fallback, smooth resize."
tags: gui, native-ui, rendering, gpu, graphics, text, accessibility, xilem, vello, parley
---

# Levien Native UI Mastery⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​​​‌​‍‌‌​​‌‌​​‍‌‌‌‌‌​‌​‍​‌‌‌‌‌‌‌‍​​​​‌​‌​‍​‌​‌‌‌‌‌⁠‍⁠

This skill is for architecture, performance, and failure-mode judgment, not API lookup or one-off visual styling. If the task is just "what method do I call?", read the crate docs instead. Keep this skill loaded when the hard part is identity, incrementalization, text correctness, or GPU/UI pipeline boundaries.

## What Levien optimizes for

- Make the borrow checker boring. If the design "needs" `Rc<RefCell<_>>` everywhere, the architecture is hiding ownership rather than solving it.
- Preserve identity for anything the user can focus, resize, scroll, select, drag, or expose to assistive tech. Cursor position, splitter geometry, tab order, and accessibility nodes all live on stable identity.
- Treat UI as a pipeline of retained structures, not one magical tree. View intent, widget or accessibility state, text layout, and GPU scene data have different incremental rules.
- Optimize at the real boundary. Some problems are subtree problems, some are text-shaping problems, some are tile-raster problems, and some are compositor timing problems.
- Prefer the common denominator only when portability dominates. Levien repeatedly trades raw backend power for deployability, but only after being explicit about what features are being given up.

## Before you design anything, ask yourself

- What state must survive reordering, conditional branches, or virtualization?
- Is the expensive work proportional to changed subtrees, scene size, glyph shaping, or screen tiles?
- Can wakeups identify a subtree precisely, or am I about to broadcast "something changed" to the whole app?
- Does this feature require real focus order, IME, screen-reader structure, or text selection? If yes, retained state is mandatory somewhere.
- Is the actual bottleneck platform integration rather than framework elegance? Resize, presentation, IME, and accessibility often decide the architecture before rendering does.

## Decision tree

- If the UI is editor-like, form-heavy, accessibility-sensitive, or has virtualized collections: use retained or hybrid architecture. Pure immediate mode becomes a pile of emulation hacks.
- If the UI is a debug overlay, game HUD, or small tool where accessibility and focus semantics are irrelevant: immediate mode is acceptable, but say out loud that you are buying out of tab focus, screen-reader fidelity, and power-efficient idle behavior.
- If large child subtrees change rarely: isolate them behind immutable handles such as `Arc` or persistent collections so memoization can use pointer equality instead of deep equality.
- If rendering cost dominates and the scene is vector-heavy: encode a scene and let GPU compute handle the embarrassingly parallel work. If browser reach, weak GPUs, or backend portability dominate: keep a hybrid path instead of forcing compute everywhere.
- If text is editable or multilingual: treat locale, fallback, shaping, and line breaking as architecture, not polish.

## Heuristics practitioners learn the hard way

- Memoization only pays when equality is cheaper than rebuild. Deriving `PartialEq` on large state often moves the hot path from rendering to comparison.
- In Adapt-style composition, do not call `Arc::make_mut` eagerly. Clone the child handle, run the child, and write back only when `!Arc::ptr_eq(old, new)`, or read-only events will poison parent memoization.
- Stable identity is not "just perf." Lose it and you lose cursor position, split-pane state, expansion state, tab order, and accessibility continuity.
- Full-tree rebuild on every event is only acceptable when the tree is genuinely small. "Half-incremental" systems feel elegant until large lists and nested conditionals show up.
- Positional identity tricks are useful, but only for static structure. Never let caller-position or child-index identity stand in for semantic identity in mutable collections.
- Clip cost is driven by partial tiles. Vello-style renderers use 16x16 tiles; zero-coverage and full-coverage tiles are cheap, while partial coverage is where masking and bandwidth costs explode.
- Scrolling has two regimes. Moderate scroll can often reuse encoded scene data and change only clip or transform. Large jumps require virtualization of scene resources, or GPU residency becomes the bottleneck.
- Text cache keys are larger than text. Real desktop fallback stacks often involve 30-80 candidate fonts; locale, font features, style, and sometimes break context matter, so caching by codepoints alone is why CJK fallback and ligature-heavy editing go subtly wrong.
- Measurement and final layout must share the same approximation. Heuristic line breaking is acceptable only if the measuring pass cannot disagree with the renderer.
- Runtime shader compilation is not free. The trade can be worth it for portability, but expect roughly 10-100ms of startup cost and 1-20MB of extra runtime or binary weight unless you invest in precompilation or native-only paths.

## Anti-patterns

- NEVER key widget identity by child index in a mutable list because it feels automatic. Insertions then steal focus, cursor state, expansion state, and accessibility identity. Instead key by semantic item identity or a stable logical path.
- NEVER spread `Rc<RefCell<AppState>>` through the tree because it quiets the borrow checker fast. You pay by losing locality, diff precision, and reusable components. Instead slice state with Adapt or lens-like boundaries and let each component own a narrow state type.
- NEVER deep-derive `PartialEq` on your whole app state just to make memoization compile because it looks idiomatic. Equality becomes O(n), rebuilds become data-size dependent, and perf cliffs hide until production datasets. Instead memoize on stable handles and persistent structures.
- NEVER choose pure immediate mode for editor-class UI because the API feels refreshingly simple. Tab focus, accessibility, IME, selection, and virtualized scrolling come back as ad-hoc hacks. Instead keep a retained widget and accessibility tree underneath any immediate-feeling surface API.
- NEVER treat clipping as a late paint effect because Porter-Duff masks feel compositional. You miss early culling and convert cheap zero/full tiles into expensive partial-tile work. Instead propagate clip bounds early and classify coverage before fine rasterization.
- NEVER cache text layout by string contents alone because ASCII demos look perfect. Locale, Han unification, OpenType features, and break context change the answer; the app stays "working" but feels wrong for real users. Instead include locale, style, and shaping features in the cache key.
- NEVER reuse shaped runs across line-break decisions because it saves allocations. Soft hyphens and ligatures can split incorrectly; the classic `f` plus soft hyphen plus `f` case can literally tear a glyph across lines. Instead re-shape at candidate breaks or use a bounded heuristic that measurement and layout both honor.
- NEVER keep the low-latency present mode during live resize because games get away with it. GPU and window-manager asynchrony produces wobble and one-frame desync. Instead switch to a synchronized resize path during resize and return to the fast path afterward.

## Platform traps that matter

- On macOS Metal, smooth live resize wants `CAMetalLayer` with `presentsWithTransaction`; `MTKView`-style async presentation is prone to wobble.
- On Windows, flip-model presentation is great until live resize. A sequential or redirection-buffer path during `WM_ENTERSIZEMOVE` and `WM_EXITSIZEMOVE` is often the difference between "feels native" and "feels like a port."
- On Linux font fallback, assume metadata is messy. Browsers hardcode common CJK families for a reason; do not expect fontconfig to infer user intent perfectly.

## Fallback strategies

- If the compute renderer is unstable on a target GPU, keep the scene encoder API and swap the backend. Do not leak backend quirks upward into widget architecture.
- If incremental logic becomes impossible to reason about, stop adding caches and first make identity and state boundaries explicit. Bad identity defeats every later optimization.
- If exact text shaping is too expensive, use a bounded heuristic only when both measurement and final layout share it. The failure mode to avoid is "measured one line, rendered another."

## Using the Linebender stack well

- Xilem is the default when you want app-level declarative UI with explicit incremental structure.
- Masonry is for low-level widget or framework work where you need direct control over widget behavior.
- Vello is for large vector scenes and GPU-parallel paint, not for escaping text-input or platform-accessibility contracts.
- AccessKit is not optional if the UI should count as real desktop software.
- Parley, Fontique, and Kurbo belong in the architecture discussion early. Text, fallback, and curves are not leaf concerns.

## Final check before shipping

- Can I reorder children without losing focus, cursor, selection, or expanded state?
- Can I explain what is memoized and why equality is cheap?
- Can I point to the retained structure that backs accessibility?
- Do large lists diff sparsely and keep semantic identity under insert or delete?
- Do clip and scroll changes avoid re-encoding the whole scene?
- Does multilingual text still behave when locale changes?
- Does live resize stay locked to the window frame on macOS and Windows?
