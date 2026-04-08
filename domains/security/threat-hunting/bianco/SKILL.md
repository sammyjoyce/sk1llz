---
name: bianco-pyramid-of-pain
namespace: bianco-pyramid-of-pain
description: Threat-hunting strategy and detection-engineering guidance for using David Bianco's Pyramid of Pain, Hunting Maturity Model (HM0-HM4), and modern Summiting-the-Pyramid/Ambiguous-Techniques methods to choose hypotheses, score robustness, and turn hunts into durable detections. Use when planning hunts, deciding whether a signal belongs at IOC/tool/TTP level, triaging living-off-the-land activity, measuring hunting maturity, or operationalizing successful hunts. Triggers: pyramid of pain, hunting maturity model, HM0, HM1, HM2, HM3, HM4, summiting the pyramid, ambiguous techniques, TTP hunting, direct correlation, loose correlation, hunt-to-detection.
---

# Hunt for attacker cost, not IOC volume

**MANDATORY**: Before maturity scoring, backlog prioritization, or detection promotion, READ `references/pyramid_of_pain.md`.
Do NOT load `scripts/pyramid_analyzer.py` unless the strategy is already frozen and you are executing a batch scoring task.
Do NOT load `references/pyramid_of_pain.md` just to write platform-specific rule syntax; once the plan and promotion decision are frozen, this skill is done.

This skill is for choosing what to hunt, how much confidence to demand, and when a hunt is mature enough to become a detection. It is not for SIEM-specific query syntax.

## Before doing X, ask yourself...

- Before starting a hunt, ask: am I trying to discover exposure, prove attacker presence, or graduate a known pattern into automation? Each goal tolerates different noise.
- Before accepting an indicator as "high pain," ask: what exactly must the adversary rebuild to evade this signal: a byte, an IP, a domain, a tool configuration, one implementation, or the technique itself?
- Before declaring an admin-like action suspicious, ask: is this an ambiguous technique that needs context rather than a standalone alert?
- Before tuning a detector, ask: am I improving precision with defender-controlled context, or with attacker-controlled values that become the next evasion path?
- Before claiming maturity, ask: are hunts creating reusable procedures and detections, or just producing one-off stories?

## The working model

Treat the Pyramid as a defender economics model:

- Levels `hash -> IP -> domain` are disposable. Use them for rapid scoping, retro-hunting, and temporary containment, not as the center of a hunt program.
- Domains deserve slightly more respect than IPs because registration, propagation, and reputation-building can slow phishing or callback infrastructure by days or weeks. That is still not durable enough to anchor a mature hunt program.
- Artifacts and tools are where durable wins often start, especially when the observable is inside your boundary or tied to pre-existing tooling the attacker does not control.
- Modern Summiting-the-Pyramid work matters because many rules that look "behavioral" collapse to low pain once you score the actual observables. Robustness is five levels: `1 ephemeral`, `2 adversary-brought/outside boundary`, `3 pre-existing tools or inside boundary`, `4 core to some implementations`, `5 core to the technique`.

The practical rule: score the observable, not the marketing label on the rule.

## Freedom calibration

- Use high freedom when choosing hypotheses and deciding which attacker problem matters most.
- Use low freedom when scoring observables, choosing correlation style, or promoting a hunt into a detection. Those mistakes create long-lived blind spots or alert debt.

## Maturity floor

Use HMM honestly:

- Bianco explicitly recommends `HMM2` as the realistic starting point. If you lack centralized network, host, and application data plus routine IOC/intel use, you are not ready for reliable top-of-pyramid hunting.
- `HMM3` starts when analysts follow recurring hunt procedures on a schedule and designated hunters or a rotation exist.
- `HMM4` starts only when successful hunts are systematically automated or used to improve alerting, and the team has a scaling method for the procedures it keeps inventing.

If you fail an HMM gate, stop chasing sophistication and fix the prerequisite.

## Procedure: run a hunt cycle without fooling yourself

1. Set the hunt class.
   - `Seed hunt`: low-pain indicators allowed, objective is scoping or containment.
   - `Durable hunt`: aim for Level 3-5 observables and write down what attacker change would be required to evade them.
   - `Ambiguous-technique hunt`: require context before paging anyone.

2. Pick the right context model.
   - Use `peripheral context` for pre-compromise or sector-specific targeting.
   - Use `chain context` when co-occurring techniques establish intent better than any single event.
   - Use `technique context` when the behavior is admin-like or LOTL. Force the question set `Who / What / When / Where`.

3. Choose correlation style deliberately.
   - Use `direct correlation` only when the actions are dependent on one another.
   - Use `loose correlation` for discovery, scripting, and other ambiguous activity. Start with `>=3` related analytics on the same user, host, or asset group inside one working window and tune from there; do not force brittle sequence logic where no dependency exists.

4. Score the observables before writing production logic.
   - Prefer `K` over `U` over `A` host telemetry when you have a choice.
   - Prefer network `header visibility` over `payload visibility` when encryption or attacker-controlled obfuscation can erase the payload signal.
   - Break every rule into observables. In STP, an `AND` chain collapses to the lowest-scoring observable. One cheap attacker-controlled term can drag an impressive-looking rule down to Level 1.
   - Look for a `spanning set`: the smallest set of observables that still fires across implementations. If you cannot describe the spanning set, you probably do not understand the true pain level yet.

5. Promote only after operational proof.
   - Baseline against roughly `30 days` of data.
   - Run in observational mode for about `1 week` before automating response.
   - Track the false positives created by transitioned hunts.
   - Treat `1 successful hunt -> 1 new analytic, rule improvement, or at least preserved IOC` as the default expectation. If a hunt produces nothing reusable, challenge whether it was mature enough to count as "successful."

