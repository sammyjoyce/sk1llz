# How First-Principles Thinking Fails

First-principles reasoning feels logical, so failures feel impossible. They
aren't. Drawing from Cedric Chin's analysis (Commoncog, 2020) and the history
of how Feynman himself got things wrong, there are **four distinct failure
modes** — and you need different defenses for each.

## Mode 1: Flawed assumption

One of your "axioms" turns out to be false. You reasoned correctly *from* it,
but from a bad starting point.

**Why it's the easiest to spot:** a sufficiently diverse group of smart people
can usually find the broken axiom in a review.

**Defense:** list your assumptions *explicitly* before you reason. Any
assumption you didn't write down is the one most likely to be wrong.
Specifically mark which assumptions you've *verified* versus which you're just
confident about. The unverified-but-confident ones are where bugs live.

**Symptoms at review time:** someone says "wait, is that actually true?" and
you realize you've never checked.

## Mode 2: Inference error

Your logic broke between steps. The axioms are fine; a deduction is wrong.

**Why it's the second easiest:** step-by-step review catches most of these.

**Defense:** write out the chain of reasoning in short discrete steps. Long
implicit jumps are where errors hide. If a step reads "and therefore…" without
an obvious mechanism, stop and prove the step.

**Symptoms:** the conclusion is surprising in a *satisfying* way, not a
disorienting way. Real insights disorient; inference errors just feel clever.

## Mode 3: Wrong SET of true principles (the dangerous one)

You picked five axioms. They are all true. Your logic is flawless. Your
conclusion is still wrong because there was a sixth true axiom — one you
didn't know about — that dominates everything.

This is the failure mode that ended businesses, wrecked military campaigns,
and embarrassed Nobel laureates. It is *invisible from inside the argument*
because nothing is technically incorrect.

**Example (Cedric Chin, verbatim from his Vietnam POS business):** "We obsessed
for months over 'why are we making so much money?' We had any number of
theories. They were all wrong." The hidden fact: the government subsidized the
vendor list through grants, distorting margins. Every theory they constructed
from *true* observed facts was wrong because they didn't know about the
grants.

**Why it's seductive:** every time you challenge the argument, it survives,
because the flaw isn't in the argument — it's in what's *missing from* the
argument.

**Defense — the only one that works:** before acting on a first-principles
conclusion, ask:

> *What fact, if I learned it tomorrow, would invalidate this entire chain?*

If you can't name a plausible candidate, you're overconfident. Go find
domain experts and ask them what you're missing. If you *can* name one, go
look for it *before* you commit.

A weaker but useful heuristic: "test against reality." Run a cheap
experiment. If your first-principles argument says you should be seeing X in
the data and you don't, something you don't know about is in play.

**Symptoms:** the world keeps punching you in the face in the same way, and
each time you construct a coherent explanation for the punch.

## Mode 4: Right principles, wrong abstraction level

Your axioms are true, your reasoning is sound, your conclusion is defensible —
and *useless*, because you're at the wrong level of abstraction.

**Example:** deriving how to design a rate limiter from Maxwell's equations.
Technically possible. Useless. The productive level of abstraction is
"requests per window per entity," not "electromagnetic force on electrons in
the switching transistor."

**Defense:** ask yourself — *is this conclusion actionable at the level I care
about?* If the conclusion lives 3 layers below the layer where the decision
gets made, you went too deep. If it lives 3 layers above, you went too
shallow. Feynman was famous for zooming abstraction levels until the answer
*felt obvious*; if it doesn't feel obvious, you're probably at the wrong
level.

**Symptoms:** the derivation is beautiful but you can't say what action it
implies. Or the action it implies is at a layer you don't control.

## When to NOT use first-principles at all

Pattern-matching beats first-principles when any of these hold:

| Condition | Why pattern-match instead |
|-----------|--------------------------|
| The domain has 50+ years of accumulated fence-building (auth, SQL, filesystems) | Every "first-principles simplification" has already been tried and failed; you will recreate known bugs |
| The decision is reversible and cheap | The cost of pattern-matching wrong is lower than the cost of rederiving |
| You lack a critical piece of information (see Mode 3) | First-principles will produce a confident wrong answer |
| The problem is dominated by tacit knowledge a practitioner holds | You cannot rebuild what you've never seen |
| Speed matters and the stakes are bounded | First-principles is slow by design |

Use first-principles when:

| Condition | Why first-principles wins |
|-----------|--------------------------|
| Genuine novelty — no established practice exists | There's no pattern to match |
| The existing consensus is known to be wrong (you have direct evidence) | Pattern-matching reproduces the error |
| Decisions are one-way and stakes are 10× normal | The extra days are cheap relative to a wrong commitment |
| Multiple smart people disagree | The axioms expose where the split really lives |
| You have verifiable direct access to the ground truth | You can check your chain |

## Chesterton's Fence (the anti-cargo-cult corollary)

The cargo-cult critique says: "don't do things you can't explain the reason
for." Taken naively, this becomes "tear down anything you can't justify." That
is also wrong — it's Mode 3 at the team level.

Chesterton's rule: before removing a fence, figure out *why it was put there*.
Only then do you have enough information to decide whether to keep it. In
code: before deleting a check, a flag, a branch, or a config option you don't
understand, find the commit, the PR, the incident ticket, the post-mortem.
Nine times out of ten you will find it was put there for a reason that still
holds.

The Feynman technique is the *inside* of cargo-cult avoidance ("understand
what you do"). Chesterton's Fence is the *outside* ("respect what was done").
Both are required. Either alone fails.

## The meta-lesson

The point of understanding these failure modes is not to stop using
first-principles thinking. It is to use it with the right humility. Feynman
himself failed at Mode 2 famously enough that Manhattan Project colleagues
developed the rule "only when Dick says it's true three separate times can
you count on it." He wasn't reliable on the first pass — and he's the model.
You won't be either. Build the rederivation loop in by default.
