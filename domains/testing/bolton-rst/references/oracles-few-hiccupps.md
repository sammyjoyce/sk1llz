# FEW HICCUPPS — Bolton's Oracle Framework

> **Load this when**: you need to recognize a problem and you don't have a spec, or you need to defend a bug report by naming the oracle behind it, or you're modeling a product to design tests.

## What an oracle actually is (and isn't)

An **oracle** is a *principle by which we recognize a problem* — not a mechanism that tells you the right answer. Bolton is precise: "all oracles are heuristic, but not all heuristics are oracles." Every oracle is fallible. Every oracle can mislead. The job of the oracle is to give you a *reason to suspect* a bug, which you must then investigate and defend.

Two modes of use that practitioners conflate:

1. **Generative**: use the oracle list to *invent* test ideas before testing. ("What would inconsistency-with-history look like for this product?")
2. **Retrospective**: use the oracle list to *justify* a bug report after observing something. ("This is a bug because it is inconsistent with the user's stated purpose.")

Bolton's view: you *must* use them retrospectively. The credibility of a tester rests on being able to explain *why* an observed thing is a problem, not just that it "feels wrong."

## The mnemonic

> *"When we're testing, actively seeking problems in a product, it's because we desire FEW HICCUPPS."*

Note the verb: **desire**, not expect. Bolton revised this from "expect" because expectations trap testers — you expect what you've been told, and you miss what you weren't told to look for.

| Letter | Principle | Inconsistency means a problem when... |
|---|---|---|
| **F** | **Familiar** problems | The product exhibits a pattern of failure we've seen in other systems before. *Bias warning*: turns into "I always test for X" if not balanced. |
| **E** | **Explainability** | We cannot explain to ourselves or others how the product's behavior makes sense. If you can't explain it, it's at least an issue. |
| **W** | **World** | The product is inconsistent with how things actually work in the physical, legal, social, or domain world it claims to model. |
| **H** | **History** | The product is inconsistent with its own past behavior — *unless* the change is intentional (a fix, a redesign). |
| **I** | **Image** | The product is inconsistent with the image the organization wants to project (brand, professionalism, tone). |
| **C** | **Comparable products** | The product is inconsistent with similar products users already know. Useful, but: similar ≠ identical, contexts differ. |
| **C** | **Claims** | The product is inconsistent with what someone said it would do — docs, marketing, tooltips, error messages, the spec. |
| **U** | **User expectations** | The product violates what a reasonable user would expect based on the user's intentions and needs. |
| **P** | **Product** (self-consistency) | One part of the product is inconsistent with another part. Different paths to the same result give different answers. |
| **P** | **Purpose** | The product is inconsistent with the *designer's or builder's* explicit or implicit intentions. (See below — this is *not* the same as User Expectations.) |
| **S** | **Standards** | The product is inconsistent with relevant external standards (HTTP, ISO, RFC, accessibility guidelines). |
| **S** | **Statutes** | The product is inconsistent with the law — GDPR, HIPAA, accessibility law, financial regulation. |

## The Purpose vs User Expectations distinction (Bolton, in his own words)

This trips up almost everyone learning FEW HICCUPPS. From Bolton's own forum reply:

> "Purpose refers to the designer's or builder's explicit or implicit purposes for the product or feature. User desires refer to users' intentions and needs — *especially the intentions of forgotten users*. These sets of intentions may not be the same."

Bolton's example: a designer intends an easy-to-use mouse interface. A keyboard-only user (forgotten in the design) is annoyed at having to reach for the mouse. The product satisfies *Purpose* and violates *User Expectations* simultaneously. Both are valid bugs.

And the difference between Purpose and Claims:
> "Claims can come from different people, so claims may conflict when people have differing agendas. The statement of a claim may differ from the intention of the person making the claim — a misinterpretation, or something outright erroneous."

Bolton tells the story of a medical device whose spec was written by a non-native English speaker who habitually omitted the word "not." The spec said the opposite of what the designers intended. The product was consistent with the *Claims* and inconsistent with the *Purpose*. That's the kind of bug only a tester applying multiple oracles will catch.

## The crucial heuristic warning

None of these principles is a *rule*. Bolton:

> "We want the product to be inconsistent with its history if there was a bug and we've fixed it. We want the product to be inconsistent with our image if we want to change our image."

So when you observe an inconsistency, the second question is *should* this be consistent? Sometimes the bug is the consistency. Sometimes the change *is* the feature.

## SFDIPOT (San Francisco Depot) — modeling the *product* you'll apply oracles to

FEW HICCUPPS tells you how to recognize problems. SFDIPOT (from Bach's Heuristic Test Strategy Model) tells you what *parts of the product* to think about when designing tests:

- **S**tructure — what the product is made of (code, files, docs, hardware)
- **F**unctions — what the product does
- **D**ata — what it processes, stores, transforms (inputs, outputs, persistent state)
- **I**nterfaces — how it connects to the world (APIs, GUIs, files, network, humans)
- **P**latform — what it depends on (OS, browser, runtime, hardware, services)
- **O**perations — how people use it, in what scenarios, in what environments
- **T**ime — how time affects it (concurrency, ordering, durations, time of day, timezone, scheduling, time-since-last-X)

The deep practitioner move: use SFDIPOT *during the kickoff conversation with developers and PMs*, not in a test plan document. Each letter generates questions you didn't know to ask. The product owner usually realizes mid-meeting that they hadn't considered Time. That moment is more valuable than any test case you'll write later.

## When to skip parts of FEW HICCUPPS

You're allowed. Bolton encourages it. If the product has no marketing image (internal tool), skip Image. If there are no relevant statutes, skip Statutes. The mnemonic is a reminder, not a checklist. Practitioners pick the 4–6 oracles most relevant to their context and apply them deeply rather than applying all 12 superficially.
