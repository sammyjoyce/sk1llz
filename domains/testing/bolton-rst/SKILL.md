---
name: bolton-rapid-software-testing
description: "Diagnose and improve software testing using Michael Bolton and James Bach's Rapid Software Testing heuristics. Use when evaluating a test strategy, reviewing automation or regression plans, creating or coaching exploratory sessions, writing charters or debriefs, framing release risk, surfacing testability issues, or defending exploratory work to managers, auditors, or developers. Triggers: exploratory testing, session-based test management, testing vs checking, FEW HICCUPPS, SFDIPOT, charter, debrief, coverage, testability, bug advocacy, \"all tests pass\"."
---

# Bolton Rapid Software Testing

RST is for moments when a team is about to fool itself: green bars are being mistaken for product knowledge, scripts are being mistaken for coverage, or bug counts are being mistaken for tester productivity.

The testing/checking distinction exists to prevent category errors, not to win status games. Great checking lowers the cost of testing. It also creates the easiest failure mode in software quality: the team starts treating "nothing noticed by the checks" as "nothing important is wrong."

RST also prefers activity-based management over artifact-based management. Large piles of cases, scripts, and status sheets are easy to count and easy to worship; they are not the same thing as learning.

## Before you act, ask yourself

- **Who needs the next decision?** Testing serves a person who matters. Name that person before writing a plan or report.
- **Am I under-tested or under-instrumented?** If you cannot control important states, vary environmental variables, or observe meaningful signals, the next task is testability advocacy, not heroic exploration.
- **What would make us look productive while shrinking coverage?** Test-case counts, bug quotas, and pass-rate dashboards often reward the wrong work.
- **What new information will this next activity produce that the previous one could not?** If the answer is "none", you are rehearsing, not learning.
- **Is subjective testability low?** If the product area demands domain knowledge or technical skill you do not yet have, say so and get help. Bluffing is slower than pairing.
- **What is the risk gap right now?** State what the team knows, what it still needs to know, and what is only being assumed.

## Choose the mode

- **Asked to "write tests for X"**: Treat it as checking unless the user explicitly wants investigation. Build the smallest worthwhile set of checks, then state the risks those checks do not cover.
- **Asked to "test X" or "find bugs"**: Treat it as exploratory work. Use a charter and time box. The deliverable is a session report, not pass/fail.
- **Asked "can we ship?"**: Do not answer yes/no first. Deliver three stories: product status, coverage achieved, and what reduced the value of the testing.
- **Blocked by unstable builds, poor logs, missing environments, or missing expertise**: Raise issues immediately. In RST, issues are threats to the project and test effort, not miscellaneous notes.

## Operate like an RST practitioner

- Start broad, then focus, then defocus. Survey testing finds seams; focused attack exploits them; defocusing before you stop catches blind spots created by tunnel vision.
- Treat emotion as telemetry. Surprise, confusion, impatience, frustration, or boredom are not noise; they often point to reliability, usability, performance, charisma, or testability threats.
- Charter with a mission, not a script. A good charter says what to learn and what to worry about. Specific charters buy focus but cost design effort, so general charters are acceptable early when the map is still poor.
- Variation beats repetition. Each test should do work the previous one did not. Once a bug is fixed, the original script rapidly loses power. Verify the fix, then vary data, order, platform, timing, or state; otherwise you are mostly testing the script.
- Testability is broader than logs. RST treats observability, controllability, smallness, and simplicity as part of testability. If setup is huge, state is inaccessible, or every path has too many interacting conditions, coverage will collapse no matter how hard the tester works.
- Do not wait for runnable code to begin testing. Ideas, designs, docs, diagrams, APIs, prototypes, infrastructure plans, and release assumptions are all testable products.
- Maintain close social distance and critical distance. Work closely with the team, but keep the job of challenging assumptions, exposing risk, and questioning "done."

## Session mechanics that matter

- Use time boxes that are loose enough for human work and tight enough for correction. Standard RST ranges are short `60 +/- 15` minutes, normal `90 +/- 15`, and long `120 +/- 15`. Beware fake precision.
- Debrief quickly. The debrief is for calibration, coaching, and adjusting charters while the session is still fresh, not for policing.
- Record TBS as rough estimates, not stopwatch theater. Use nearest `5%` or `10%`.
- When classifying simultaneous work, use precedence `T > B > S`: test design and execution, then bug investigation and reporting, then setup.
- Exclude opportunity testing from TBS. The point of TBS is to track interruptions to charter work.
- Read high `B` time as a coverage warning, not a productivity badge. Heavy bug investigation can mean you hit a rich seam, but it also means the rest of the space stayed dark.
- Never compare testers using raw TBS, bug counts, or test counts. Those numbers are confounded by module complexity, luck, product dirtiness, and the quality of the questions being asked.

