---
name: fielding-rest
description: >
  Design network-based software architectures using Roy Fielding's REST constraints
  with expert-level knowledge of HATEOAS, media type design, caching semantics, and
  the misunderstood boundaries of REST. Use when designing hypermedia APIs, reviewing
  API architectures for REST constraint violations, choosing between REST and alternatives
  (gRPC, GraphQL, JSON-RPC), implementing content negotiation, versioning via media types,
  or debugging cache invalidation across CDN/proxy layers. Trigger keywords: REST API,
  HATEOAS, hypermedia, uniform interface, stateless, content negotiation, media type
  versioning, Cache-Control, ETag, RFC 9457, problem+json, link relations, API evolution.
---

# Fielding REST — Expert Decision Guide

## The One Thing Most Get Wrong

Fielding's dissertation is NOT a guide to building HTTP APIs. It describes constraints that made the *World Wide Web itself* scale across organizational boundaries. REST is only the right choice when your API crosses trust/org boundaries like the web does. Before adopting REST, ask: **"Do I control all clients?"** If yes, consider gRPC or JSON-RPC — REST's decoupling constraints carry cost you may not need.

Fielding himself: *"REST is software design on the scale of decades: every detail is intended to promote software longevity and independent evolution. Many of the constraints are directly opposed to short-term efficiency."*

## Decision Framework: Is REST Right Here?

```
Do you control all clients?
├─ YES → REST's decoupling tax is unnecessary. Use gRPC/JSON-RPC.
│         Exception: you plan to open the API publicly within 2 years.
└─ NO  → Does the API cross organizational boundaries?
         ├─ YES → REST is correct. Invest in media types and link relations.
         └─ NO  → REST is acceptable but not required.
              Consider: will intermediaries (CDNs, API gateways) need to
              understand your traffic? If yes → REST. If no → pick by team skill.
```

## Expert Constraints — What Fielding Actually Requires

Fielding's 2008 rules (frequently violated even by "REST" APIs):

1. **Descriptive effort goes to media types, not URL schemas.** If your API docs are mostly endpoint lists with URL patterns, you're building RPC with REST branding. The effort should define representation formats and link relations.

2. **No fixed resource hierarchies.** Servers must control their namespace. Clients discover URIs through hypermedia, not documentation. A hardcoded `/api/v2/users/{id}/orders` path in client code means the client and server cannot evolve independently.

3. **Entered with one bookmark.** A REST API is entered with no prior knowledge beyond the initial URI and standard media types. Every subsequent interaction is driven by server-provided links. This is what "state transfer" in REST means — navigating a state machine via hyperlinks, not "transferring resource state over the wire."

## Media Type Design — The Overlooked Core

Before designing any endpoint, ask: **"What media type will carry both the data AND the controls?"**

| Approach | When to use | Trade-off |
|---|---|---|
| `application/hal+json` | General-purpose hypermedia API | Lightweight but no forms/actions |
| `application/vnd.siren+json` | Needs embedded actions + entities | Heavier; clients need Siren parser |
| `application/vnd.yourco.resource.v1+json` | Per-resource versioned types | Precise but clients must handle negotiation |
| `application/json` + `Link` headers | Simple APIs, few relations | Loses discoverability inside the body |

**The version-in-media-type pattern** (Fielding-approved over URL versioning):
- Put version in the subtype, not as a parameter: `application/vnd.acme.order.v2+json` — parameters are often silently stripped by proxies and some frameworks fail to route on them.
- A backward-compatible addition (new optional field) does NOT require a new version. Only break the version when you remove or rename fields.
- When both v1 and v2 exist, the server inspects `Accept` and responds with the negotiated version. Return `406 Not Acceptable` if the client requests a version you've sunset.

## Caching — Where REST Pays Off (or Breaks)

Before setting cache headers, ask: **"Who are ALL the caches between my server and the client?"** (Browser, service worker, CDN edge, reverse proxy, API gateway — each has different behavior.)

**Critical gotchas practitioners learn the hard way:**

- **`Vary` header misuse kills CDN hit rates.** `Vary: Authorization` makes every user get their own cache entry — your CDN becomes a pass-through. If you must vary on auth, use `Cache-Control: private` instead and let only the browser cache.
- **`stale-while-revalidate` is NOT universally honored.** CDNs like Cloudflare support it; many reverse proxies ignore it. Test your actual cache stack before relying on it.
- **ETags + `Vary` interaction:** If you `Vary: Accept` and serve both JSON and HTML, each variant gets its own ETag. Clients switching `Accept` headers mid-session will always miss cache. Design your API to serve one format per endpoint.
- **Conditional request race window:** Between `If-None-Match` check and response generation, the resource can change. For write-heavy resources, use short `max-age` (5-30s) with ETag revalidation rather than long TTLs.
- **Strong vs. Weak ETags matter for range requests.** Weak ETags (`W/"abc"`) cannot be used with `If-Range`. If your API serves partial content (file downloads, paginated byte ranges), you must use strong ETags.

