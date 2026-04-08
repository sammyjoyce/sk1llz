---
name: feynman-first-principles
description: "Stress-test explanations, estimates, fixes, and designs by rebuilding them from primitives, operating envelopes, and falsification probes instead of trusting polished stories. Use when a bug survives normal debugging, when experts disagree on basics, when a system is outside its validated regime, when you need a model you can hand-simulate, or when a \"simple explanation\" feels suspiciously smooth. Triggers: \"first principles\", \"rederive\", \"what actually causes this\", \"reason from scratch\", \"sanity check\", \"back of envelope\", \"falsify this\", \"simple example\", \"explain why this works\", \"I don't really understand this\"."
---

# Feynman First-Principles

First-principles work is not default reasoning. It is expensive, and in mature
domains it can destroy compressed incident-history that already lives inside
ugly heuristics, checks, and conventions. Use it when pattern-matching is
unsafe, not because scratch reasoning feels purer.

## Invoke or avoid

Use this skill when:
- the system is outside its operating envelope: scale, load, temperature,
  latency, concurrency, budget, adversary model, or failure tolerance changed;
- two competent people disagree because they are using different primitives;
- a clean explanation cannot make a novel prediction;
- you must trust a result before the normal feedback loop can save you;
- tool output conflicts with a hand model or direct measurement.

Do not use this skill when:
- the domain is mature and the question is mostly "which proven pattern fits?";
- the decision is cheap and reversible, so ship-observe beats heroic rework;
- the bottleneck is missing facts, not weak logic.

**Before any high-stakes or unfamiliar-domain use, READ
`references/failure-modes.md`.**
**Before using historical calibration examples, READ
`references/case-studies.md`.**
Do NOT load either reference for routine sanity checks in code you already
understand.

## Before you start, ask yourself

- What exact claim am I trying to trust: explanation, estimate, fix, or design
  rule?
- What is the real operating envelope, not the default one?
- What would count as "right" before I see the answer?
- What quantity should stay conserved, monotonic, bounded, or dimensionally
  consistent?
- What fact, if learned tomorrow, would collapse this reasoning chain?
- What is the simplest concrete case, and how would I know the answer is right
  there?

If you cannot answer those questions, you are not reasoning yet; you are
arranging words.

## Core loop

1. **Freeze the envelope.** Challenger O-rings were not a generic "seal"
   problem; they were a below-53 F problem forced toward a launch near
   freezing. Reasoning that ignores the actual regime is theater.
2. **Get a one-significant-digit answer.** Borrow Feynman's Los Alamos bar:
   first pass is direction and rough scale, not polished algebra. If you cannot
   get the sign of the effect or even the order of magnitude quickly, you
   probably chose the wrong dominant variables.
3. **Build three independent views.**
   - pictorial or physical: what is pushing on what?
   - formal or invariant-based: what must stay true?
   - executable or hand-simulated: what happens at `N = 1, 2, 3`?
   If two views agree and the executable view disagrees, trust the executable
   or measurement first and debug the model.
4. **Attack the boundaries.** Check zero, infinity, empty, saturated,
   single-user, worst-case, and degenerate cases. Singular limits are where
   nearby intuition breaks.
5. **Demand a novel prediction.** A real model predicts a fact you did not
   start with. If it only redescribes known behavior, it is a story, not
   understanding.
6. **Try to kill it.** Change one variable. Design the input that should break
   the explanation. If nothing could falsify it, you have built a worldview,
   not a tool.

## Domain procedures

### Debugging path

- Reproduce the failure in the smallest live case before reading more code.
- Clamp one variable per iteration. Relief is not understanding.
- Explain the failure at the level of the actual mechanism: row, packet,
  syscall, branch, buffer, or clock edge.
- Re-derive the fix from a second starting point before trusting green tests.

### Design path

- Start from invariants and cost surfaces, not APIs or framework slogans.
- Reason at target scale at least once; toy-scale designs lie about caches,
  queues, and coordination.
- Compare your scratch design against the mature pattern you are about to
  reject. If the old pattern exists, ask what wound created it.

### Learning path

