---
name: polya-how-to-solve-it
description: Apply Pólya's problem-solving framework with Schoenfeld's empirical corrections. Use when stuck on an algorithmic, mathematical, or debugging problem; when a direct approach has failed; when someone is chasing the wrong lead (wild goose chase); when a solution "works" but won't generalize; when you need to turn a vague hunch into a tractable plan. Trigger phrases - "I'm stuck", "don't know where to start", "tried everything", "how do I approach this", "can't see the pattern", heuristics, problem-solving, proof strategy, root cause analysis, why is this hard.
---

# Pólya + Schoenfeld: Problem Solving That Actually Works

Pólya's 1945 *How to Solve It* gave us the famous four phases (Understand, Plan, Execute, Look Back). Four decades of empirical research — chiefly Alan Schoenfeld's — showed the framework is right but **insufficient by itself**. Pólya's "heuristics" are not strategies you can execute: each is a *family* of ~10 distinct strategies hiding under one label. And the real binding constraint on problem-solving success is not which heuristic you pick but whether you notice when your current path is failing. This skill encodes what survived the scrutiny.

## The central insight Claude usually misses

**Pólya's heuristics are labels, not moves.** "Try an easier related problem" is not an instruction — it names roughly a dozen distinct sub-strategies, each triggered by different features of the problem. If you invoke the label without committing to a specific family member, you are pattern-matching to a word cloud, not solving.

Before using any Pólya heuristic, force yourself to answer: *which specific member of this family am I using, and what feature of the problem suggested it?* If you cannot name the feature, you are bluffing.

**Loading guidance**:
- For any non-trivial problem, **READ `references/heuristic-families.md` before choosing a heuristic**. It contains the full Schoenfeld-style decomposition. Do not reconstruct it from memory — the whole point of the research is that the sub-strategies are *not* obvious.
- **Do NOT load `references/worked-example.md`** for normal problem-solving; it is an end-to-end demonstration useful only when teaching the method or debugging why the method is not working on a specific problem.

The 1970s AI research tried to implement Pólya's heuristics directly and largely failed; one leading researcher concluded the labeled strategies were "epiphenomenal rather than real". The finding that rescued Pólya was Schoenfeld's: the labels are *correct descriptions* of what experts do, but each label covers a family that must be taught (or executed) piece by piece.

## Pólya's precise vocabulary (use it literally)

Pólya defined three words with surgical precision. Using them loosely destroys the method:

- **Unknown** — the specific thing whose value or form is missing. Not "the goal" or "the answer". A problem may have several unknowns but only one *principal* unknown.
- **Data** — what is given. Concrete, enumerable.
- **Condition** — the *relation* that ties the data to the unknown. Solutions hide here.

The diagnostic questions that follow from this vocabulary are the ones non-experts skip:
- Is the condition **sufficient** to determine the unknown? **Insufficient**? **Redundant**? **Contradictory**?
- Have I used **all** the data? If not, either I missed a clue or the problem is over-specified (often itself a clue).
- Have I used the **entire** condition? If my solution ignores part of the condition, the solution is almost certainly wrong or incomplete.

The "have I used all the data" check catches more wrong solutions than any other single Pólya move. When a teammate produces a plausible answer that nags at you, this is the first thing to check.

## The metacognitive loop (what Pólya left out)

Schoenfeld recorded video of students failing on problems they had the knowledge to solve. The dominant pathology he named the **wild goose chase**: pick a direction in the first 30 seconds, pursue it for 20 minutes, never re-evaluate. The missing piece is not heuristic knowledge but self-interruption.

Every ~5 minutes of active work on a hard problem, stop and answer three questions:

1. **What am I doing right now?** (Name the specific sub-strategy, not the family name.)
2. **Why am I doing it?** (Connect to the unknown, the data, or the condition.)
3. **How will it help if it works?** (Forecast the *shape* of the intermediate result.)

