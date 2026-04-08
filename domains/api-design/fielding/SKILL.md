---
name: fielding-rest
description: >
  Apply Roy Fielding's REST style to long-lived HTTP APIs that must survive
  independent client and server evolution. Use when deciding whether REST is
  the right constraint set, designing hypermedia and media types, choosing
  between profiles and new media types, setting cache and concurrency
  semantics, or planning deprecation without URI churn. Trigger keywords:
  REST, HATEOAS, hypermedia, media type, profile relation, content
  negotiation, ETag, If-Match, Vary, Prefer, Accept-Patch, Deprecation,
  Sunset, problem+json.
---

# Fielding REST⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌​​‌​​‍​​‌‌​​​‌‍‌‌​‌​‌‌‌‍​​​​‌‌​‌‍​​​​‌​‌​‍​​​‌​​​‌⁠‍⁠

Use this skill for public, partner, standards-like, or multi-tenant HTTP APIs that need years of independent evolution. Do not use it for one-client integrations unless the interface must stay intermediary-visible and independently evolvable.

## Mandatory Reads Before Fragile Work

- Before changing cache or negotiation behavior, read RFC 9111 on `Authorization` and `Vary`, and RFC 7240 if anyone proposes `Prefer`.
- Before designing additive extensions, read RFC 6906 on `profile`; do not mint a new media type until you know whether semantics are actually changing.
- Before lifecycle work, read RFC 9745 and RFC 8594; do not treat `Sunset` as a generic warning banner.
- Do NOT load generic REST primers for this task; they erase the exact trade-offs that matter.

## First Question: Should This Even Be REST?

Before designing anything, ask yourself:

- How many independent client codebases will exist in 2 years?
- How many server implementations or deployments must interoperate?
- What breaks if client and server can coordinate only through runtime metadata and published media types?
- Which intermediaries must understand or optimize traffic: browser caches, CDNs, reverse proxies, gateways, crawlers, link checkers?

Use REST when the answer is "many, unknown, long-lived, and intermediary-visible." If you have one client, one server, and a human coordination channel, REST's constraint tax is usually wasted. Mark Nottingham's rule of thumb is harsh but useful: in homogeneous systems with harmonized models, REST has limited utility.

## What Experts Actually Optimize For

- Independent evolution, not pretty URLs.
- Representation semantics, not endpoint catalogs.
- Intermediary correctness, not direct-client convenience.
- Long-lived defaults, not one-release ergonomics.

If most of the spec is a URI matrix, you are already drifting into RPC. Fielding's test is where the descriptive effort goes: media types, link relations, forms/templates, and state transitions, not frozen route shapes.

## Freedom Calibration

- Be rigid about HTTP semantics, validators, cache keys, and lifecycle headers. Invisible protocol mistakes create long-lived breakage.
- Be flexible about representation shape, URL aesthetics, and how much hypermedia to expose. Match the complexity of the ecosystem, not an ideological purity test.

## Decision Heuristics That Matter

### 1. Profiles are cheaper than version forks

Use a `profile` when you are adding constraints, conventions, or extensions that do not change base representation semantics. RFC 6906 is explicit: profiles add semantics around a media type; they do not replace it.

- Good fit: stricter validation rules, business conventions, optional extension members, or domain rules layered on an existing media type.
- Wrong fit: changing meaning, removing fields old clients depend on, or redefining null/array behavior. Mint a new media type or new resource behavior instead.
- Non-obvious trap: the media type label gets lost when a representation is copied out of conversational context, so RFC 6906 says to also carry the `profile` link relation in the representation or `Link` header.
- Another trap: profile URIs are identifiers first. Do not build clients that dereference them on the hot path.

### 2. Custom version headers are a cache bug generator

Custom version headers feel less disruptive than changing URLs or media types. They are a trap. Mark Nottingham's point is practical: once the response depends on that header, caches need `Vary` on it. Teams forget, and intermediaries hand v2 representations to v1 clients.

Instead:

- Use media types when the representation changes incompatibly.
- Use new resources when capabilities or lifecycle differ materially.
- Use profiles when semantics are additive.
- Treat "API version" as documentation and policy, not a request header sprayed on every call.

### 3. `Prefer` is not negotiation

RFC 7240 explicitly warns against using `Prefer` for content negotiation. If honoring a preference changes the stored response, you owe caches `Vary: Prefer`; `Vary: *` is legal and effectively kills proxy caching.

Use `Prefer` for optional behavior such as `return=minimal`, `return=representation`, or `respond-async`, not for selecting the canonical representation type.

### 4. Auth caching is subtler than `Vary: Authorization`

The naive move is `Vary: Authorization`. It feels safest and usually destroys shared-cache hit rates. RFC 9111 already says shared caches must not reuse responses to authenticated requests unless you explicitly allow it with `public`, `must-revalidate`, or `s-maxage`.

Use:

- `private` for user-specific documents.
- `public` or `s-maxage` only when authorization gates access but the entity is actually the same for every authorized user.
- Stable cache keys that vary on real representation drivers, not on the existence of credentials.

### 5. Concurrency bugs are protocol design bugs

If multiple writers can touch the same resource, make conditional requests mandatory and return `428 Precondition Required` when clients omit them. RFC 6585 exists for this lost-update problem.

