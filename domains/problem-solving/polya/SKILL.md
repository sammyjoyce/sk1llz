---
name: polya-how-to-solve-it
description: "Apply Pólya-style problem solving corrected by Schoenfeld and Mason for unfamiliar math, algorithm, debugging, proof, and design problems. Use when you are stuck, chasing the wrong lead, can produce examples but not a method, have a result that will not generalize, or need to turn a hunch into a tractable attack. Triggers: \"I'm stuck\", \"don't know where to start\", \"tried everything\", \"works on examples only\", \"what heuristic should I use\", \"can't see the invariant\", \"why does this fail\", \"false lead\", \"need a proof strategy\"."
---

# Pólya After The Corrections

Pólya's four phases are still the right shell. The expert corrections are these: the heuristic labels are too coarse to execute directly; most failure is bad control, not lack of ideas; and anomaly management matters more than elegance. This skill is for unfamiliar problems, not routine exercises whose method is already signaled by context.

## Mandatory Loading Rules

- Before choosing a plan for any non-routine problem, READ `references/heuristic-families.md`. Schoenfeld's key result is that "try a simpler problem" or "use analogy" are labels for families of moves, not executable moves.
- READ `references/worked-example.md` only when teaching the method, auditing why your process failed, or you need one full end-to-end demonstration.
- Do NOT load `references/worked-example.md` during live solving unless you are explicitly debugging your process; it anchors you to one canonical pattern and makes surface analogy more likely.
- Do NOT load either reference for routine exercises or tasks where the operative method is already known. In those cases, execute directly.

## Hard-Won Truths

- Treat every heuristic label as a menu, never as an action. If you cannot name the exact family member you are using and the feature that triggered it, you are not solving; you are narrating.
- On unfamiliar problems, experts spend more than half their early time making sense of the problem and exploring structure before committing to implementation. If you rush into execution, you are already in novice mode.
- In Schoenfeld's videotapes, roughly 60 percent of unsuccessful attempts followed the same pattern: read, make a quick decision, then pursue it stubbornly. The first bad commitment is usually the real bug.
- Analogy rarely fails at retrieval; it fails at transfer. The relaxed problem or analogous problem is only half the job. The hidden job is the **connection subproblem**: how its solution maps back to your original data, condition, and unknown.
- Surprise is signal. When a result contradicts your expectation, do not smooth it away as noise. Mason's point is that contradiction is often the event that makes structure visible.
- Specializing has three different jobs. Use **random** examples to get a feel, **systematic** examples to expose pattern, and **artful** examples to break your own conjecture at regime changes. Using only one mode gives you the wrong evidence.

## Before You Move, Ask Yourself

Before planning, pin down Pólya's exact vocabulary:

- **Unknown**: what specific value, object, or statement is missing?
- **Data**: what is actually given?
- **Condition**: what relation ties the data to the unknown?

Then ask the questions that experts use and novices skip:

- Is the condition sufficient, insufficient, redundant, or contradictory?
- Which datum has not been used yet? Unused data is usually a clue, an invariant, or evidence you solved an easier problem.
- For proof tasks, which hypothesis has not been used? If your argument works without it, either you proved a stronger statement, the problem is overspecified, or the proof is wrong.
- What are the **dimensions of possible variation** here? What can change while the object is still the same kind of object?
- What range of change is actually permissible? Many false generalizations come from varying a parameter outside the problem's meaningful range.
- Before relaxing a condition, ask: which condition carries the core structure, and which one can I drop without destroying the mechanism I need to learn?
- Before trusting a pattern, ask: which boundary case, carry, parity flip, empty case, or dimension jump have I not crossed yet?
- Before using an analogy, ask: what exact object from the analog will reconnect to the original problem, and where could that reconnection fail?

## Operating Loop

### 1. Understand Structurally

- Rewrite the problem in terms of unknown, data, and condition.
- If the problem has parameters, identify the smallest non-degenerate case that preserves the structure. "Smallest" is often wrong; "smallest still interesting" is right.
- If you already have examples, classify them by regime changes, not adjacency. Boundary crossings like zero/nonzero, even/odd, sign changes, carry/borrow, empty/non-empty, or dimension changes expose structure faster than ten neighboring cases.
- When generating examples, use Mason's order deliberately: random for orientation, systematic for pattern, artful for attempted refutation.

### 2. Choose A Real Move

Pick one explicit family member from `references/heuristic-families.md`, then write one sentence of the form:

`I am using <specific move> because this problem has <trigger feature>, and if it works I expect to obtain <intermediate object>.`

If you cannot finish that sentence, go back to understanding.

### 3. Run The Control Loop

Every 5 minutes on a hard problem, interrupt yourself and answer:

