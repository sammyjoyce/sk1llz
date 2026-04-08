# Charters — Deep Reference

Load this when writing charters from scratch, reviewing someone else's charter library, or converting a risk list into testable missions.

## The format (and why each slot matters)

> *Explore **[target]** with **[resources]** to discover **[information]**.*

- **Target** — the *what*. Must be narrow enough that a single tester can form a mental model of it in the first 10 minutes of the session.
- **Resources** — the *with what*. Tools, data sets, personas, environments, documentation, colleagues available to consult. Forces you to think about testability up front.
- **Information** — the *why*. This is the slot beginners skip. Without it, you are inviting aimless clicking.

## Charter sizing — the craft beginners get wrong

The single biggest charter mistake is bad sizing. Use this decision:

```
Can a skilled tester reach a satisfying stopping point in 60–120 minutes
AND produce a reviewable session sheet from it?
├─ No, much longer → charter is a MISSION, not a charter. Split it.
├─ No, much shorter → you wrote a CHECK. Automate or discard.
└─ Yes → right size.
```

Heuristic: if you can imagine writing three distinct session sheets for the same charter on three different days with three different discoveries, it's the right size. If every session would look nearly identical, it's too small.

## Worked examples

**Too big — don't do this:**
> Explore the checkout flow to find bugs.

Why it fails: no target discipline (which parts of checkout?), no resources (which payment methods? which currencies? which personas?), no information goal (bugs of what kind? usability? security? data integrity?). Five testers would do five incomparable sessions.

**Too small — don't do this:**
> Verify that clicking Save in the profile page stores the updated email address.

Why it fails: single proposition, single assertion, deterministic outcome. This is a *check*. File it under regression and move on. It does not deserve a tester's 90 minutes.

**Right-sized — like this:**
> Explore the *Express Checkout* flow with international credit cards (JCB, UnionPay, Mir) and VPN-masked locales to discover currency-conversion, error-messaging, and timeout issues.

Why it works: one flow, a bounded but non-trivial set of resources, three information categories the tester can pivot between when one dries up. A session will produce surprises.

**Right-sized — another flavour:**
> Explore the password-reset flow under *interruption* (lost tab, expired link, two devices, clock skew) to discover state-reconciliation and audit-logging issues.

Why it works: one feature, one clear failure model (interruption/state), specific information goals. Any of the interruption categories alone would fill the session.

## Generating charters from a risk list

Bach's trick: write a risk list **first**, then translate each high-priority risk into one or more charters.

| Risk (from a product risk list) | Charter derived |
|---|---|
| "New locale code paths may corrupt stored data for pre-existing users" | *Explore the locale-switch flow with legacy user accounts from v2.x to discover data-corruption and migration-rollback issues.* |
| "Auth token refresh may interact badly with background tabs" | *Explore multi-tab sessions under token-expiry boundaries with slow networks to discover session-dropout and silent-logout issues.* |
| "Billing invoices may double-charge during retry" | *Explore the payment-retry and webhook-replay paths with simulated network drops to discover idempotency and ledger-consistency issues.* |

Notice that every charter has a **failure mode** (corruption, dropout, double-charge) baked into the information slot. This is what keeps the session focused without scripting it.

## Six charter anti-patterns (seen in real charter libraries)

1. **"Explore X to find bugs"** — missing the information slot. Always state *what kind of* bugs.
2. **Charters that are actually test-case lists** — bullet points of clicks. If the charter says *"click Save, then click Cancel, then click..."* you wrote a check and pretended it was exploratory.
3. **Charters tied to story IDs only** — *"Charter: JIRA-4281"*. Unreadable without context, unreviewable in a debrief, impossible to prioritise against risk.
4. **Charters for features that don't exist yet** — "explore the new ML ranker" before the ranker is deployed. This is a planning document, not a charter. Rename it.
5. **Repeated charters with a version number** — *"Regression pass v14"*. If you're running the same charter every sprint, it has become a regression protocol. Extract the deterministic parts into automated checks and let the charter *explore the parts that change*.
6. **Charters assigned to "the team"** — a charter is a contract with *one* tester (or a named pair). Team charters have no accountable owner and produce no session sheet.

## Tour heuristics — a library of charter lenses

When a charter feels stale, pick a *tour* as its resource/lens. Each tour biases the tester toward different bugs.

- **Money Tour** — only touch features that *sell* the product. Bugs here hurt revenue directly.
- **Landmark Tour** — hop between the most-advertised features. Bugs here damage brand image.
- **FedEx Tour** — follow one piece of data end-to-end through every stage of the pipeline. Catches data-loss and transformation bugs.
- **Guidebook Tour** — follow the docs/tutorials *exactly*. Catches claims oracle violations and broken onboarding.
- **Bad Neighborhood Tour** — focus on historically buggy areas. Pesticide-paradox resistant if you *also* defocus on technique.
- **Garbage Tour** — look for dead code, unused features, orphaned endpoints. Security bugs hide here.
- **Museum Tour** — test legacy features nobody touches. When they break, nobody notices until a big customer does.
- **Back Alley Tour** — least-used features. Low traffic = low check coverage = high bug density.
- **All-Nighter Tour** — leave it running for hours. Catches leaks, timeouts, scheduled-task bugs.
- **Anti-tour (Saboteur)** — adopt the mindset of a user trying to break the product on purpose. Catches input-validation and state-machine bugs.

**Key rule:** every tour is a *resource* slot on a charter, never the whole charter. *"Explore the reporting module using a FedEx tour on customer-purchase records to discover data-transformation and aggregation-bucket issues."*

## Charter review checklist (use before adding to a library)

- [ ] Target is narrower than "a whole feature area."
- [ ] Resources are named specifically (not "test data" — *which* test data).
- [ ] Information slot names a failure mode or question, not "bugs."
- [ ] One tester can finish in 60–120 minutes.
- [ ] A stakeholder can be named who will act on the result.
- [ ] The charter is not a disguised test case list.
- [ ] The charter is not a disguised check (deterministic assertion).
- [ ] The charter is written in language the whole team can read, not internal jargon.
