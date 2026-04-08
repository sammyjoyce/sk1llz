---
name: osmani-patterns-performance
description: |
  High-signal heuristics for JavaScript performance work in browser-delivered web apps. Use when choosing how to improve LCP, INP, CLS, startup cost, route transitions, speculative loading, or rendering containment without harming other metrics. Trigger keywords: Addy Osmani, web performance, cost of JavaScript, preload scanner, fetchpriority, prefetch, prerender, quicklink, long tasks, INP, LCP, CLS, bfcache, content-visibility, scheduler.yield, third-party facade.
---

# Osmani Patterns + Performance⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌​​‌‌​‍‌‌​‌​‌‌‌‍​​​‌‌‌‌​‍​​‌‌‌‌​‌‍​​​​‌​‌​‍​​‌‌​​​​⁠‍⁠

Use this when "make it faster" is really a sequencing problem: the browser discovers the wrong bytes too late, executes too much JavaScript before input, or speculates on work the user never needs.

## Before changing code, ask yourself
- Is the user blocked by late resource discovery, JavaScript execution, or render invalidation? Pick one bottleneck first.
- Am I about to improve one metric by stealing time from another, especially trading LCP for INP?
- Is the browser already good at this natively? If yes, will my JavaScript wrapper hide URLs from the preload scanner or force extra main-thread work?
- Is this optimization actually speculation? If the user never takes the path, what bandwidth, cache, CPU, or auth risk did I just impose on everyone else?
- If this code runs after input, can each synchronous slice stay under the long-task boundary of about 50 ms, and each visual response fit the next frame budget (16.7 ms at 60 Hz, 8.3 ms at 120 Hz)?

## Operating stance
- Optimize discoverability before optimization math. If the browser cannot see a critical image, font, or script early, compression and caching only polish a late request.
- Treat JavaScript as a debt instrument. Download cost matters, but parse, compile, execute, and hydrate costs often dominate once bytes arrive.
- Rendering work is part of responsiveness. INP is frequently lost after the event handler, inside style, layout, and paint.
- Speculation must be bounded. Prefetch and prerender are wins only when navigation likelihood, cacheability, and network slack are all high.

## Loading discipline
- This skill is intentionally self-contained. Do not fan out into extra references unless the task becomes browser-implementation-specific or the user explicitly asks for framework-specific code.

## Decision tree
- If LCP is late:
  Check discovery first. Viewport images should use real `src` or `srcset`, not `data-src`, and CSS background LCP candidates often need preload or markup changes because the preload scanner cannot see CSS-discovered URLs.
- If INP is bad after clicks or typing:
  Trace the whole interaction, not just the handler. Move analytics, secondary fetches, cache warming, and non-visual reconciliation out of the first response slice.
- If next navigation is slow:
  Prefetch or prerender only stable, anonymous, reversible pages with high follow-on probability. Skip auth, checkout, mutation endpoints, and special schemes like `mailto:`, `tel:`, `javascript:`, `market:`, and `intent:`.
- If scrolling or route switches jank:
  Reduce render surface first. `content-visibility` and smaller DOM subtrees often beat handler micro-optimizations. Then audit non-passive touch or wheel listeners, because they can force scrolling back to the main thread.
- If back-navigation feels slow or repeated shifts appear:
  Fix bfcache blockers before inventing custom state caches. The wrong lifecycle hook can destroy instant restores.

