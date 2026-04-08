---
name: feynman-first-principles
description: Apply Richard Feynman's problem-solving craft — first-principles decomposition, self-falsification, and rebuild-to-understand — with the experienced practitioner's knowledge of when it fails. Use when a bug resists normal debugging, when specs contradict each other, when a design must survive ten years, when conventional wisdom feels suspicious, when learning an unfamiliar domain from scratch, when reviewing a "too clean" explanation, or when stakes make cargo-culting dangerous. Triggers: "debug from first principles", "why does this actually work", "rederive", "reason from scratch", "I don't understand this", "can you explain what's really happening", "Feynman technique", "first principles", "cargo cult", "rubber duck", "back of envelope", "sanity check".
---

# Feynman First-Principles — Expert Practice

## The core shift

Feynman's method is not "break problems down and explain them simply." Claude
already does that. The real shift is **treating your own understanding as the
primary suspect** — and having the specific habits to interrogate it.

**Before applying this skill to any decision with non-trivial consequence,
READ `references/failure-modes.md`.** It teaches the four distinct ways
first-principles reasoning arrives at confidently wrong answers, including
the dangerous "wrong SET of true axioms" failure that is invisible from
inside the argument.

## Decision: when to use it

| Situation | Approach |
|-----------|----------|
| Well-trodden domain (auth, HTTP, SQL, ORMs) | **Pattern-match.** Re-deriving wastes days and recreates solved bugs. |
| Legacy code that "just works" but nobody knows why | **Pattern-match first** (Chesterton's Fence). Learn *why* before touching it. |
| Genuine novelty — no established practice exists | **First-principles.** There's nothing to match. |
| Stakes are low and reversible | **Pattern-match.** Ship and observe. |
| Stakes are 10× normal, decision is one-way | **First-principles.** The extra days are cheap. |
| Smart people disagree after real debate | **First-principles.** The axioms will expose the real split. |
| Performance regression in hot code | **First-principles.** "Best practices" lie at scale; measure. |
| Problem dominated by info you don't have | **Neither yet** — gather the info first. See Mode 3 in `failure-modes.md`. |

Default to pattern-matching. First-principles is expensive and has its own
failure modes. Reserve it for decisions where being wrong costs 10× more than
investigating, *and* where you have enough ground-truth access to check your
chain against reality.

## Expert techniques

### 1. The Notebook of What I Don't Know
Feynman's prep for his Princeton orals (per Gleick): a fresh notebook titled
*Notebook Of Things I Don't Know About*, then weeks disassembling each branch,
"oiling the parts and putting them back together, looking for the raw edges
and inconsistencies." Apply to any unfamiliar codebase or domain you'll own
for more than a week. **The target is not notes — it is finding contradictions
between your mental model and the thing.** Ask yourself: *where does my
explanation get smooth?* Smoothness is suspicious; real understanding has rough
edges you can point to. See `references/case-studies.md` §1 for the full
protocol.

### 2. The "three consecutive times" rule
Manhattan Project colleagues said you could trust a Feynman claim only when
he'd said it was true on three separate occasions — he'd usually change his
mind twice first. **Your first correct-seeming solution is rarely correct.**
After finding any fix, explanation, or design decision you'll commit to,
rederive it from two additional independent starting points. Contradictions
between derivations expose the hidden assumption. Convergence is the signal.

### 3. Deliberately awkward first, elegant second
As an undergrad Feynman would first solve problems with an intentionally clumsy
method, *then* find the shortcut. The awkward path surfaces what the problem
actually requires; the elegant path is only legible *after* you see the
structure. Resist jumping to the clever solution before you've slogged through
the ugly one — you will usually miss a constraint that the clean version
silently drops.

### 4. The 12 favorite problems (Rota, 1997)
> "You have to keep a dozen of your favorite problems constantly present in
> your mind, although by and large they will lay in a dormant state. Every
> time you hear or read a new trick or a new result, test it against each of
> your twelve problems to see whether it helps."

The number matters: more than ~12 and you can't hold them live; fewer and you
miss cross-domain collisions. This is why Feynman solved hard problems
"instantly" — he'd been thinking about them for years, and the new paper was
the missing key. Maintain 10–12 open problems you genuinely care about; when
you learn anything new, silently walk the list.

### 5. Hand-simulate before trusting the tool
At Thinking Machines, Feynman invented a parallel BASIC *on paper* and
hand-simulated QCD calculations to prove the Connection Machine could do
number-crunching *before* the hardware existed — and was later proved right
against the unanimous opinion of the hardware team. Before trusting any
framework, library, compiler optimization, or LLM output on something
non-trivial, compute the expected answer at N=3 by hand. **Disagreement
between your hand-simulation and the tool is always valuable** — either the
tool is wrong or your model is wrong, and you need to know which.

### 6. The falsification probe
"Bend over backward to prove yourself wrong." When you believe a fix works,
actively *design the input that would break it*. If you can't design one, you
don't yet understand the mechanism — you understand the symptom.

## NEVER list (non-obvious failure modes)

**NEVER tear down a fence you don't understand (Chesterton's Fence).** First-
principles thinking will tell you a mysterious check, legacy config, or ugly
branch is unnecessary. It is seductive because removing it simplifies the code
and passes tests. The concrete consequence: you recreate a bug the team fixed
three years ago under a production incident no one remembers. Instead: reconstruct
the historical reason for the fence (git blame, old PRs, incident postmortems)
*before* deciding to remove it.