- Do a "Notebook Of Things I Don't Know About" pass: record rough edges,
  contradictions, and missing steps, not polished summaries.
- Stop simplifying the instant the metaphor smuggles the original mystery back
  in. Feynman's magnets rule generalizes: do not explain the unknown with a
  friendlier story that secretly depends on the same unknown.

## High-value heuristics

- **Simplest example first.** Danny Hillis remembered Feynman starting with
  "What is the simplest example?" and "How can you tell if the answer is
  right?" Use those two questions to cut through most fake understanding.
- **`N = 1, 2, 3` before `N = 10^6`.** Feynman hand-simulated a parallel BASIC
  model of QCD on the Connection Machine before trusting the hardware story. If
  you cannot simulate a tiny case yourself, you are borrowing conviction.
- **Representation mismatch is not disproof.** Feynman analyzed a Boolean
  message router with continuous equations and predicted five buffers where the
  engineers wanted seven. Weird representation, correct answer. If the
  representation predicts better, keep it.
- **Treat models as status-labeled objects.**
  - `sketch`: helps intuition
  - `predictor`: survives one novel test
  - `design rule`: survives across regimes
  Do not promote a sketch just because it feels vivid.
- **Keep a small live set of hard problems.** Rota's "dozen favorite problems"
  is the right scale: much above roughly 12 and nothing stays live; far below
  that and you miss cross-domain collisions.

## NEVER do these

NEVER reason inside the wrong operating envelope because nearby intuition often
fails at the exact boundary that matters. It is seductive because the algebra
stays clean and the middle of the curve behaves nicely. The consequence is a
model that works at 3 threads and fails at 3,000, or works at 75 F and fails
below 53 F. Instead, state the envelope first and test tiny, typical, worst,
and boundary cases.

NEVER promote a psychologically useful picture to ontology because a model can
be a great guide and still be the wrong thing to believe in. It is seductive
because vivid pictures coordinate teams fast. The consequence is Feynman's
"house of cards": you defend the metaphor instead of the measurement. Instead,
label the model `sketch`, `predictor`, or `design rule` and make it earn
promotion.

NEVER replace the unknown with a comforting analogy because the analogy often
sneaks the mystery back in under friendlier words. It is seductive because
teaching becomes smooth and everyone nods. The consequence is a team that
thinks it understands the mechanism but cannot predict the edge case. Instead,
explain at the lowest honest primitive and admit what remains unexplained.

NEVER re-derive a mature commodity domain from scratch because seasoned
patterns often encode forgotten postmortems, not just convention. It is
seductive because scratch reasoning feels cleaner than inherited ugliness. The
consequence is tearing down a fence and reintroducing a bug the team already
paid for. Instead, ask what incident, regulation, or scaling wound created the
ugly part.

NEVER keep reasoning when hidden external facts dominate the outcome because
true axioms can still be irrelevant. It is seductive because the chain of
logic is internally coherent. The consequence is the Vietnam POS failure mode:
you explain margins beautifully while missing the subsidy, gatekeeper, or human
process that actually sets them. Instead, ask "what unseen fact would flip this
answer?"

NEVER change multiple variables in a falsification loop because getting green
fast feels like progress. It is seductive because it quiets the system quickly.
The consequence is symptom suppression without causal knowledge, so the bug
returns in a new costume. Instead, clamp one variable, predict the result, and
keep the chain inspectable.

## When to stop

Stop first-principles work and switch modes when:
- 45-60 minutes pass without a new falsifiable prediction;
- every disagreement traces back to missing measurements;
- you discover the domain is mostly mature-pattern selection, not novel
  mechanism;
- the cheapest next step is an experiment, benchmark, or log capture.

When that happens:
- switch representation;
- gather the missing measurement;
- compare against the incumbent pattern;
- or deliberately step away and return later with a different primitive.

## Output contract

When using this skill, produce:
- the claimed mechanism in one sentence;
- the operating envelope;
- the smallest case that should work;
- the boundary case most likely to break it;
- the single measurement or experiment that would disconfirm it.

If you cannot provide those five items, say you do not yet understand it.