## Expert heuristics that matter
- Native lazy loading in Chromium already fetches earlier than many teams assume. After Chrome tuned the thresholds, offscreen fetch distance dropped from roughly `3000px -> 1250px` on fast connections and `4000px -> 2500px` on slower ones. If your custom `IntersectionObserver` margin is even larger, you may just be overfetching.
- `loading="lazy"` plus `fetchpriority="high"` is usually a contradiction. Offscreen lazy loading still delays discovery; high priority only applies once the browser decides the resource is close enough to fetch.
- `fetchpriority="low"` is not just for below-the-fold assets. It is useful for above-the-fold-but-not-immediate assets, such as carousel slides 2..N, because browsers may otherwise treat them as "close enough" and compete with the real LCP asset.
- `prefetch` is cheap only when the resource is cacheable and the network is genuinely idle. It runs at lowest priority, and any non-cacheable response is discarded. If you raise speculative work to high priority, it stops being cheap speculation and starts competing with user-visible work.
- Quicklink-style defaults are intentionally permissive: `threshold: 0`, `delay: 0`, `throttle: Infinity`, `limit: Infinity`, `timeout: 2000`. Leaving those defaults on a dense link surface turns curiosity into a request stampede. Constrain concurrency and total work deliberately.
- Quicklink same-origin behavior is safer than it looks. The default allowlist is `[location.hostname]`; flipping to `origins: []` enables all origins and can trigger CORS or CORB surprises in addition to wasted bandwidth.
- Splitting one big startup bundle into many `defer` files does not automatically remove input jank. In Chromium, deferred scripts are commonly evaluated in the `DOMContentLoaded` task, so the long task can survive even when the waterfall looks more fragmented.
- Native `type="module"` is not a free win either. Chromium splits compile work more helpfully, but Safari and Firefox can still evaluate each module in separate requests and tasks, so module count still matters if you ship unbundled graphs.
- Dynamic `import()` helps startup because it moves compile and evaluate work later, but it still hurts INP if the imported chunk is huge and triggered in the same frame as the interaction you are trying to protect.
- `content-visibility: auto` only pays off if you avoid DOM reads that force layout or paint on skipped subtrees. Pair it with `contain-intrinsic-size` so offscreen content does not collapse to zero height or cause scrollbar jitter.
- `content-visibility: hidden` is a better inactive-view cache than `display: none` when tab or route switches matter, because it preserves rendering state instead of rebuilding it from scratch. This pattern has produced measurable route-return wins in large SPA deployments.
- A startup strategy that lazily adds hidden DOM later can still poison steady-state responsiveness. As the session grows, selector matching, style recalculation, and layout cost grow with it. Startup wins that worsen later INP are not wins.
- Third-party widgets are often cheaper as facades. Lighthouse calls out third-party main-thread blocks over `250 ms`; if the embed is not needed until interaction or scroll, ship a preview shell first.
- `scheduler.yield()` is better than `setTimeout(0)` when available because the continuation keeps a higher effective priority than unrelated queued tasks. That matters when you want to yield for responsiveness without losing the rest of the user-visible flow.
- `unload` is a performance bug disguised as lifecycle hygiene. It is unreliable, can evict pages from bfcache, and makes return navigations slower. `pagehide` is the safer hook.

## NEVER rules
- NEVER lazy-load an LCP image because it feels "consistent". That consistency is seductive, but it delays discovery until after layout and directly taxes LCP. Instead load the actual LCP asset eagerly and, when needed, boost only that asset with `fetchpriority="high"`.
- NEVER hide critical URLs behind `data-src`, CSS-only discovery, or client-only rendering because it feels framework-clean. The preload scanner cannot speculate on what it cannot see, and your abstraction becomes late network start. Instead expose critical resources in HTML or preload them surgically.
- NEVER assume "more chunks" means fewer long tasks because the waterfall looks fragmented. In Chromium, deferred scripts still tend to evaluate together, so you can keep the same user-visible stall. Instead reduce total startup JavaScript or shift work behind dynamic `import()`.
- NEVER turn on blanket prefetch or prerender because nav prediction demos look magical. The seductive part is instant follow-on pages; the consequence is wasted bandwidth, cache pollution, auth and checkout bugs, and accidental request floods. Instead prefetch only stable, anonymous, high-probability next hops with explicit ignores and hard limits.
- NEVER set Quicklink-style `origins: []` or high-priority mode without intent. It feels like broader coverage, but it invites cross-origin waste, CORS or CORB surprises, and competition with current-navigation work. Instead keep same-origin allowlists and raise priority only for verified wins.
- NEVER fix INP only inside the event handler when the real cost is post-handler rendering. It is seductive because handler code is easy to profile, but the user still waits through style, layout, and paint. Instead trace the whole interaction and cut render work or yield between visible and non-visible phases.
- NEVER use `setTimeout(0)` as the first-choice yielding primitive when `scheduler.yield()` is available. `setTimeout` feels universal, but its continuation can fall behind unrelated queued tasks and stretch user-visible completion. Instead prefer `scheduler.yield()` for prioritized continuation, and fall back only where support is missing.
- NEVER keep `unload` listeners for analytics cleanup because they feel like the last reliable hook. They are unreliable, can disqualify pages from bfcache, and slow return navigations. Instead use `pagehide`, and if third parties keep reintroducing `unload`, consider `Permissions-Policy: unload=()`.

## Fallbacks
- If `scheduler.yield()` is unavailable, use bounded `setTimeout` or `postTask` yielding and keep each continuation comfortably below one long-task budget.
- If native lazy loading is unavailable, load a lazy-loading library conditionally. Do not ship both paths by default.
- If `content-visibility` placeholders mismatch real content, start with conservative `contain-intrinsic-size`, then let remembered sizes stabilize repeat views.
- If field CLS disagrees with lab CLS, suspect post-load shifts, iframes, or user-triggered loading paths before chasing load-only fixes.
- If third-party code is the blocker and cannot be removed, isolate it after LCP or behind a facade, and document the budget it still consumes.

## Freedom calibration
- High freedom: choosing which user path deserves budget, deciding when speculation is worth the tax, selecting facade vs full embed strategies.
- Medium freedom: chunk boundaries, dynamic-import placement, `content-visibility` scoping, and prefetch heuristics.
- Low freedom: LCP asset discovery, passive-listener correctness, lifecycle hooks affecting bfcache, and any hint or priority configuration that can duplicate or mis-prioritize fetches.

This is a philosophy-and-decision skill. Use it to choose the least harmful performance intervention, not to cargo-cult every available optimization.