If you cannot answer any one of them, abandon the current path and return to planning. This single loop accounts for most of Schoenfeld's measurable improvement over raw Pólya instruction. It is more important than any specific heuristic.

## The connection subproblem (why analogy often fails)

Finding an analogous solved problem is the easy half. Bridging from the analog's solution to the original is the hard half — Newell called this the **connection subproblem** and showed it is where most analogical reasoning silently breaks. When you catch yourself saying "this is just like X", pause and ask: *what is the isomorphism?* Write it out explicitly: data→data, condition→condition, unknown→unknown. If the mapping breaks on any of the three, you have a **surface analogy**, not a **structural** one, and it will mislead you.

Surface analogy is the leading cause of wrong-pattern-match bugs in debugging: *"we had this before, it was a race condition"* — symptom matches, mechanism differs, two days wasted.

## Inversion and the inventor's paradox

Pólya observed that the more general problem is sometimes *easier* — because it has more hooks for techniques to grab. But he distinguished two kinds of generalization, and only one works:

- **Concentration** (good): extract the structural essence. Group theory concentrated algebra + number theory + geometry. Test: the general proof is *shorter* than the specific proof.
- **Dilution** (bad): wrap the problem in abstraction that uses no new property of the abstraction. "Let X be any object..." followed by arguments that only use the original special case. Test: the general proof is *longer* than the specific proof, or uses the same steps.

If generalizing did not give you a new tool, you diluted. Roll back.

## Look Back is not "check your work"

"Look Back" in the original means four things, only one of which is verification. Doing only verification is the most common corruption of the method:

1. **Verify** — is the answer correct? (Everyone does this.)
2. **Derive differently** — can you reach the same result by a completely different route? Two independent derivations rarely fail the same way, so agreement is strong evidence. Disagreement localizes the bug.
3. **Audit the hypotheses used** — Pólya's *qui nimium probat, nihil probat* ("he who proves too much proves nothing"). Examine the solution and check that *every* assumption in the problem statement was actually needed. If your argument works with fewer hypotheses, either the problem is over-specified (suspect), or you accidentally proved a stronger statement (extract it), or your solution is wrong and the unused hypothesis would have broken it.
4. **Generalize** — for which larger class of inputs does the argument still work? Where *exactly* does it break?
5. **Transfer** — which *other* problems does this method now solve? Pólya's rule: an idea used once is a trick; used twice, it becomes a method. Name one concrete second problem or you have not transferred.

Skip steps 2–5 and the problem leaves no trace. You will re-solve the same class of problem five times because the first solution never became a technique in your repertoire. For the canonical worked example showing all five in sequence, see `references/worked-example.md` (only load if you need the full walkthrough).

## Anti-patterns (NEVER do these)

- **NEVER invoke a Pólya heuristic by its label alone** (e.g., "let's try a simpler case") because labels feel like progress but commit you to nothing. Consequence: you burn cycles with no falsifiable next step, and 20 minutes later you cannot say what you tried. **Instead**: name the specific sub-strategy from the family table and the feature that triggered it ("I'm setting n=3 because the problem has an implicit integer parameter").

- **NEVER believe a pattern from ≤4 examples.** The classic trap: chords joining `n` points on a circle divide the disk into 1, 2, 4, 8, 16 regions. The next term is **31**, not 32. Consequence: confident wrong answers in a domain where you can't tell you are wrong. **Instead**: treat every pattern as a *conjecture* until you have either a proof or a counterexample, and always compute one more term than feels necessary. Four data points is when the pattern *starts* being worth stating, not when it's worth believing.

- **NEVER work backward without translating to forward form at the end.** Working backward produces a chain of *implications*, not equivalences. Each step must be *reversible*, and reversibility is easy to assume and hard to verify. Consequence: your solution contains a one-way step you cannot justify in forward direction. **Instead**: after working backward, rewrite the whole chain forward and check every "therefore" actually holds.

