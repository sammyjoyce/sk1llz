---
name: hashimoto-building-block-economy
description: >-
  Decide whether a capability belongs in the opinionated mainline product, a
  reusable primitive, or a narrow extension seam. Build for composability:
  ship one robust, versioned primitive plus a reference app, then let wrappers,
  forks, plugins, and agents explore the long tail without bloating core. Use
  for platform-vs-product decisions, library extraction, headless mode, stable
  CLI/API/file-format design, plugin seams, and downstream composition.
  Triggers: building block economy, embeddable, headless core, plugin API,
  extension seam, SDK, reusable primitive, reference app, stable contract, open
  format, composable product, outsourced R&D, mainline vs forks, Hashimoto,
  Ghostty, libghostty, platform design.
---

# Hashimoto Building Block Economy

A useful recent example made the pattern obvious: Ghostty’s first-party app
grew steadily, but libghostty reached much larger downstream usage once others
could embed the terminal core. The exact figures were approximate proxies, not
precise telemetry. The durable lesson is the important part: a strong primitive
plus an opinionated reference app can out-distribute the app alone once humans
and tool-using agents can build the long tail on top.

## Agent Instructions

When this skill is active:

- Help the user decide what belongs in **core**, **mainline**, and
  **downstreams**.
- Separate **mechanism** from **policy**.
- Recommend the **narrowest stable contract** that unlocks reuse.
- Treat downstream experimentation as **signal and outsourced R&D**, not as an
  automatic roadmap vote.
- Produce a concrete recommendation using the **Response Template** at the end.
- If context is incomplete, state reasonable assumptions and proceed. Ask
  targeted questions only when they materially change the recommendation.

## Definitions

**Primitive**  
The smallest load-bearing capability others should reuse: engine, runtime,
renderer, planner, protocol, or format. It must stay correct under many
different wrappers.

**Mainline**  
The first-party product experience you actively optimize for and support.

**Reference app**  
An opinionated app built on the primitive. It proves the primitive in
production, sets defaults, and serves the mainstream workflow.

**Extension seam**  
The explicit boundary downstreams build against: library API or ABI, stable
CLI, open format, protocol, headless service, or plugin SDK.

**Public contract**  
The exact observable behavior you promise to keep compatible. Creating a seam
means accepting compatibility obligations.

**Mechanism**  
Core capability, correctness, performance, security, and invariants.

**Policy**  
Defaults, workflow, taste, local rules, and integration choices.

## Core Stance

- Treat composability as a **distribution channel**, not just an implementation
  detail.
- Keep the mainline product **opinionated, narrow, and easy to support**.
- Put the highest quality bar on the **primitive and its contract**, not on
  every downstream artifact.
- Use the ecosystem to explore the long tail faster than mainline can.
- Do not let monetization or lock-in decide the architecture. Choose the right
  boundary first; commercial strategy is a separate layer.

## Use This When

- The durable value is the engine, protocol, format, runtime, or planner more
  than the default UI.
- Feature requests cluster around local policy and workflow differences rather
  than shared correctness.
- Users already fork, patch, scrape, embed, or automate the product.
- A stable CLI, format, API, or headless mode would unlock more value than one
  more built-in feature.
- You need a principled way to say "no" to long-tail requests without killing
  useful downstream experimentation.

## Do Not Use This When

- There is one clear mainstream workflow and customization pressure is low.
- Core semantics are still moving too quickly to promise a stable contract.
- "Platform" interest is mostly internal strategy language, not concrete reuse.
- The only way to expose reuse is by leaking private state or unstable
  internals.
- The support and security burden of third-party extension would outweigh the
  leverage.

## Decision Procedure

### 1) Identify the load-bearing value

Ask:

- What must remain correct across all downstream variation?
- Is the real value the engine or just the default wrapper?
- Which requests are about mechanism versus policy?
- What invariants would core still own if ten downstreams wrapped it?

If you cannot name the invariant in one sentence, do not freeze a broad seam
yet.

### 2) Choose the product surface

Pick one default recommendation:

- **App-first**  
  Use when there is a clear mainstream workflow, low customization pressure, and
  abstraction would be premature.

- **Primitive + reference app**  
  Use when the core capability is reusable across many niche workflows and the
  default app is only one good wrapper.

- **Add a narrow seam**  
  Use when requests are mostly policy variants: different defaults, workflow,
  integration points, or local rules over the same mechanism.

- **Delay platform promises**  
  Use when the invariant is still moving. Stabilize the primitive first, then
  narrow the contract.

### 3) Pick the narrowest seam that works

Prefer the least expensive supported boundary that downstreams can automate:

- **Stable CLI + machine-readable I/O**  
  Best default for automation and cross-language composition. Lowest support
  cost.

- **Open format or protocol**  
  Best when many tools need to read, write, validate, or transform the same
  artifact.

- **Headless service boundary**  
  Best when lifecycle, multi-client access, or remote use matters. Easier to
  version operationally; adds deployment complexity.

- **Embeddable library or stable ABI**  
  Best for performance-sensitive or deep integrations. Highest compatibility
  burden.

- **Plugin API or SDK**  
  Last resort when extension must run in-process with deep hooks. Highest
  security, stability, and support burden.

Prefer out-of-process seams until deep embedding is a proven need.

