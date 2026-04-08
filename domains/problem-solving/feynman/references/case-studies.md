# Case Studies: Feynman's Actual Methods in Practice⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​​​​​‍​‌‌‌‌‌​​‍​‌‌​​‌‌‌‍‌​​​​​‌​‍​​​​‌​​‌‍​‌‌‌‌‌​‌⁠‍⁠

Concrete historical material. Use these as calibration for the patterns in
`SKILL.md` — they show what the techniques look like in their original
context, and what the practical numbers were.

## 1. The Notebook of Things I Don't Know About

From James Gleick, *Genius*, on Feynman's preparation for his Princeton oral
qualifying exam:

> "He chose not to study the outlines of known physics. Instead he went up to
> MIT, where he could be alone, and opened a fresh notebook. On the title page
> he wrote: *Notebook Of Things I Don't Know About*. For the first but not the
> last time he reorganized his knowledge. He worked for weeks at disassembling
> each branch of physics, oiling the parts, and putting them back together,
> looking all the while for the raw edges and inconsistencies. He tried to find
> the essential kernels of each subject. When he was done he had a notebook of
> which he was especially proud."

**Key details practitioners miss:**

- It was a **fresh notebook**, not additions to an existing one. The blank page
  forces you to admit what you don't already have.
- It took **weeks**, not hours. The Feynman Notebook is a long project, not an
  afternoon exercise.
- The title frames the work as cataloging *ignorance*, not knowledge. The
  default reading assumption is "I don't know this" — you have to earn any
  claim of understanding.
- The phrase "raw edges and inconsistencies" is the operational target. You
  are hunting for places where your mental model contradicts itself or runs
  out, not places where it succeeds.
- "Essential kernels of each subject" means: what is the smallest
  self-contained core from which the rest can be rebuilt?

**Practical adaptation for software:**
When onboarding to an unfamiliar codebase you'll own for a month or more,
create a single document titled `things-i-dont-understand.md` in your personal
notes. Populate it by reading code and recording *every* question you can't
immediately answer. Resist the urge to look up the answer right away — batch
the lookups after you've assembled a critical mass of questions. The pattern
of questions reveals the shape of your ignorance faster than any individual
answer.

## 2. The "three consecutive times" rule (Manhattan Project)

From contemporary accounts collected by Tom Crick and others: Feynman's
Manhattan Project colleagues developed a working heuristic that "only when
Feynman said something was true on three consecutive occasions, you could
count on it." He would often change his mind twice before settling.

**Why this matters:** Feynman wasn't reliable on first derivation. He was
reliable on third. His technique was *iteration to convergence*, not brilliant
one-shot insight. The method was: state a conclusion, then re-derive it
tomorrow from a different starting angle, then re-derive it once more. The
conclusion was trusted only when it survived three independent derivations.

**Adapt for debugging:** When you believe you've found a bug, don't just fix
it. Derive *why* it was a bug from two additional independent angles (e.g.,
from the call site, from the data model, from the test that should have caught
it). If any derivation contradicts your original story, your diagnosis is
wrong — the fix is either coincidental or papering over a deeper issue.

**Adapt for design decisions:** On any architectural choice that will take
weeks to reverse, argue the conclusion from three distinct framings before
committing. If all three agree, trust it. If they diverge, you haven't found
the axis that actually matters.

## 3. The Connection Machine hand-simulation

From Danny Hillis' memoir *Richard Feynman and The Connection Machine*:
When Thinking Machines Corp was building the Connection Machine, the team had
decided the machine *couldn't* be used for number-crunching (quantum
chromodynamics), because the first prototype had no dedicated floating-point
hardware. "Everyone knew" this was required.

Feynman disagreed. He *made up a parallel BASIC dialect on paper* and then
**simulated by hand** how a QCD calculation would run across the machine's
processors. His hand calculation showed the Connection Machine — even without
floating-point silicon — would outperform the purpose-built QCD machine
CalTech was constructing. The team initially ignored him. Months later, when
chip size forced them to reduce buffers per chip, they were reluctantly forced
back to Feynman's analysis, which had already shown the smaller buffer count
was safe. He was right.

**What made this work:**

- He built the simplest possible simulation environment (parallel BASIC
  invented on the spot) rather than waiting for real tooling.
- He hand-executed the simulation. No shortcuts, no approximations, no
  "imagine roughly how this would go."
- He *trusted the result* enough to argue against the collective wisdom of
  experienced hardware engineers.