- **NEVER abandon a failing approach without diagnosing why it failed.** The seductive move is to try the next idea; the expensive move is to ask "what did this attempt rule out?". Consequence: you re-enter the same dead end from a different angle a week later. **Instead**: before switching, write one sentence naming the specific obstacle — e.g., *"sorting does not help because the order depends on a quantity I have not computed yet."*

- **NEVER treat "I've seen this before" as permission to skip Understanding.** Familiarity is the leading cause of solving the wrong problem — you ship a correct solution to the problem you imagined. **Instead**: restate data / condition / unknown *for the current problem* before comparing to the remembered one, and note the **differences** before the similarities. Differences are where the bugs live.

- **NEVER consider a problem solved at the first green test.** Pólya's Look Back is not optional; it is where the technique gets extracted for reuse. Consequence: the solution remains a single-use trick. **Instead**: before closing the task, answer "where else does this method apply?" with one *concrete* second example — not a class of problems, a specific one.

- **NEVER dismiss "have I used all the data" as pedantic.** Unused data is one of: (a) a clue you missed, (b) redundancy that hints at an invariant, or (c) evidence the problem is over-specified for the path you are on. All three are actionable. Consequence of ignoring: you solve an easier problem than the one asked. **Instead**: for each datum in the problem statement, point at the specific step of your solution where it was consumed.

- **NEVER relax a condition without asking which one to relax.** Relaxation is powerful but novices drop any condition; experts drop the one that leaves the *core structural constraint* intact. Consequence: you get a relaxed problem whose solution teaches you nothing about the original. **Instead**: list every condition, rank them by structural importance, drop the least structural one first.

## Decision tree: which heuristic, when

```
Stuck at "I don't understand the problem"
 └── Restate in your own words using Pólya's vocabulary (data/condition/unknown).
 └── If you cannot, you do not understand it. Draw a diagram or build one concrete example.

Stuck at "I understand it but see no plan"
 ├── Is there an analogous solved problem?
 │    ├── Yes, with structural isomorphism → map data/condition/unknown explicitly, reuse method.
 │    └── No or only surface match → proceed to decomposition / specialization.
 ├── Can I drop one non-structural condition?
 │    ├── Yes → solve the relaxed problem, then find the locus/family of relaxed solutions,
 │    │         then intersect with the dropped condition to re-satisfy it.
 │    └── No → specialize.
 ├── Is there an integer parameter (explicit or tacit)?
 │    └── Tabulate n=1,2,3,4,5 looking for an invariant. Compute one more term than you trust.
 └── None of the above → invert. Assume a solution exists and derive its properties.

Plan selected but execution keeps breaking
 └── Run the metacognitive loop (what/why/how). If the three questions
     have bad answers, the plan is wrong, not the execution. Do not patch.

Solution found but feels fragile
 └── Look Back. Derive it a second way. If the two derivations disagree,
     at least one is wrong. If they agree, you have high confidence and
     often a generalization for free.
```

## Fallback when nothing works

After decomposition, analogy, specialization, and inversion have all failed:

1. **Compute something — anything.** This is Schoenfeld's "exploration" phase: generate data about the problem even without a plan. Calculate concrete instances, draw pictures, measure things. The next move often becomes visible after ~10 minutes of concrete computation, for reasons no one fully understands.
2. **Talk through the problem using Pólya's vocabulary.** State the data, the condition, and the unknown out loud. The mismatch between what you say aloud and what the problem literally says is often where the insight hides — you will catch yourself paraphrasing a condition in a way the problem did not.
3. **Incubation is not procrastination.** If a problem has resisted ~45 minutes of sustained focused effort, stop. Pólya explicitly endorsed "sit tight and wait till you get a bright idea" — not as laziness but because focused attention can lock you into a local optimum. The unconscious continues to work; returning in an hour often trivializes what was impossible.
4. **Do not read the solution yet.** If a reference solution is available, resist for at least one incubation cycle. Reading the solution collapses the problem-solving experience into pattern recognition, and you lose the only thing that would have turned this problem into a method in your repertoire.