## When the environment is hostile

- If management insists on scripted cases, note every unrun idea together with the trigger for it: risk, oracle, coverage hole, or testability concern. Report those ideas every `1-2` hours. Ask, "Are you okay with us not running these?"
- If bureaucracy demands exhaustive documents, show the opportunity cost. Over-documentation suppresses the kinds of scenario variation and exploratory moves that actually find surprising problems; one day of sharp exploratory work often reveals what a week of detailed procedure execution will miss.
- If builds keep breaking, environments differ from production, or tools slow you down, log issues before anyone asks why coverage is thin. Issues amplify existing risk because they give bugs more time and more places to hide.
- If you do not know enough to test an area well, say that subjective testability is low and seek pairing, a briefing, or a narrower charter. Lack of knowledge is a project fact, not a personal failure.

## Reporting stance

- Report product problems in terms of value loss to a person who matters.
- Report coverage as "coverage with respect to a model", never as a naked percentage.
- Report issues separately from bugs, but do not obsess over taxonomy. If a problem threatens both the product and the project, treat it as both. The goal is awareness, not classification purity.
- Use better questions than "How many passed?" Ask what was examined, what was not, what assumptions were made, and what slowed learning down.

## Anti-patterns

- **NEVER** grade testers by bug count because the number is seductively simple while hiding module dirtiness, luck, and lost coverage. A tester who finds many bugs may have explored narrowly and a tester who finds few may have covered far more risk. Instead evaluate debrief quality, coverage notes, issue awareness, and the relevance of discoveries.
- **NEVER** turn every fixed bug into a permanent scripted regression check because it feels prudent while quietly consuming future coverage budget. Specific fixed bugs often stay fixed; the repeated path mainly proves script stability. Instead add checks only where regression risk is credible or the invariant is cheap and valuable, then vary around the fix during exploration.
- **NEVER** use TBS or session metrics to rank people because numbers look objective while destroying the real purpose of the metrics. Session metrics were designed to reveal interruptions, guide coaching, and reshape charters. Instead use them to ask why learning slowed down and what the project should change.
- **NEVER** accept low observability or controllability as "we'll test harder" because reduced testability gives bugs more time and more opportunities to hide. Instead raise a testability issue and ask for logs, hooks, state control, data access, or simpler setup.
- **NEVER** let issue reporting disappear into tracker bureaucracy because "it's logged" is seductive and often means "it's invisible." Out-of-sight issues normalize deviance and silently reduce coverage. Instead keep issues visible in debriefs, risk lists, or other channels that force acknowledgment.
- **NEVER** let a charter turn into numbered steps because structure feels safe but suppresses the tester's best ideas at the moment they appear. Instead define the mission, note the worry, and preserve room for adaptation.
- **NEVER** treat a green bar as release evidence because green is psychologically narcotic and confirms only the beliefs already encoded in the checks. Instead say what the checks did not notice, what humans explored, and what remains unexamined.

## Mandatory reference loading

- Before designing, reviewing, or coaching exploratory sessions, **READ** `references/sbtm-and-charters.md`.
  Do **NOT** load `references/test-framing-and-reports.md` unless you are also preparing an output for others.
- Before framing bugs, issues, release risk, or debrief narratives, **READ** `references/test-framing-and-reports.md`.
  Do **NOT** load `references/breaking-test-case-addiction.md` unless documentation pressure is part of the problem.
- Before expanding product models or choosing recognition heuristics, **READ** `references/oracles-few-hiccupps.md`.
  Do **NOT** load it just because someone said "quality"; load it when you need better oracles or broader coverage ideas.
- Before replacing script-heavy processes, defending exploratory work to auditors, or deciding what to automate, **READ** `references/breaking-test-case-addiction.md`.
  Do **NOT** load every reference at once; pull only the file that matches the decision you are making.

## Minimal fallback

- If you have less than an hour, run one short survey session.
- Report the top three risks noticed, the biggest unexamined area, and the main testability issue.
- End with the next charter you would run if given another session.
