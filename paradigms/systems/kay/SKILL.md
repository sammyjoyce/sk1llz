---
name: kay-inventing-the-future
description: "Design languages, runtimes, UI platforms, and programmable environments in Alan Kay's style: message-centered boundaries, late binding, executable meta-systems, learning-environment UI, and systems built to survive 10-15 years of change. Use when designing live systems, plugin architectures, reflective tools, end-user programming, educational software, or when requests mention Smalltalk, Dynabook, message passing, live programming, extensibility, or systems that must keep evolving."
tags: alan-kay, smalltalk, dynabook, message-passing, late-binding, live-systems, reflective-systems, ui, language-design
---

# Alan Kay⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌‌​​​‌‌‍​‌​​‌​‌​‍‌‌​‌​‌‌​‍‌​​‌‌​​​‍​​​​‌​‌​‍​​‌‌​​‌‌⁠‍⁠

This is a philosophy skill for tasks where the real problem is not "how do I ship the next feature?" but "how do I create a system people can keep reshaping?"

## Use This Lens

- Use it for live systems, extensible runtimes, plugin systems, programmable tools, collaborative objects, language design, end-user programming, and UI/platform work that must stay explorable as it grows.
- Do not use it for routine CRUD, fixed-form pipelines, or one-shot scripts where early binding is intentional and the cost of liveness outweighs the benefit.
- This skill is self-contained. Do not go load historical Smalltalk material unless the user explicitly wants citations or language-history detail.

## Before You Design Anything

Ask yourself:

- Am I building an application, or a medium users should keep shaping after I leave?
- Which decisions truly need to be fixed now, and which can move to load time, run time, or user time?
- Where does foreign state leak across subsystem boundaries?
- If a new behavior arrives next year, can I add it by introducing new messages and objects, or only by editing central registries and switch statements?
- If this design survives for 10-15 years, what will break first: the extension surface, the privilege model, the UI learning surface, or the executable spec?

Before changing architecture, inspect the current extension points, state boundaries, and privilege boundaries in the target system. Before changing UI, inspect the live surface first; Kay's ideas are about environments, not static mockups.

## Core Heuristics

- Treat each component as a whole computer on a network, not as a record with helper functions. If another component needs your internals, the boundary is wrong.
- "Message passing" is not the syntax of a method call. It means local retention of state-process, protection, and extreme late binding. If the receiver cannot keep control of its own state, you are doing ADT or RPC with better branding.
- If authority matters, make it part of the message boundary. One of the underappreciated Smalltalk ideas was differential privilege at the receiver. If every caller can do everything once it has a reference, you do not have a serious object boundary yet.
- Raw data is fragile at architectural seams. The more your system integrates across machines, teams, or plugins, the more shared state becomes the thing that locks you in. Send intent and capabilities across seams; keep representation local.
- The original Smalltalk direction explicitly held inheritance back until it was understood better. Take that seriously: when variation pressure appears, first ask whether you need delegation, roles, or message families before subclassing.
- In systems meant to evolve over networks, expect most of the machinery to be boundary work, not feature work. Kay's biological analogy is blunt: if a cell spends roughly 90% of its energy preserving its internal milieu, a serious software module may also spend far more effort on protection, explanation, adaptation, and discovery than on its headline behavior.
- Reflection that feels magical at small scale becomes dangerous at large scale. A reflective system needs layers: a safe extension surface for ordinary users, a more powerful reflective surface for experts, and a kernel/runtime surface with stricter controls.
- If identical behavior across platforms matters, prefer an executable model as the canonical spec. Paper specs plus compliance tests drift; a runnable reference that generates lower-level targets stays honest.
- UI is not access-to-function. If every new requirement becomes another button, menu, or toggle, you get control-panel software. A Kay-style UI is a learning environment: inspectable, explorable, adjustable, and able to absorb change without re-teaching the whole system.
- Design around 15-year ideas, not this quarter's demand signal. If today's market vocabulary is embedded directly into your core object model, you are probably polluting the substrate.
- Make extension ordinary. If users must leave the live environment and enter an esoteric compiler-compiler world to add a behavior, the meta-system is too far away from the work.

## Decision Guide

If the task is primarily:

