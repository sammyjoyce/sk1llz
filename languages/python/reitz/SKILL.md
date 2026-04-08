---
name: reitz-api-design
description: "Design Python libraries and developer-facing APIs in Kenneth Reitz's \"for humans\" style: README-first interface design, porcelain-over-mechanics layering, brutally small public surfaces, and humane naming and error semantics. Use when shaping a package's public API, simplifying an HTTP or client SDK, choosing top-level imports, designing Session or Response style objects, or reviewing a library that feels correct but hostile. Triggers: requests-style, developer experience, wrapper library, client SDK, public API, Session, Response, timeout, auth, retries, adapters, ergonomics."
---

# Reitz API Design

Use this skill for public APIs. Do not use it to justify private cleverness, protocol-purity arguments, or gratuitous abstraction inside implementation code.

## Mandatory Context

- Before changing a public API, read the README or usage examples first, then the package's public re-exports (`__init__.py` or equivalent), then the tests that lock user syntax. Reitz starts from the call site, not the implementation.
- Before touching transport or session behavior, read the wrapper's timeout, TLS, and proxy behavior first. Do not load low-level transport internals unless the task is explicitly about the escape-hatch layer.

## Before You Design, Ask Yourself

- What real irritation am I removing for myself? If the pain is second-hand, you will optimize for theory instead of relief.
- What would the README example look like if the library already existed? Write 3-5 examples first; if they need commentary, the API is not ready.
- Which 90 percent path deserves a one-glance solution, and which 10 percent path deserves an explicit lower layer?
- Which names still make sense if the transport, payload format, or auth scheme changes in a year?
- If this ships as 1.0, what syntax am I willing to support for years even after I regret the internals?

## Reitz Operating Rules

- Design top-down. Reitz's "Responsive API Design" is README-driven: write the ideal calls first, then implement toward them.
- Keep three layers when complexity is real: a porcelain API for the default path, a verbose advanced layer for configuration, and a low-level escape hatch for exact control. Requests' `requests.get(...)` / `Session` / `PreparedRequest` split is the model.
- Preserve the public spellings once adopted. Internals can be rearchitected aggressively; the user-facing call shape is the contract.
- Prefer object extension points over global knobs. Requests v1 removed global configuration, kept session creation optionless, and pushed complexity into `Session`, adapters, and prepared requests.
- Keep constructors boring. If a constructor starts attracting retries, proxies, certs, hooks, or debug flags, move that state to a session/client object or explicit adapter; long constructors turn discovery into archaeology.
- Name by user intent, not current implementation. `get()` aged well; names like `XMLHttpRequest` age badly because they freeze the wrong mental model.
- Treat computation or failure as a method, not a property. Requests moved `Response.json` from property to method because parsing is work and can throw.
- Keep only extension points with repeated, serious use. Requests deleted all hooks except `response` and moved OAuth and Kerberos out of core; dead flexibility becomes permanent compatibility debt.
- Separate decaying security data from code releases. Requests moved trusted CA material to `certifi` so trust updates would not wait on a full library release; do the same for any bundle that ages faster than your API surface.
- If a rare feature has to exist, push it downward. Kenneth's move was not "more flags"; it was "create your own `Request` or adapter when you truly need exact bytes."

## Decision Tree

| Situation | Preferred move | Why |
| --- | --- | --- |
| Happy path is awkward in examples | Redesign the public call first | README friction predicts adoption friction |
| One simple call hides real complexity | Add a layered API, not more kwargs | Keeps the top layer memorable |
| One corner case wants total control | Add an escape-hatch object or prepared form | Avoid contaminating the main path |
| Need pluggable transport or auth behavior | Use adapter or auth objects | Objects compose better than boolean matrices |
| A "clean" refactor breaks public syntax | Keep syntax, refactor underneath | Reitz treats API stability as a social contract |
| Naming debate is stuck | Pick the name that still fits if implementation changes | Names shape future design |
| Users need special TLS, proxy, or env behavior | Prefer session-scoped or request-scoped config | Global defaults create non-local bugs |

## Freedom Calibration

- High freedom: naming, public object boundaries, top-level exports, error wording, README examples. Optimize for elegance and memorability.
- Low freedom: timeout semantics, TLS verification, proxy DNS behavior, streaming lifecycle, thread or shared-session behavior. Here the human interface must be explicit because a wrong default creates security or latency bugs.