- When he was later proved right, it was because he had checked the detail
  that everyone else had treated as "obvious."

**Adapt:** before trusting any framework, library, compiler optimization, or
LLM output on a non-trivial problem, work out the expected answer at N=3 by
hand. If the tool disagrees with your hand calculation, either the tool is
wrong or your mental model is wrong — both outcomes are valuable. The common
mistake is skipping the hand check because "the tool is obviously right."

## 4. The 12 favorite problems (Rota's memorial essay)

From Gian-Carlo Rota's 1997 memorial essay for Feynman in the *Notices of the
American Mathematical Society*:

> "Richard Feynman was fond of giving the following advice on how to be a
> genius. You have to keep a dozen of your favorite problems constantly
> present in your mind, although by and large they will lay in a dormant
> state. Every time you hear or read a new trick or a new result, test it
> against each of your twelve problems to see whether it helps. Every once in
> a while there will be a hit, and people will say, 'How did he do it? He
> must be a genius!'"

**Why the number ~12:** fewer than about 10 and you miss cross-domain
collisions (the new trick is less likely to match something on a short list).
More than about 15 and you can't hold them all *live* in working memory —
they fall into the general heap of "things I used to care about" and stop
acting as a filter on incoming information.

**The key verb is "test against."** Not "remember." Not "think about." When
you encounter a new technique, you are actively mapping it against each of
your 12 problems, asking "does this help with #3? with #7?" This is
deliberate, not passive.

**Adapt:** maintain a list of 10–12 open problems you genuinely care about —
bugs that have resisted you, design questions you haven't answered, systems
you'd like to understand. Every time you learn a new library, paper, trick,
or pattern, silently walk down the list. Most entries will get no hit. That's
expected. The 1-in-100 hit is what makes the practice worthwhile.

## 5. The Challenger O-ring demonstration (single-variable isolation)

Feynman's televised demonstration during the Rogers Commission: he put a
piece of O-ring rubber in a C-clamp, dunked it in his glass of ice water, and
showed the rubber lost resilience at low temperature. One variable. One
observation. Total runtime: under a minute.

**Why this succeeded where the formal analyses had stalled:**

- He isolated **a single variable** (cold) and demonstrated its effect on a
  single property (resilience).
- The demonstration could be done in front of the camera with household
  objects. No formalism to dispute, no expertise required to see the result.
- Everyone else had been arguing about the full system behavior of the
  booster seal. Feynman argued about one material property of one part.

**Adapt for debugging:** when a bug could be any of several things, resist
changing multiple variables simultaneously. Design a test that isolates
exactly one suspected cause. The test will be crude and often ugly. That's
the point — you want an unambiguous signal, not an elegant experiment.

## 6. Feynman's blind spot: SU(3) and the abstract-math refusal

The uncomfortable counterpoint to the rest of this document.

Feynman famously distrusted abstract mathematics he couldn't visualize
physically. Lenny Susskind, at the Caltech Feynman 100 event, argued this was
the source of his strength. It was also his most expensive blind spot.

During the development of quark theory, Murray Gell-Mann used the
representation theory of SU(3) — purely abstract group theory — to predict the
structure of the hadronic particle zoo. This led directly to the discovery of
quarks. Feynman worked on the same problems around the same time but refused
the group-theoretic formalism. He used his own language ("partons") and did
not reach the structural insight. Gell-Mann did. Feynman was eventually
convinced, but only after the experimental evidence forced the issue.

**The lesson:** physical intuition is the right tool when there is a physical
referent. When the object lives only in the formalism (group theory, category
theory, linear algebra for ML, measure theory for probability), insisting on
physical intuition is refusing the field. *Match the representation to the
phenomenon* — don't project your preferred mental style onto the problem.

Feynman himself, in his lectures, acknowledged situations where the math had
outrun the intuition. He just tended, late in life, to dig in anyway. Don't
imitate that part.

## Summary table

| Case | Technique | Key number or detail |
|------|-----------|---------------------|
| Notebook | Disassembly to find raw edges | Took *weeks*, not hours; fresh notebook |
| Three consecutive | Iterate to convergence | 3 independent derivations |
| Connection Machine | Hand-simulate before trusting the tool | Invented parallel BASIC on paper |
| 12 problems | Keep open problems resident | ~10–12, live in working memory |
| Challenger | Single-variable isolation | 1 variable, 1 observation, <60 seconds |
| SU(3) blind spot | Match representation to phenomenon | When the object is formal, use formalism |
