---
name: fowler
description: "Apply Martin Fowler and Thoughtworks architecture heuristics for legacy modernization, evolutionary design, refactoring strategy, monolith-vs-microservice decisions, service extraction, integration boundary reviews, and technical-debt triage. Use when reviewing or designing system boundaries, planning strangler-fig migrations, deciding whether to split a monolith, introducing feature flags, or diagnosing distributed-monolith failures. Triggers: fowler, refactoring, evolutionary architecture, technical debt, monolith first, microservices, strangler fig, branch by abstraction, service extraction, distributed monolith."
---

# Fowler⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌‌‌​​‌‍‌​​​‌‌​‌‍​​​‌​‌‌‌‍​​‌​​‌‌‌‍​​​​‌​​‌‍‌‌​‌‌‌​​⁠‍⁠

This skill is for architecture work where the hard part is not naming patterns, but choosing a migration path that preserves delivery speed while the system is changing underneath you.
It is intentionally self-contained. Do not pull in extra Fowler material unless the user explicitly asks for source-backed citations or historical wording.

## Operating Stance

- Default to a well-modularized monolith or coarse-grained "duolith" until complexity clearly outruns the microservice premium. Fowler's repeated bias is not "never distribute"; it is "do not pay distributed-systems tax before you have proof the system earns it."
- Treat architecture as a set of reversible bets plus a few expensive constraints. If you cannot name the feedback loop that will tell you the decision was wrong, the decision is too abstract.
- Prefer temporary scaffolding that is explicitly designed to die. Transitional architecture is valid only if you already know how you will remove it.
- Use precise language. Refactoring is a sequence of small behavior-preserving changes. Bug fixes, interface changes, and pure optimization may be worthwhile, but they are not automatically refactoring.
- Separate internal quality from permanence. Sacrificial architecture still needs strong modularity; "we will rewrite this later" is not permission to create mud.

## Before You Advise

### Before recommending microservices, ask yourself

- Can the team provision infrastructure in hours, not weeks?
- Can the deployment pipeline complete in no more than a couple of hours?
- Do they have both technical monitoring and business monitoring, plus rapid rollback?
- Are the candidate boundaries learned from change patterns, or guessed from org charts and normalized tables?
- Is the real pain inside one codebase, or in coordination across teams and services already?

If any of the first three answers is "no", stay monolithic for now. Fowler's point is that microservices multiply operational consequences before they deliver modularity benefits.

### Before approving a service extraction, ask yourself

- Where is the single write copy during the transition?
- Are we extracting a leaf capability, or severing a central dependency with too many live consumers?
- Can we complete the atomic migration unit end-to-end, including moving clients, data ownership, and rollback hooks?
- If performance already hurts after moving joins into memory, what do you think will happen once those joins cross the network?

If logic moves but the authoritative data does not, you have usually created a worse state than the monolith: runtime coupling, debugging drag, and write-conflict risk.

### Before approving a rewrite, ask yourself

- Is this code genuinely high-value and aligned to a clear domain, or just emotionally expensive to throw away?
- Is the team protecting an amortized asset on a spreadsheet rather than a useful architecture in production?
- Are we replacing our own architecture knowingly, or is a new team simply reacting to code they did not live through?

Fowler's sacrificial-architecture stance is sharper than most summaries: build for roughly 10x growth, but plan to rewrite before roughly 100x. The point is not the exact number. The point is to stop pretending early architecture is forever.

### Before blessing a shared standard, ask yourself

- Is this boundary inside a high-bandwidth domain, or across socially distant teams?
- Are we demanding org-wide backward compatibility where local coupling would be cheaper?
- Who will absorb the cost when the "standard" slows delivery?

Counterintuitive Fowler-style guidance: tighter coupling inside one domain can be cheaper than over-engineered loose coupling. Compatibility discipline should rise with social distance and blast radius, not with ideology.

## Decision Trees

### Monolith vs microservices

- Use a modular monolith when boundaries are still being learned, the team is still building operational muscle, or most change still lands in a few coordinated areas.
- Use coarse-grained services first when you need some deployment independence but boundaries are still fluid. Split later after the change pattern stabilizes.
- Use finer-grained services only when teams can independently release, monitor, and operate them, and when domain seams are stronger than the cross-service workflow seams.

### Legacy modernization path

- Replacing a library, framework, or subsystem inside one codebase: use Branch by Abstraction. Insert an abstraction at the client-supplier seam, improve tests through that seam, run old and new implementations side by side in test, then delete the abstraction when the swap is complete.
- Replacing a legacy flow while old and new must coexist in production: use Strangler Fig plus Transitional Architecture. Route, duplicate, or divert traffic with scaffolding that is explicitly temporary.
- Breaking direct database coupling: first try to introduce an API or repository seam. If that is impossible, replicate state deliberately and keep one write authority.