| Resource type | Recommended strategy | Why |
|---|---|---|
| User-specific data | `Cache-Control: private, max-age=0, must-revalidate` + ETag | Never let shared caches store it |
| Reference/config data | `Cache-Control: public, max-age=3600, stale-while-revalidate=86400` | Stable; background refresh OK |
| Collection endpoints | `Cache-Control: public, max-age=30` + short TTL | Additions invalidate the list |
| Immutable resources | `Cache-Control: public, max-age=31536000, immutable` | Versioned URLs (e.g., `/assets/ab3f.js`) |

## Error Responses — Use RFC 9457

NEVER invent a custom error format. Use `application/problem+json` (RFC 9457) because it gives you machine-readable `type` URIs, extensible fields, and interop with emerging tooling. The `type` field should be a resolvable URI that documents the problem — this doubles as developer docs.

```json
{
  "type": "https://api.example.com/problems/insufficient-credit",
  "title": "Not enough credit.",
  "status": 403,
  "detail": "Balance is 30, cost is 50.",
  "instance": "/account/12345/txns/abc",
  "balance": 30
}
```

**Key decisions:**
- Use `422 Unprocessable Content` for validation errors (not 400) — 400 means the HTTP request itself is malformed; 422 means the server understood the request but the content is semantically wrong.
- Return ALL validation errors in one response using an `errors` extension array with JSON Pointer `pointer` fields. Forcing clients to fix-resubmit-fix cycles is a UX failure.
- NEVER return 200 with an error body. Every cache, monitor, and proxy between you and the client treats 200 as success.

## NEVER List — Hard-Won Anti-Patterns

**NEVER use URL path versioning** (`/v1/users`) because it implies different resources rather than different representations of the same resource. CDN cache keys differ per path, so v1 and v2 caches are entirely separate even for identical data. Clients cannot use content negotiation to gracefully degrade. **Instead:** version in the `Accept`/`Content-Type` media type.

**NEVER tunnel operations through POST** (`POST /users/search`) because you lose cacheability, idempotency guarantees, and intermediary visibility. A search is a read — use `GET` with query parameters. If the query is too complex for a URL (>2000 chars), use a stored-query pattern: `POST /queries` returns a query resource, then `GET /queries/{id}/results` is cacheable.

**NEVER use GET for state-changing operations** because every spider, prefetch, browser link-preview, and monitoring probe will trigger the mutation. This has caused real production data loss.

**NEVER omit `Content-Type: application/problem+json`** on error responses because without it, clients parsing `application/json` won't know they've received an error structure — they'll try to deserialize it as the expected success type and produce cryptic failures.

**NEVER rely solely on `Last-Modified` for concurrency control** because its 1-second resolution means concurrent writes within the same second silently overwrite each other. **Instead:** use strong ETags derived from content hash or DB row version for `If-Match` on writes.

**NEVER expose database IDs as the only resource identifier** because changing your database (sharding, migration) changes every client's hardcoded references. **Instead:** use a stable, opaque identifier (UUID or slug) that survives infrastructure changes.

## Pragmatic HATEOAS — When and How Much

Full HATEOAS (client knows only the entry point) is the theoretical ideal but almost never achieved in practice. Instead, adopt **graduated hypermedia:**

1. **Minimum viable (every API):** Every response includes a `self` link. Collection responses include pagination links (`next`, `prev`, `first`, `last`).
2. **Useful middle ground:** Responses include `_links` for related resources and available actions. Client code uses link presence to enable/disable UI actions (e.g., `withdraw` link only appears when balance > 0).
3. **Full HATEOAS:** Client navigates entirely from root. Only pursue this for public APIs intended for unknown future clients.

The litmus test: **"If I change a URL, does any client break?"** If yes, you have insufficient hypermedia.

## When REST Fails — Escape Hatches

| Symptom | Root cause | Alternative |
|---|---|---|
| Client makes 8+ requests to render one view | Over-normalized resources | Add composite resource or consider GraphQL |
| Real-time push needed | REST is pull-only | Add SSE for events; keep REST for commands |
| File upload > 100MB | HTTP request/response doesn't fit | Use presigned URLs + direct-to-storage upload |
| Sub-millisecond latency required | HTTP overhead too high | Use gRPC with persistent HTTP/2 streams |
| Internal microservice-to-microservice | No org boundary crossed | Use gRPC or async messaging |