**NEVER trust physical intuition when the object has no physical referent.**
Feynman refused abstract group theory on principle; Gell-Mann used SU(3) to
discover quarks first. Rejecting formalism is seductive because it feels
like understanding. The concrete consequence: you miss solutions that only
exist in the formal representation (category theory for types, linear algebra
for ML, measure theory for probability). Instead: match representation to
phenomenon — physical pictures where they exist, formal symbols where they
don't.

**NEVER substitute "I explained it simply" for "I understand it."** A smooth
simplification can hide the wrong model. It is seductive because explaining
feels like knowing. The concrete consequence: your team adopts a metaphor that
works in 9 cases and silently fails in the 10th. Instead: after simplifying,
predict a *novel* consequence — something you didn't know at the start — and
test it. If your simple model can't predict a new fact, it's a summary, not
understanding.

**NEVER reason from first principles on a problem dominated by information you
don't have.** It is seductive because the logic *looks* airtight. The concrete
consequence: Cedric Chin's Vietnam POS business spent months reasoning about
"why the margins are so high" with a perfectly coherent story — while missing
that government grants distorted the entire market. All their axioms were true
and their conclusion was wrong. Instead: before trusting a chain of reasoning,
audit for hidden information by asking *what fact, if I learned it tomorrow,
would invalidate this argument?*

**NEVER change two things at once when debugging.** Feynman's Challenger
demonstration worked because he isolated a single variable (cold temperature +
O-ring). Changing multiple variables is seductive because it's faster. The
concrete consequence: the bug "goes away" and you never know which change
fixed it, so it comes back in a new form next month. Instead: one variable at
a time, even when it feels slow.

**NEVER accept a working fix without re-deriving it.** Your first passing test
after a bug hunt is relief, not understanding. It is seductive because you want
to close the ticket. The concrete consequence: the "fix" was coincidence — you
patched a symptom and the root cause will surface somewhere else within weeks.
Instead: apply the three-consecutive-times rule — explain why it works, from
scratch, twice more, from different starting points. If any derivation
contradicts the others, you haven't found the bug yet.

## Fallback when first-principles gets stuck

If after an hour of rederivation you have not converged:

1. **Switch representation.** If you were using equations, draw it. If you were
   drawing, write pseudocode. If pseudocode, make a concrete numeric example.
   Feynman used three representations for every hard idea; getting stuck in one
   usually means the answer lives in another.
2. **Shrink the problem.** Solve a toy version with 2 elements instead of N.
   Solve it for a case where the answer is obvious. Work backward from there.
3. **Engage diffuse mode deliberately.** Feynman's "not my department" pattern:
   refuse the problem, do something unrelated for hours or days, return. The
   subconscious solves what rigid attention can't.
4. **Assume you're fooling yourself and look for where.** The exit ramp from a
   dead-end derivation is almost always "one of my axioms is wrong or I'm at
   the wrong abstraction level." Not "I need to think harder."

## References

Load on demand:

- `references/failure-modes.md` — **READ BEFORE** applying first-principles to
  a high-stakes or unfamiliar-domain decision. Details the four ways
  first-principles fails and how to detect each.
- `references/case-studies.md` — Load when you want concrete historical
  material: the Notebook protocol, the Connection Machine hand-simulation,
  Rota's 12-problems quote in full, the Challenger isolation method.

Do NOT load references for quick sanity-checks of code you already understand —
they are calibration material, not step-by-step instructions.