### Boundary strictness

- Inside one high-bandwidth domain: some tighter coupling can be cheaper than elaborate compatibility machinery. Shared code and even legacy database coupling may be survivable if the blast radius is contained.
- Across neighboring domains: assume coordination is slower. Prefer expand-contract changes and watch for shared-platform drag.
- Across the whole organization or external consumers: raise the bar sharply. Published schemas, versioning, deprecation strategy, and backward compatibility are mandatory; ETL-style or database-level integration is architecture debt with a wide blast radius.

### Toggle choice

- Release toggle: static, short-lived, usually one to two weeks. Add the removal work when you create it.
- Experiment toggle: dynamic per request, lives hours to weeks. Do not confuse it with canary release logic.
- Ops toggle: dynamic and fast to flip. Keep only a small number of long-lived kill switches.
- Permissioning toggle: per-request and potentially multi-year. Avoid scattering `if` statements through core domain logic.
- Validation rule: do not brute-force every flag combination. Test the changed flag both ways, test plausible interacting flags, and expose current toggle configuration so failures are diagnosable.

## Anti-Patterns

- NEVER start with tiny CRUD-shaped services because normalized tables make the split feel objective. That path is seductive because it looks measurable and "clean," but it produces anemic services, broken transaction boundaries, and a distributed system that teams cannot independently release. Instead start with macro services around rich domain behavior and split only after operations can support it.
- NEVER treat remote calls as if they were local method calls because middleware makes the syntax look similar. That illusion is seductive because it preserves the object model in diagrams, but remote calls are orders of magnitude slower and can fail for network reasons alone. Instead design coarse-grained document-style APIs and batch interactions aggressively.
- NEVER leave transitional architecture in place "for safety" because the scaffold is already paid for and nobody wants another migration. That is seductive because cleanup has no immediate product payoff, but dead scaffolding becomes permanent cognitive load and blocks future simplification. Instead define deletion criteria up front and remove the seam when the migration step is done.
- NEVER run multi-writer transitions because "last write wins" seems simpler than coordinating client cutover. That shortcut is seductive because it defers organizational work, but it creates write conflicts, unclear authority, and user-visible anomalies. Instead keep one write copy throughout the migration and move clients as part of the same atomic step.
- NEVER conflate canary releases with A/B tests because both route subsets of users. That is seductive because the mechanics overlap, but canaries should converge in minutes or hours while experiments often need days or weeks for significance, and mixing them pollutes rollback signals. Instead keep reliability rollouts and product experiments as separate controls.
- NEVER let release toggles accumulate because each one feels cheap in isolation. That is seductive because toggles unblock trunk-based development, but the carrying cost shows up later as validation complexity, dead paths, and production incidents. Instead assign expiration dates, create removal tasks immediately, and fail builds when stale flags survive too long.
- NEVER demand exhaustive testing of every feature-flag permutation because it sounds thorough and compliance-friendly. That is seductive because combinatorics looks like rigor, but most flags do not interact and the real effect is slower feedback plus false confidence. Instead test both sides of the changed flag, add targeted interaction tests, and keep live toggle state observable.
- NEVER present all technical debt as moral failure because that turns design trade-offs into slogans. That is seductive because "debt bad" sounds disciplined, but Fowler's useful distinction is prudent vs reckless and deliberate vs inadvertent. Instead explain the interest payment, the touched frequency, and whether the debt is buying time or just hiding ignorance.
- NEVER practice ivory-tower architecture because abstractions sound better on slides than in delivery. That is seductive because architects can optimize for conceptual neatness, but without feedback loops they never pay the operational cost of their decisions. Instead require measurable outcomes and keep architects accountable to delivery and runtime evidence.

## Response Shape

- Lead with the pressure that matters most: operational readiness, boundary quality, migration safety, or debt economics.
- Prefer "keep, split, strangle, abstract, or rewrite" over pattern recitation.
- When you recommend a migration, name the seam, the single write authority, the rollback move, and the deletion condition for temporary scaffolding.
- When you recommend microservices, state the premium being paid and why a monolith no longer contains the complexity cheaply enough.
- When you recommend against microservices, do not romanticize the monolith. Say what modular discipline must improve so the monolith does not become a big ball of mud.

## Calibration

- High freedom: greenfield boundary shaping, debt framing, org-technology tradeoff analysis. Use principles and trade-offs, not ritualized steps.
- Low freedom: active legacy displacement, data ownership transitions, toggle rollout strategy, and canary-vs-experiment decisions. Here the constraints are real; enforce them.

## Self-Check

- Did I identify the cheapest reversible move instead of the most fashionable architecture?
- Did I preserve delivery throughput while changing the structure?
- Did I name what must be temporary and when it will be removed?
- Did I avoid generic "clean code" advice and instead surface Fowler-specific failure modes?
