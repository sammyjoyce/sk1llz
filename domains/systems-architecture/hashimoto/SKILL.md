---
name: hashimoto-building-block-economy
description: >-
  Design software for the building-block economy: ship a robust primitive, keep
  the mainline app opinionated, and let forks, agents, plugins, and downstream
  wrappers explore the long tail. Use when deciding whether a feature belongs in
  core or in an extension seam, whether to split out a library, headless mode,
  stable CLI, or API, or how to structure a product so downstream composition
  creates adoption and R&D leverage. Triggers: building block economy,
  embeddable, headless core, plugin API, extension seam, SDK, reusable
  primitive, reference app, fork-friendly, stable contract, open format,
  composable product, agentic factory, outsourced R&D, mainline vs forks.
---

# Hashimoto Building Block Economy

Mitchell Hashimoto's April 2026 observation is not abstract: Ghostty reached
roughly one million daily macOS update checks in 18 months, while libghostty
reached multiple millions of daily users in about two months once downstreams
began composing it. The lesson is not that polished apps are dead. It is that a
high-quality primitive plus a reference app can out-distribute the app once
others can build the long tail on top.

## Operating Stance

- Treat composability as a distribution channel, not just an implementation
  detail. Agents and humans both prefer to assemble proven parts when the seams
  are obvious.
- Raise the quality bar on the primitive, not on every downstream artifact. In
  this model the core must be robust, documented, versioned, and boring under
  load.
- Keep the mainline opinionated and stable. Let extensions, embeddings, forks,
  and wrappers absorb niche demand that would otherwise turn core into a junk
  drawer.
- Treat downstream experimentation as outsourced R&D. A fork proving demand is
  signal, not an automatic roadmap mandate.

## Before You Choose The Product Surface

- Is the reusable value an engine, protocol, format, renderer, planner, or
  runtime more than it is the default UI?
- Are requests piling up that differ mostly in policy, workflow, or local
  preference rather than shared correctness?
- Would a stable API, CLI, format, or headless mode unlock more value than one
  more built-in feature?
- Can downstreams specialize safely without weakening the invariant the core
  must uphold?
- Is the primitive mature enough to freeze a narrow contract, or are you still
  learning the shape?

## Decision Tree

- Clear mainstream workflow, low customization pressure, and support burden
  dominates: start with the app. Do not split out a platform from premature
  abstraction.
- Strong reusable core plus many niche workflows: build or extract the
  primitive first, then ship a reference app on top.
- Many feature requests are local variants, not universal wins: reject them in
  mainline and expose a seam such as a plugin hook, library API, file format,
  CLI contract, or config surface.
- Downstreams already patch, fork, or scrape: formalize the narrowest stable
  contract immediately. A documented minimal seam is cheaper than pretending the
  demand is accidental.
- Core behavior is still unstable: do not freeze a broad extension API yet.
  Harden invariants, shrink the surface, and delay platform promises until the
  shape stops moving.

## Design Rules That Matter

- Separate mechanism from policy. Core owns correctness and capability;
  reference apps and downstreams own defaults, taste, and local workflow.
- Prefer one load-bearing primitive and one opinionated reference experience
  over three half-reusable layers.
- Make the first seam machine-friendly. A stable CLI plus JSON, embeddable
  library, open format, or headless service usually beats forcing downstreams to
  automate a GUI.
- Version the contract downstreams depend on, not the prose around it. If
  consumers must guess from screenshots or changelog vibes, the seam is fake.
- Measure leverage separately from mainline usage. Track embeds, wrappers,
  plugins, forks, or downstream deployments so product decisions do not optimize
  only the default app.
- Use "no" precisely. If the right answer to a request is "build this on top,"
  the humane move is a seam plus clear docs, not silently carrying the request
  forever.

## What To Pull Back Into Mainline

- Pull back features that multiple downstreams independently reinvent and that
  strengthen the primitive for everyone.
- Leave features outside core when they mainly express policy, UI taste, one
  team's workflow, or a niche integration burden.
- Promote a downstream pattern only after it proves durability, not novelty.
  The point of the ecosystem is to learn before you commit.

## Anti-Patterns

- NEVER call app internals a platform because wrappers exist. If downstreams
  must patch private state or scrape terminal or UI output, you created an
  attractive nuisance, not a building block.
- NEVER absorb every long-tail request into mainline because some users need
  it. That feels generous, but it raises maintenance cost, muddies the product,
  and weakens the stable core that makes downstream creativity possible.
- NEVER ship a weak primitive and expect ecosystem energy to compensate. In the
  building-block economy the core has to be the highest-quality layer, not the
  lowest.
- NEVER freeze a giant extension surface on day one because "platform" sounds
  strategic. Start with the narrowest contract that unlocks composition, then
  widen only from repeated real demand.
- NEVER confuse downstream experimentation with democratic roadmap voting. The
  ecosystem exists to explore faster than mainline can, not to force every
  successful fork back upstream.
- NEVER bury the reusable capability behind an inseparable GUI if the real
  value is the engine underneath. If others can only reuse it by reimplementing
  the core, you are throwing away leverage.

## Fallbacks

- Not ready for a plugin API: ship a stable config format, file format, or
  machine-readable CLI first.
- Not ready to split a library: isolate the core behind the reference app and
  make the seam explicit so extraction is boring later.
- Too many weird downstream patches: identify the smallest repeated need and
  formalize only that. Do not respond by platformizing everything.
- Commercial model is unclear: optimize for adoption and leverage first, but do
  not anchor the design on opaque lock-in being the default choice for agents or
  builders.

## Self-Check

- Did I identify the reusable primitive, or am I just renaming app internals?
- Did I define what stays in mainline versus what belongs in extensions, forks,
  or wrappers?
- Is the contract narrow, documented, and machine-usable?
- Did I keep the core quality bar higher than the downstream quality bar?
- Did I use ecosystem demand as R&D input without letting core become a dumping
  ground?
