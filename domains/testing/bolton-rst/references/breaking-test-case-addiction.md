# Breaking the Test Case Addiction

> **Load this when**: someone insists on detailed scripted test cases "because regulation," when a team is being crushed by 100+ page test plans, when arguing about whether to automate the regression suite, or when proposing to replace scripts with charters.

## The myth

The myth: "Detailed, step-by-step, scripted test cases are required for serious software, especially in regulated industries (FDA, ISO 26262, IEC 62304, SOX, banking)."

## What the regulation actually says

The FDA's guidance document on software validation mentions "test case" or "test cases" 30 times. **It never defines what a test case is or how it should be documented.** It says things like:

> "A software product should be challenged with test cases based on its internal structure and with test cases based on its external specification."

If you read "test case" as *an artifact*, this is terrible advice — like saying children should be fed *with recipes* instead of *with food*. If you read "test case" as *a test* or *testing*, it's good advice: challenge the product with tests based on internal and external perspectives.

The pattern across regulations: the regulator wants **evidence that the product was challenged thoroughly and that problems were sought, found, investigated, and addressed.** The regulator does not specify the artifact format. Teams that demand scripted test cases "because FDA" usually have not read the FDA guidance.

This applies almost universally. Read your actual regulation before writing a single test script.

## The James Bach medical-device story

James Bach was hired to assess testing for a Class III medical device — a control box that delivered "Healing Energy" to patients. Insufficient energy is just energy. Too much, or for too long, becomes Hurting Energy or Killing Energy. The cost of a missed bug is human harm.

The company gave testers more than 100 pages of test scripts that looked like:

> Step 1: Press the green button.
> Step 2: Verify the LED illuminates.
> Step 3: Press the red button.
> Step 4: Verify the LED extinguishes.
> ...

James **replaced 50 pages of this with two paragraphs**. The first paragraph was a general protocol:

> "In the test descriptions that follow, the word 'verify' is used to highlight specific items that must be checked. **In addition to those items**, a tester shall, at all times, **be alert for any unexplained or erroneous behavior of the product**. The tester shall bear in mind that, regardless of any specific requirements for any specific test, **there is the overarching general requirement that the product shall not pose an unacceptable risk of harm to the patient**, including any unacceptable risks due to reasonably foreseeable misuse."

The second paragraph was a list of concise test ideas — *what to investigate*, not *what to click*. Where genuine measurement precision mattered (e.g., power output accuracy with statistical confidence intervals), James wrote out the *measurement procedure* in detail. Procedure where it matters; mission elsewhere.

**The auditors accepted it.** Better: the new approach revealed bugs the old scripts had missed for years, because the old scripts trained testers to ignore everything that wasn't on the script.

## Why scripts find fewer bugs

The cognitive trap: a scripted test case is a *decision rule*. By definition it can detect only the problems it was designed to detect. The tester executing the script has been instructed *not to think* — that's what makes the execution repeatable and "objective." But a tester who has been instructed not to think has been instructed not to test.

What scripts produce:
- Detection of regressions in *anticipated* failure modes
- Evidence of "test case execution" for compliance theater
- A false floor of confidence ("we ran 4,000 test cases this week")

What scripts cannot produce:
- Detection of failure modes nobody anticipated
- Insight into product structure or risk
- Tester skill development (the script is the skill, the tester is the executor)

This is exactly the testing/checking distinction. Scripted test cases are *checks dressed as tests*.

## The replacement pattern

When replacing a scripted test case suite, do not delete and do not start from scratch. The pattern:

1. **General protocol** at the top (one paragraph). Spells out the meta-rule: testers must be alert for any unexplained or erroneous behavior, regardless of what the specific items below say. This is the legal anchor that satisfies "evidence of thorough challenge."

2. **Test ideas** instead of test cases. A test idea is one or two sentences that describes *what to investigate*, not *what to click*. Example replacement:
   - **Old**: "1. Open Settings. 2. Click Profile. 3. Click Edit. 4. Change name to 'Test User'. 5. Click Save. 6. Verify name is 'Test User'."
   - **New**: "Investigate the profile-edit flow. Try unusual names (Unicode, very long, RTL, names with control characters), interrupted edits, concurrent edits from another session, and edits while offline. Look for data loss, display problems, security issues, and unexpected coupling with other features."

3. **Detailed procedures only where measurement precision is required.** If the test requires specific calculations (statistical confidence intervals on power output, drug dosage tolerance, financial calculation accuracy), write the procedure in full. Procedure where it matters; mission elsewhere.

4. **Session reports as the evidence trail.** Each tester records what they did, what they found, what they didn't get to, and their TBS percentages. This is the audit evidence — *what was actually done*, not *what a script said someone might have done*.

5. **Traceability matrix** maps requirements to *test ideas* and *session reports*, not to test case IDs. Auditors care that requirements were challenged, not that a particular script was executed.

## When detailed scripts ARE appropriate

Be honest about this. There are cases where the detailed script is the right tool:

- **Calibration procedures**: when the steps must be performed exactly and reproducibly to compare against a known standard.
- **Statistical sampling**: when you need to apply the same procedure to many specimens and aggregate results.
- **Smoke checks**: when you want to confirm that a known set of behaviors did not regress, fast, before deeper testing begins.
- **Regulatory measurement**: when the regulation specifies the procedure (rare, but it does happen — read the actual regulation).

In every other case, a charter and a list of test ideas finds more bugs in less time and produces a better audit trail.

## The pushback you will get and the answer

**"But how will we know testing happened?"**
Session reports with TBS metrics. They show *what was actually done*, which is more honest than "test case 4127 was marked Pass."

**"But how will we estimate?"**
Sessions per charter. After a few sprints you can estimate "this story needs N sessions" with surprising accuracy.

**"But how will we get repeatability?"**
You won't, and you don't want to. Repeatability is a property of *checks*, not of *testing*. Repeatable execution finds repeatable bugs. Variable execution finds variable bugs — and the bugs in production are variable bugs.

**"But the auditor will reject it."**
James Bach has done this with FDA Class III device auditors. The auditors accept it when the protocol paragraph is in place and the session reports are honest. **Read your regulation. Most "the auditor requires X" claims are folklore.**

**"But the testers will just slack off."**
If your testers will slack off without scripts, they will also slack off *with* scripts (and you won't notice because the script will be marked "Pass"). The fix is hiring and training, not paperwork.

## What "up to speed" really means

When asked for testers who are "up to speed quickly," managers usually mean "banging on keys as quickly as possible." That is the wrong metric. *Banging on keys* is checking. The thing that finds bugs is *thinking about the product*, and that takes time to develop. A tester who is slow on the keys but who understands the product, the risks, and the FEW HICCUPPS oracles will find more bugs than a fast typist following a script.

This is why RST emphasizes skill, training, and tacit knowledge over throughput. The Productivity Paradox is real: faster check execution does not translate to better testing, and pretending it does eventually ships a serious bug.
