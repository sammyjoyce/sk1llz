# Oracles and Coverage — Deep Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​​‌​​‌‍‌​‌​‌​​​‍‌​​‌​​​‌‍‌‌‌​‌​​​‍​​​​‌​‌​‍‌​​​​​‌​⁠‍⁠

Load this when you need to frame a bug report precisely, build an oracle checklist for a risky area, or construct a test strategy from scratch.

## A FEW HICCUPPS — full definitions with failure modes

Each oracle is **heuristic**: it can suggest a problem but cannot prove absence. Every oracle has conditions under which it misleads you. Knowing the failure mode is half the skill.

| Oracle | What you expect | How it misleads you |
|---|---|---|
| **A**cceptability | The product is as good as it can reasonably be, not merely "not wrong" | "Acceptable to whom?" drifts across stakeholders; product-owners and end-users disagree silently |
| **F**amiliarity | Product is inconsistent with patterns of bugs you've seen before | Blinds you to novel failure modes in new domains; turns into confirmation bias fast |
| **E**xplainability | You can describe its behaviour clearly to a colleague | You may mistake your own confusion for a product bug (it's often a model bug instead — still worth noting!) |
| **W**orld | Product behaves consistently with real-world knowledge | Your world-model may be wrong; product may expose something real you didn't know |
| **H**istory | Present version behaves like past versions | A deliberate improvement looks like a bug; a deliberate regression to fix past misbehaviour looks correct |
| **I**mage | Behaviour fits the brand/reputation the org wants to project | You may not know the image the org is projecting; image shifts quietly after marketing changes |
| **C**omparable products | Behaves like analogous features in other products | Competitors may have the *same* bug; "everyone does it this way" ≠ correct |
| **C**laims | Behaves as docs, specs, ads, conversations promise | Claims may be wrong; the bug may be in the claim, not the product. Report both. |
| **U**sers' desires | Behaves as reasonable users would want | "Reasonable user" is a model you invented; build several personas, not one |
| **P**roduct (internal consistency) | Product is consistent with itself across features | Large products are inconsistent by design (legacy areas); escalate systematically, not as individual bugs |
| **P**urpose | Behaves consistently with explicit AND implicit uses | Implicit purpose is the leakiest — stakeholders only articulate it after you violate it |
| **S**tatutes | Consistent with laws, regulations, standards | Laws change; standards conflict across jurisdictions |

**The "A" was added later** — the original HICCUPPS didn't include Acceptability. If you're reading older material or training decks, know that the current Bach/Bolton list is *A FEW HICCUPPS*, not HICCUPPS or FEW HICCUPPS alone.

## Doug Hoffman's complementary oracles (for when A FEW HICCUPPS isn't concrete enough)

Hoffman's list is more operational and better-suited to high-volume automated checking. Use alongside, not instead of, consistency heuristics.

- **Constraint oracle** — impossible values or impossible relationships (a US ZIP that isn't 5 or 9 digits; a timestamp in the future on a historical record)
- **Regression oracle** — compare current result against prior version on same input
- **Self-verifying data** — embed the correct answer in the test input (round-trip encoders, protocols with known-response pairs)
- **Physical/business model oracle** — a model of the domain predicts what should happen; divergence is a signal (but the model can be wrong too)
- **Statistical oracle** — expected distribution of outputs (useful for LLMs, recommender systems, simulations)
- **Consistent-with-specification** — the simplest, laziest, and most brittle. Use last.

**Rule of thumb:** A FEW HICCUPPS is for humans recognising problems in the moment; Hoffman's list is for encoding oracles into checks.

## SFDPOT — Product Elements (Bach's HTSM coverage model)

When asked *"what did you look at?"*, organise your answer by these six. Anything in your product that isn't in one of these six is probably uncovered.

- **S**tructure — code modules, files, config, DB schemas, dependencies, binaries
- **F**unction — features, error handling, calculations, transformations, workflows, business rules
- **D**ata — inputs, outputs, stored state, data flows, constraints, lifecycles (CRUD)
- **P**latform — OS, browsers, hardware, networks, external services, configurations
- **O**perations — real user scenarios, startup/shutdown, recovery, updates, maintenance, integration
- **T**ime — concurrency, timeouts, date/time handling, scheduling, sequencing, race conditions

**Expert move:** if your test strategy document has no explicit Time section, you almost certainly have no Time coverage. This is the single most common gap.

## CRUSSPIC STMPL — Quality Criteria

Used to answer *"what makes this product good?"* — each criterion is a different lens, and bugs hide in the seams between lenses.

- **C**apability — can it perform its functions?
- **R**eliability — will it keep working?
- **U**sability — can real users use it?
- **S**ecurity — is it protected from threats?
- **S**calability — does it handle growth?
- **P**erformance — is it fast enough?
- **I**nstallability — can it be deployed?
- **C**ompatibility — does it play well with other things?
- **S**upportability — can it be maintained?
- **T**estability — can *it* be tested effectively?
- **M**aintainability — can it be changed?
- **P**ortability — does it move to new environments?
- **L**ocalizability — can it be adapted for locales?

**Testability deserves its own attention.** If testability is poor, you're not allowed to say "we tested it" — you're allowed to say "we testability-limited tested it." Raise testability as a first-class bug category.

## The Honest Manual Writer Heuristic

Play a game: *you are writing an honest tech-support manual for this feature.* Not marketing copy — a cold, accurate, user-facing manual that must not cause support tickets. Now use the feature and try to write the manual as you go.

Why it works: marketing copy can paper over anything. A manual cannot. You'll find yourself writing things like *"if the dialog takes more than 3 seconds, press Cancel and retry"* — which is a bug report in disguise. Anything that requires a caveat, a workaround, or a "don't do X or Y will happen" note is a bug candidate.

This heuristic is especially powerful on features the team is proud of. Use it exactly there.

## The Blink Test

Scroll through a huge volume of output (a long log, a table with 10,000 rows, a rapid UI refresh) **too fast to read any individual item**. Let your peripheral vision and pattern recognition work. Bugs that are invisible in detailed inspection jump out as visual anomalies — a sudden colour shift, a changed column width, a gap in a stream. Use Excel's conditional formatting or `less -S` to help.

**When to use:** any time the data volume is so high that deliberate inspection is impossible. It is not a substitute for inspection; it is a filter to find *where* to inspect.

## Generic Test Procedure (a last-ditch framework)

When you've run out of ideas on an unfamiliar product:

1. **Model** the product — sketch SFDPOT on paper or a whiteboard.
2. **Determine coverage** you want for this session.
3. **Select oracles** (A FEW HICCUPPS + any Hoffman oracles that apply).
4. **Configure** the system under test — baseline state, data, platform.
5. **Operate** — interact with it deliberately, noting what you do.
6. **Observe** — what does it do, what doesn't it do, what does it *almost* do?
7. **Evaluate** against your oracles.
8. **Report** using safety language.

This is not a script. It is a scaffold to get you unstuck — use it once and move on.

## Coverage claims: the only honest phrasing

Bach's levels for describing coverage in any area — use these verbatim in status reports:

- **Level 0** — "We know this area exists but it's a black box to us."
- **Level 1** — "Basic reconnaissance; smoke/sanity done; we have some artifacts of our models."
- **Level 2** — "We've looked at the core and critical aspects; meaningful tests exist."
- **Level 3** — "We've tested under complex, harsh, and exceptional conditions."

Most teams mistake Level 1 work for Level 2 coverage. That's the single most common reporting error. If in doubt, downgrade.