1. What exactly am I doing?
2. Why am I doing it?
3. How will the outcome help?

If any answer is vague, you are in a wild goose chase. Stop execution, vary the problem, or change representation. If you fail two check-ins in a row, the plan is wrong; do not "push through."

### 4. Look Back The Expert Way

Look Back is not "spot-check the answer." It means:

- Verify the result against all data and the whole condition.
- Derive it a second way if the stakes are high. Independent derivations catch different classes of error.
- Audit which hypotheses were actually used.
- Ask where the method breaks, not just where it works.
- Name one second problem to which the method transfers. If you cannot, you found a trick, not a method.

## Decision Tree

**If you do not understand the problem**
- Restate it with unknown, data, and condition.
- Draw one concrete instance or construct one exact example.
- If you still cannot restate the condition precisely, you are not ready to plan.

**If you understand it but have no plan**
- Check `references/heuristic-families.md`.
- Prefer specialization that preserves structure, or relaxation that drops the least structural condition first.
- If an analogy appears, write the mapping explicitly: data to data, condition to condition, unknown to unknown.

**If you have a conjecture from examples**
- Test it on regime changes, not just nearby cases.
- Compute one more case than feels necessary. Four matching cases is where a conjecture becomes interesting, not trustworthy.

**If a relaxed problem or analog is solved**
- Stop and solve the connection subproblem: what exact object, invariant, or locus from the easier problem reconnects to the original condition?

**If execution keeps expanding**
- You are probably in a belief trap: "real progress must look like computation" or "this must reduce to the last bug I saw."
- Change representation before doing more work: algebra to picture, picture to coordinates, recurrence to table, symptom list to mechanism.

**If the result feels brittle**
- Rewrite the reasoning forward.
- Check reversibility of each backward step.
- Ask which hypothesis would break the proof if removed. If none would, inspect for overgeneralization.

## NEVER Do These

- NEVER invoke a heuristic by label alone because it feels like forward motion while committing you to nothing; the consequence is that 20 minutes later you cannot say what you actually tried. Instead do a named sub-strategy with an explicit trigger and expected payoff.
- NEVER sample only adjacent easy cases because neighboring examples create fake regularity and hide regime changes; the consequence is a conjecture that survives friendly tests and dies on the first real boundary. Instead sample ladder cases and adversarial boundary cases.
- NEVER relax a condition just because it is easiest to drop because the seductive path usually removes the very structure that teaches you about the original problem; the consequence is an easier subproblem whose solution does not reconnect. Instead drop the least structural condition that leaves the core mechanism intact.
- NEVER treat an analogy as valid because the symptoms or vocabulary match because surface resemblance is cheap and structural correspondence is rare; the consequence is importing the wrong method and burning time on a mechanism the problem does not have. Instead write the data-condition-unknown mapping and downgrade the analogy to a hint if any part does not map cleanly.
- NEVER keep computing after two bad control-loop answers because more work on a bad plan only deepens commitment and makes abandonment emotionally harder; the consequence is grind that feels productive and teaches nothing. Instead stop, diagnose the obstacle in one sentence, and vary the problem.
- NEVER work backward and then present the backward chain as a proof because implication is not equivalence and one irreversible step can poison the whole argument; the consequence is a polished false proof. Instead rewrite the full chain forward and verify each step is justified.
- NEVER ignore an unused datum or hypothesis because it is often the only evidence that you solved a simpler cousin of the real problem; the consequence is a solution that looks clean while silently missing the asked constraint. Instead point to the exact step where each datum or hypothesis is consumed.
- NEVER sand down a surprising example because contradiction is often the first glimpse of the invariant, hidden case split, or wrong representation; the consequence is losing the one clue that would have changed your model. Instead preserve the surprise and ask what assumption it just falsified.

## Freedom Calibration

- Use high freedom when choosing a representation or heuristic family. Experts vary problems creatively; rigid scripts fail here.
- Use low freedom when validating a proof, checking reversibility, or reconnecting a relaxed problem to the original. These are brittle steps where hand-waving causes elegant wrong answers.
- Use medium freedom when specializing: vary boldly, but record why each example was chosen.

## Fallbacks And Stop Rules

- If nothing is working, write `STUCK:` followed by the current obstacle, the last move attempted, and one regime you have not tested. Mason's point is that naming stuckness restores the monitor.
- If you have many examples but no method, organize them by what changed, what stayed invariant, and where behavior flipped.
- If you have a method for the easier problem but not the original, stop generating new easier problems and solve the connection subproblem.
- If you have been executing for 10 straight minutes without changing representation, asking a control question, or learning a new structural fact, you are almost certainly grinding.
- If the problem has resisted a full focused pass, take an incubation break before reading a solution. Looking too early converts a learnable method into borrowed pattern recognition.