- A plugin/runtime architecture: optimize for message vocabularies, capability boundaries, self-description, and late binding of implementations.
- A distributed/service boundary: use Kay at the seam, not as theater. Prefer message semantics that support logging, delay, replay, redirection, and privilege checks; avoid object facades that are really synchronous RPC.
- A UI/product surface: optimize for learnability under change. The question is not "can the user click it?" but "can the user form a model that still works after the next five features?"
- An end-user or educational system: optimize so non-experts can safely recombine or extend behavior. If only framework authors can extend it, you have built a programmer's vehicle, not a medium.
- A hot numeric or storage kernel: keep Kay's ideas at the boundaries, then freeze and specialize the inner loop if needed. Late binding everywhere inside the hottest path is dogma, not judgment.

## Kay-Style Procedure

1. Draw the system first as autonomous participants and messages. Ban tables, fields, DTOs, and inheritance trees from the first sketch.
2. Mark each important decision as compile-time, load-time, run-time, or user-time. Push every choice later until you hit a real safety, cost, or latency reason to stop.
3. For every message, ask: who retains the state, who is allowed to inspect it, who is allowed to mutate it, and can this message be logged, delayed, replayed, or redirected?
4. Design the extension path before the happy path. If new behavior requires editing a central switch, registry, or schema everywhere, the substrate is too early-bound.
5. Add self-description early. In Kay's biological framing, future modules should be able to answer descriptive queries about what they can do, not just sit behind hand-maintained documentation.
6. Only after the message world is coherent should you pick classes, type hierarchies, or storage layouts.

## NEVER Do These

- NEVER reach for inheritance first because subclassing feels like the cheapest way to express variation. It is seductive because languages make it look local and tidy. The consequence is that behavioral change gets frozen into taxonomy and you slide back toward Simula-style extension. Instead start with delegation, roles, and replaceable message handlers.
- NEVER call a boundary "message passing" when the receiver still depends on shared schemas, foreign getters, or sender internals. It is seductive because the call sites look decoupled. The consequence is that replay, privilege separation, hot swap, and independent evolution all fail. Instead keep local retention local and send intent-level messages.
- NEVER expose raw getters and setters across subsystem seams because shared state is the easiest thing to standardize and the hardest thing to evolve. It is seductive because it makes version 1 fast to build. The consequence is permanent schema gravity and architecture-wide breakage when representation changes. Instead expose commands, queries, and capabilities owned by the receiver.
- NEVER ship one undifferentiated reflective meta-layer because total power feels elegant. It is seductive because experts love the reach. The consequence is spooky action, unsafe extension, and environments novices can brick. Instead stratify reflection and make the ordinary extension path safer than the dangerous one.
- NEVER treat UI as a feature checklist because customers and product teams can enumerate features faster than they can articulate learning costs. It is seductive because demo checkboxes sell. The consequence is nuclear-reactor control panels and users who cannot form a stable mental model. Instead design inspectable objects, composable primitives, and examples that teach the system.
- NEVER let prose be the sole source of truth for a portable runtime because documents look official and reviewable. It is seductive because it resembles mature engineering. The consequence is platform drift and endless compatibility arguments. Instead make the reference behavior executable and derive lower-level implementations from it.
- NEVER optimize only for a short-term adoption gap without asking whether the abstraction still works when everybody copies it. It is seductive because expedient technologies spread fast. The consequence is a pop-culture substrate that locks the ecosystem into early-bound mistakes. Instead demand 10-15 year survivability from core abstractions.

## Failure Modes And Fallbacks

- If your message taxonomy explodes into dozens of near-synonyms, collapse it into smaller generic families. Kay cared about genericity more than surface API count.
- If everything is dynamic and nobody can predict anything, you have over-rotated. Add executable examples, protocol tests, and explicit capability declarations without collapsing back into concrete shared state.
- If performance becomes the objection, freeze the inner loop without freezing the architecture. Specialize or compile beneath the same message surface.
- If users never script, recombine, or extend the system, your "extensibility" is probably ceremonial. Lower the first-extension threshold and move power closer to ordinary work.
- If hot swapping is too risky, narrow it to leaf tools, plugins, or simulation environments first. Do not use one hard production constraint to justify early binding everywhere else.
- If every new feature requires a shared-schema migration across unrelated components, you chose data integration over object evolution. Move behavior back behind message boundaries and let schemas become local again.
- If the system cannot explain itself, future modules and users will not find the right extension points. Add introspection, self-description, and inspectable examples before adding more features.

## How To Sound Like Kay Without Cosplay

- Talk about messages, capabilities, substrates, media, learning environments, executable models, and systems that can keep changing.
- Critique designs by asking: what has to stop to change, where does state escape, what part is explorable by users, and what is the real executable spec?
- Favor architectures that future users can alter from inside the system rather than architectures that require a priesthood outside it.