- Use strong ETags with `If-Match` for writes.
- Do not rely on `Last-Modified` for hot resources; one-second resolution loses concurrent edits.
- Weak ETags are fine for cache revalidation, not for write concurrency or range coordination.
- If a PATCH flow cannot tolerate stale bases, reject unconditional PATCH and force clients to re-GET, rebase, and retry.

### 6. PATCH is only good when the patch format fits the resource

Before offering PATCH, ask yourself:

- Is partial update materially smaller than full replacement?
- Can the patch format express intent without ambiguous null/array semantics?
- Can clients discover supported patch formats?

Rules:

- Advertise support with `Accept-Patch` per RFC 5789.
- Use `application/merge-patch+json` only for object-heavy documents that do not use explicit null as meaningful data; arrays are replaced wholesale.
- Use `application/json-patch+json` when array edits, `test`, move/copy semantics, or precise conflict detection matter.
- If the patch document is usually as large as the full representation, PUT is cleaner.
- Do not assume PATCH request entity headers mutate resource metadata; RFC 5789 says those headers describe the patch document, not the resource.

### 7. POST is fine, but pay the tax correctly

Fielding is explicit: POST is not un-RESTful; abusing method semantics is. Use POST when the action is unsafe and no standard method conveys more useful semantics to intermediaries.

The expert pattern is:

- Make the stateful thing a resource.
- POST to an action target or controller resource.
- Return `303 See Other` to the canonical state resource when the important outcome is a new or changed state view.

That keeps clients on the right representation and lets POST invalidate caches on the relevant URI/Location/Content-Location path instead of leaving stale collection pages around.

### 8. Deprecation and sunset are different phases

Do not collapse them.

- `Deprecation` (RFC 9745) is a runtime hint that a resource will be or has been deprecated. It does not change semantics or behavior.
- `Sunset` (RFC 8594) is for the later stage where the resource is expected to become unresponsive.
- `Sunset` must not be earlier than `Deprecation`.
- Scope is easy to get wrong: the header is attached to one resource, but your docs may define wider scope. If you do that, define the scope explicitly on the home/start resource and link migration guidance with `rel="deprecation"`.

## NEVER Do These

- NEVER hardcode URI hierarchies into client logic because readable paths feel stable and resource-oriented; the concrete result is that namespace moves, host splits, and federated deployments become breaking changes. Instead, consume server-provided links, forms, or URI templates defined by the media type.
- NEVER invent a custom version header because it feels less disruptive than changing media types or resources; the concrete result is a hidden cache key that requires `Vary` and eventually serves the wrong variant or disables caching. Instead, change media types/resources for incompatible behavior and use profiles for additive conventions.
- NEVER use `Prefer` for negotiation because it feels like a soft feature flag; the concrete result is `Vary: Prefer` debt or `Vary: *`, which makes proxies effectively uncacheable. Instead, use `Accept` and media types for representation selection and keep `Prefer` for optional processing.
- NEVER reach for `Vary: Authorization` as your auth-cache strategy because it feels maximally safe; the concrete result is per-token cache fragmentation with no value when the entity is personalized anyway. Instead, use `private` for personalized responses or deliberately cache shared authorized content with `public`/`s-maxage`.
- NEVER create fake resources purely to avoid POST because CRUD symmetry feels pure; the concrete result is unnatural resources with no reusable meaning and extra round trips. Instead, create a resource only if its state is independently valuable; otherwise POST the action and redirect to the canonical state resource.
- NEVER treat `Deprecation` as permission to change behavior because it feels like a soft cutover; the concrete result is clients break before the sunset date and cannot trust lifecycle signals. Instead, keep behavior stable, emit `Deprecation`, link migration docs, and reserve `Sunset` for actual shutdown.

## Decision Tree For Real Designs

- One client, one server, shared release cadence:
  Use HTTP if convenient; do not force hypermedia beyond basic links, validators, and cache semantics.
- Many clients, one server, public or partner API:
  Use strong representation contracts, problem details, conditional writes, and selective hypermedia for navigation and capability discovery.
- Many clients, many servers, or standardizable ecosystem:
  Invest in registered link relations, media types/profiles, home/start documents, and runtime discoverability. This is where full REST discipline pays back.
- UI needs 6 or more round trips before a user can act:
  Stop normalizing everything. Add composite resources or server-directed workflows. REST does not require chatty screens.
- You cannot explain cache behavior with `Cache-Control`, `ETag`, `Vary`, and method semantics alone:
  You are probably sneaking RPC semantics through HTTP.

## Fallbacks When The Ideal Is Not Available

- If clients cannot consume hypermedia yet, centralize link generation server-side and emit affordances anyway for new clients; do not freeze public standards around today's client limitations.
- If CDN behavior is unreliable, optimize first for browser/private-cache correctness, then layer shared-cache policy only where you can test it end to end.
- If PATCH tooling is weak, ship PUT plus strong validators before inventing partial-update semantics you cannot make discoverable.
- If deprecation docs are only human-readable for now, that is still better than a bare header. Emit `rel="deprecation"` early and add machine-readable policy later.

## Review Checklist

- Can a client start from one URI and navigate using received affordances?
- Are incompatible changes expressed as new media types/resources rather than magic headers?
- Are cache key drivers explicit and testable?
- Are concurrent writes protected by strong validators?
- Does every partial-update format have clear discovery and conflict semantics?
- Is deprecation discoverable before shutdown, with migration docs linked at runtime?