## NEVERs

- NEVER add a top-level boolean for a rare edge case because the tiny patch is seductive and feels "simpler." It teaches every user about an expert-only problem, multiplies combinations, and turns docs and tests into a truth table. Instead add a lower-layer object, adapter, or prepared-request escape hatch.
- NEVER expose work that can parse, block, or fail as a property because property syntax falsely signals cheap field access. It is seductive because `obj.json` looks elegant, but it hides latency and exceptions inside attribute access. Instead use a verb or method for fallible computation.
- NEVER ship extension points "just in case" because unused flexibility calcifies faster than real APIs. It is seductive because hooks feel future-proof, but dead hooks become compatibility tax and mental clutter. Instead keep only hooks with repeated real use and eject niche integrations into companion packages.
- NEVER solve advanced configuration with global defaults because global state feels humane in small demos and becomes hostile in real systems. The consequence is action at a distance: one request, test, or import silently changes another. Instead scope behavior to a request, session/client, or explicit policy object.
- NEVER bundle fast-aging trust or security metadata inside the core package because shipping one artifact feels tidy. The consequence is stale root stores, forced upgrades, and emergency releases for data churn instead of code churn. Instead externalize decaying security material so it can update independently, as Requests did with `certifi`.
- NEVER treat `stream=True` as a free memory optimization because the seductive part is one flag and lower peak RAM. The concrete consequence is pool starvation: the connection is not reusable until the body is fully consumed or closed. Instead stream inside a `with` block and either exhaust or close every response.
- NEVER assume a single timeout number is a wall-clock SLA because the seductive part is "just use `timeout=5`." In Requests semantics that number applies to both connect and read, read timeout is idle time between bytes, and total elapsed time can exceed it across multiple IP attempts. Instead expose separate connect and read policy or at least document tuple semantics; `3.05` and `27` are the canonical Requests example for a reason.
- NEVER share a mutable Session-like object across threads unless the contract explicitly promises safety because pooling reuse is seductive. The consequence is cookie, cache, and redirect state leaking across callers and heisenbugs under load; Requests issue discussions have long recommended one session per thread. Instead treat session state as thread-affine or build immutable per-request config on top.
- NEVER use `verify=False` as a casual convenience because the seductive part is unblocking a dev environment in one keystroke. The concrete consequence is security drift, and in `requests<2.32.0` a Session could continue skipping verification for later requests to the same origin unless the pool was closed. Instead prefer proper CA bundles; if you must bypass locally, isolate it to a fresh session and close it immediately.
- NEVER use `socks5` when you actually need proxy-side DNS because the scheme looks like the obvious SOCKS choice. The consequence is local DNS resolution and privacy or routing surprises. Instead use `socks5h` when the proxy must resolve hostnames.
- NEVER reopen or duplicate a streaming line iterator because it feels harmless to call `iter_lines()` twice. The consequence is dropped data: Requests documents `iter_lines()` as not reentrant safe. Instead create one iterator and pass it around.
- NEVER open upload files in text mode because it looks convenient and often works in trivial tests. The consequence is broken `Content-Length` and encoding-dependent bugs once bytes matter. Instead open upload bodies in binary mode.

## Edge Cases Practitioners Forget

- Prepared-request flows bypass environment behavior unless you merge it back in. If you build a `PreparedRequest` or escape-hatch layer, document how proxy, CA bundle, and other environment settings are reintroduced.
- Small streaming helpers need lifecycle semantics. If your API returns an iterator, say who owns closing the socket or file and when connection reuse resumes.
- Convenience top-level functions should be thin, not magical. Re-export the 2-5 verbs users think in; force uncommon behavior through a client or session object rather than adding cute aliases everywhere.
- When a naming debate sounds philosophical, test it against future verbs. `user.authenticate()` and `human.authenticate()` imply different systems; the noun constrains every verb that follows.

## Done When

- A new user can guess the happy-path call from the README example without cross-referencing docs.
- The public surface is smaller after the change, or more explicit at the advanced layer, never both larger and blurrier.
- Rare power-user needs have an escape hatch that does not leak into the beginner path.
- Timeout, TLS, proxy, and streaming behavior are explicit enough that operators will not learn the semantics only in production.