6. Keep the backlog honest.
   - Hunt for TTPs you do not already catch well. Leave previously solved patterns to automation maintenance.
   - If a sprint spends more time recovering missing fields than testing hypotheses, reclassify it as telemetry engineering and stop calling it a hunt.

## Decision tree

If you only have hashes, IPs, or domains:
- Use them to scope blast radius or pivot to richer behavior.
- If the hunt ends there, record it as containment support, not durable detection progress.

If the signal is a pre-existing admin tool, LOLBin, or normal-looking network action:
- Assume ambiguity.
- Require context before alerting.
- Prefer loose correlation unless you can prove the steps are causally dependent.

If the observable is inside your boundary or tied to invariant platform behavior:
- Try to lift it to Level 3-5 and build from there.
- Add precision with defender-controlled context, not attacker-controlled strings.

If the candidate rule is robust but noisy:
- Keep the robust core.
- Improve accuracy with surrounding context or chaining.
- Do not replace the robust core with easy-to-evade low-level filters.

If telemetry is missing:
- Open a logging-gap task immediately.
- Do not fake maturity with a thinner hypothesis.

## Anti-patterns that cost real programs

- NEVER call a rule "top-of-pyramid" because it mentions an ATT&CK technique. That is seductive because the label sounds strategic. The consequence is false confidence: the rule can still be Level 1 if one attacker-controlled field anchors the condition. Instead decompose the rule and score the weakest observable first.
- NEVER alert on ambiguous admin behavior in isolation because it feels like proactive coverage. The consequence is durable analyst fatigue and defenders training themselves to ignore the exact LOTL activity they needed to see. Instead add peripheral, chain, or `Who/What/When/Where` context first.
- NEVER force strict sequence logic onto discovery-style hunts because it feels mathematically clean. The consequence is expensive engineering and missed adversary activity when real campaigns vary order or distribute steps. Instead use direct correlation for dependent actions and loose correlation for converging patterns.
- NEVER tune precision with attacker-controlled exclusions because it is the fastest way to make a chart look better. The consequence is that the exclusion becomes an evasion recipe, and in STP the cheap term may govern the rule. Instead filter with inside-boundary context such as sanctioned admin hosts, approved maintenance windows, stable parent lineage, or privileged-role expectations.
- NEVER judge a new hunting program by the first spike in incidents found because leaders expect hunting to lower the graph immediately. The consequence is killing the program during the normal startup bump, when hunting is finally surfacing old compromise, logging gaps, and insecure practices. Instead watch dwell time, detection gaps filled, logging gaps corrected, and transitioned hunts.
- NEVER let CTI feeds define the backlog because ingesting IOCs is easy to automate and easy to report. The consequence is a program optimized for expiration dates instead of attacker cost. Instead use feeds as seeds, then climb toward inside-boundary artifacts, spanning sets, and behavior.
- NEVER assume "tool detection" is automatically high pain because tools sit high on the diagram and tool names sound meaningful. The consequence is overrating detections tied to adversary-brought frameworks, malleable profiles, or easily swapped kits. Instead prefer observables tied to pre-existing platform behavior, inside-boundary dependencies, or technique-spanning sets.
- NEVER hand-triage the bottom of the pyramid at scale because it feels safer than behavioral work. The consequence is that senior analyst time gets burned on disposable infrastructure while robust detection never improves. Instead automate most hash/IP/domain handling and reserve human cycles for Level 3-5 reasoning.

## Metrics and thresholds that matter

- Use `HMM2` as the realistic entry point for serious hunting.
- On a mature team, the expected ratio is roughly `1:1` between successful hunts and some reusable output: new analytic, improved rule, or at minimum a preserved indicator.
- Baseline prospective detections over about `30 days`, then run them observationally for about `1 week` before enabling automated response or hard paging.
- A practical portfolio target is to keep roughly `60%` of engineered coverage in the top three levels and automate about `80-90%` of hash/IP/domain handling; if humans are still spending more than about `20%` of hunt time there, the program is upside down.
- Judge hunts by attacker cost or defender visibility moved: dwell time, detection gaps filled, logging gaps corrected, false positives on transitioned hunts, and newly gained visibility. Raw hunt count is management theater.

## Edge cases and fallback rules

- Cloud, identity, and SaaS attacks often make infrastructure indicators nearly useless. Treat valid-credential or token abuse as an ambiguous-technique problem and lean on context, not malware assumptions.
- Network detections age badly when the useful observable lives only in payload. Re-score after protocol changes, TLS adoption, product upgrades, or sensor changes.
- STP assumes trusted telemetry. If attackers can suppress, delay, or blind the sensor, the nominal score overstates real pain; compensate with sensor hardening or a second source.
- Highly robust detections can still be unusable if the behavior is common in your environment. Robustness and accuracy are separate variables; do not trade one away blindly.
- If the hunt returns zero results, ask whether the behavior is already covered by automation or whether you chose the wrong context model.
- If the hunt returns too much noise, do not immediately drop down the pyramid. First ask whether a spanning set exists, whether context is missing, or whether the logic should remain a hunt-only analytic instead of a production alert.

## Stop conditions

- Stop calling the work "threat hunting" if the main human action is just remediating something a tool already found.
- Stop promoting hunts when telemetry gaps, not adversary behavior, are driving the results.
- Stop adding clever logic if the weakest observable is still cheap to evade.
- Stop claiming maturity growth if procedures are not being published, reused, or automated.