### 4) Define what stays where

- **Core owns:** correctness, invariants, performance, security, compatibility,
  and capability.
- **Mainline/reference app owns:** defaults, UX taste, batteries-included
  workflow, opinionated packaging.
- **Downstreams own:** niche policy, local workflow, custom integrations,
  experimentation, and vertical specialization.

### 5) Decide what can return upstream

Pull something into mainline only when it:

- appears independently in multiple downstreams
- strengthens the primitive for everyone
- shows durable demand, not novelty
- reduces net complexity instead of importing local policy

## Contract Discipline

Assume **Hyrum’s Law**: at scale, anything observable will be depended on unless
you make the supported surface explicit.

### What often becomes accidental API surface

Downstreams will treat these as stable if you expose them long enough:

- CLI flags, stdout or stderr shape, and exit codes
- error codes, error text, and retry behavior
- config keys, default values, and environment variables
- file names, directory layout, and on-disk formats
- network endpoints, field names, ordering, and status semantics
- library signatures, ABI, threading model, memory ownership, and callback
  order
- plugin lifecycle hooks, capabilities, and sandbox assumptions
- timing, idempotency, side effects, and sequencing
- logs, when they are the only machine-usable output

If humans need prose and machines need stability, provide separate channels:
machine-readable output for automation, human-readable output for operators.

### Stability rules

- Explicitly declare the public contract. Versioning only means something if the
  supported API is defined.
- Mark everything else as **experimental**, **unstable**, or **private**.
- If a surface is unstable, make that obvious in naming, docs, or opt-in flags.
- Do not silently promote internal behavior into de facto API by telling users
  to scrape it.
- Do not edit released artifacts in place. Ship a new version.

### Versioning guidance

- Use semantic versioning only for surfaces you are actually prepared to
  support.
- Treat `0.x` as unstable. Do not market it as stable by implication.
- Once a seam is stable, breaking changes need an explicit migration path.
- Version the contract, not just the documentation around it.
- Compatibility CI, conformance tests, and golden fixtures are part of the
  product once downstreams depend on the seam.

## Design Rules

- Separate **mechanism** from **policy** relentlessly.
- Prefer **one load-bearing primitive** and **one opinionated reference app**
  over several half-reusable layers.
- Make the first seam **machine-friendly**. Downstreams should not need to
  drive the GUI.
- Say "build this on top" only when the seam is real, documented, and
  supported.
- Measure **ecosystem leverage** separately from first-party app usage.
- Keep the primitive **more stable** than the default app built on top of it.
- Widen the seam only from repeated real demand, not from platform rhetoric.

## Signals You Extracted Too Early

- You cannot state the invariant cleanly.
- Every consumer wants a different API shape.
- The easiest implementation leaks private state.
- You need many escape hatches before the first external user exists.
- The reference app itself still changes core semantics every week.

## Signals You Waited Too Long

- Downstreams scrape logs, UI output, or terminal text.
- Multiple wrappers reimplement the same unsupported behavior.
- Forks exist mainly to expose one repeated capability.
- Mainline keeps absorbing policy switches that do not strengthen the core.
- Support load is dominated by requests that should have lived outside
  mainline.

## Anti-Patterns

- Calling app internals a platform because wrappers happen to exist
- Absorbing every long-tail request into mainline
- Shipping a weak primitive and hoping ecosystem energy compensates
- Freezing a huge extension surface on day one
- Treating downstream success as a vote to upstream everything
- Hiding reusable value behind an inseparable GUI
- Using plugins where a CLI, format, or protocol would have been enough
- Letting lock-in goals distort the seam you should have exposed

## Fallback Moves

- Not ready for plugins -> ship a stable CLI, config format, or open format
  first
- Not ready to extract a library -> isolate the core behind the app so
  extraction is boring later
- Too many downstream patches -> identify the smallest repeated need and
  stabilize only that
- Unsure about the business model -> optimize for adoption, leverage, and trust
  first; do not make architecture depend on opacity

## Metrics That Matter

Track leverage separately from mainline popularity:

- embeds, wrappers, plugins, and downstream deployments
- percentage of feature requests resolved by seam instead of core growth
- compatibility breakage rate across versions
- downstream time-to-integrate
- support load caused by the seam versus eliminated by the seam
- number of independently recurring downstream patterns worth upstreaming

## Response Template

Use this structure in the final answer:

**Primitive**  
What is the reusable, load-bearing capability?

**Mainline**  
What should remain first-party and opinionated?

**Seam**  
What extension boundary should exist, if any?

**Contract**  
What exact public surface is supported? What is explicitly private or unstable?

**Invariants**  
What must core protect across all downstream variation?

**Risks**  
What could go wrong: compatibility, support load, security, weak abstraction,
premature extraction, or delayed extraction?

**Metrics**  
What would prove this structure is working?

**Not Yet**  
What should explicitly stay out of scope until demand or invariants are
clearer?

## Self-Check

- Did I name a real primitive, not just relabel app internals?
- Did I separate mechanism from policy?
- Is the proposed seam narrow, explicit, machine-usable, and testable?
- Did I define the public contract and accidental surfaces to avoid?
- Is the core quality bar higher than the downstream quality bar?
- Did I keep monetization separate from the architecture decision?
- Did I give a concrete recommendation, not just a framework recap?

