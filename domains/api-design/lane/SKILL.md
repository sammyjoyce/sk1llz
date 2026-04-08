---
name: lane-api-evangelist
description: >
  Apply Kin Lane's API Evangelist approach to API design, governance, cataloging,
  and public API experience. Use when designing or auditing HTTP APIs, standing
  up API governance, mapping an enterprise API landscape, defining deprecation or
  versioning policy, evaluating 3rd-party APIs for consumption, or turning an
  API into a discoverable product surface. Trigger keywords: OpenAPI, APIs.json,
  API governance, API catalog, public API portal, developer experience,
  deprecation policy, versioning strategy, Spectral, API survey, API sprawl,
  API product, 3rd-party API, machine-readable metadata.
---

# Lane API Evangelist

Kin Lane's useful contribution is not "use OpenAPI." It is that API work is
half contract design and half political inventory management. The spec is only
valuable when it helps humans discover, trust, onboard, change, and support the
API over time.

## Route First

| Situation | Do this first | Do NOT load |
|---|---|---|
| New public or partner HTTP API | MANDATORY: read `references/openapi-contract-template.md` | `references/spectral-governance.md` unless you are also defining rules |
| Governance rollout, lint rules, CI policy, multi-team consistency audit | MANDATORY: read `references/spectral-governance.md` | `references/openapi-contract-template.md` unless you need an example contract |
| Enterprise API sprawl, merger, platform reset, catalog cleanup | Read neither reference first; inventory the landscape before prescribing standards | Both references until the inventory exists |
| Evaluating a vendor or 3rd-party API for consumption | Read neither reference first; create or acquire an OpenAPI and assess consumer fitness | Both references until the dependency is mapped |
| Deprecation, sunset, or versioning decision | Start from usage evidence and consumer impact, not from the spec | Both references unless you are editing contract or governance artifacts |

## Default Stance

- OpenAPI is the technical surface. APIs.json is the business surface. If you
  only publish the OpenAPI, consumers still cannot discover pricing, terms,
  support, status, roadmap, or deprecation policy programmatically.
- Map before reforming. If the organization cannot say what APIs exist, who owns
  them, and which suppliers it depends on, a new style guide is theater.
- Govern for consistency before purity. Lane explicitly tolerates patterns he
  would not call ideal if they are consistent enough to reduce consumer
  surprise.
- Governance is everywhere: central policy, federated team execution, existing
  IDEs and pipelines, incremental rollout, and no final "done" state.
- A produced API is not a finished API until it is consumed. Design, governance,
  support, and change policy should all be evaluated from the consumer side.

## Before You Act, Ask Yourself

### Before designing a new API

- Is this for public or partner humans, or only controlled internal automation?
  Lane's versioning and onboarding defaults change when real people debug with
  curl, spreadsheets, and docs rather than generated clients.
- What must be machine-readable besides endpoints? If you cannot point to
  machine-readable pricing, rate limits, support, and legal terms, you have a
  protocol surface but not a product surface.
- Which business capability does each tag represent? Tags are inventory labels
  for your digital warehouse, not reflections of controller names or package
  layout.

### Before writing governance rules

- Am I writing a rule because the business needs the outcome, or because the
  linter can detect it? Lane's policy order is policy -> rule -> guidance ->
  provenance, not rule first and rationale later.
- Can the team challenge or refine this rule? Rules without discussion links,
  rationale, or ownership become compliance theater.
- Will this rule teach, not just punish? Every blocking rule should have linked
  guidance, and often an informational "good job" counterpart.

### Before standardizing an enterprise

- Do we have a survey repo with OpenAPI, JSON Schema, APIs.json, rules,
  generated docs, tooling inventory, and team ownership? If not, stop trying to
  redesign the future and map the present.
- Are we covering enough of the landscape to reason from facts? Lane's bar is
  roughly 90%+ of produced and consumed APIs represented by machine-readable
  artifacts, with about 80%+ of observed patterns and anti-patterns captured as
  rules before major governance decisions.

### Before removing or versioning anything

- Which API keys, teams, or contracts are still using this? A low-request
  consumer can still be economically critical.
- Is the friction of a "more correct" design higher than the friction of an
  explicit, boring design? Lane moved from defending header-based versioning to
  favoring path-based major versions because field reality beat theory.
- What is the removal class? Lane's practical floor is roughly 3 months for a
  field, 6 months for an endpoint, and 12+ months for a major-version sunset;
  removed surfaces should return `410 Gone`, not an ambiguous `404`.

## Working Modes

### 1. Survey Before Strategy

Use this when the landscape is messy, political, or unknown.

1. Create or pick one source-of-truth repo for the survey.
2. Gather or generate rough OpenAPI for every HTTP API you can find; do not
   wait for perfect completeness.
3. Add team ownership, tooling inventory, and generated docs.
4. Assemble APIs.json so the portfolio includes support, pricing, terms,
   deprecation policy, rate limits, status, and roadmap where available.
5. Only then derive rules from the patterns and anti-patterns that are already
   present.

This is deliberately boring. Lane's advice in sprawl is "do not do anything
new; map what you have."

### 2. Design a Public or Partner API

- Write the contract so product, legal, and support can review it, not just
  engineers.
- Put the major API version where humans can see it when consumers are manual or
  mixed-skill. Lane's 2024-10-30 versioning note cites thousands of evaluated
  APIs and five of seven style guides favoring URI/path major-version
  visibility, despite header-based versioning being the theoretically purer
  answer.
- Keep one externally visible API version per contract file. Mixing v1 and v2
  in one artifact invites silent `$ref` reuse and misleading change reviews.
- Keep API version and `info.version` separate in your head. The first is
  consumer change management; the second is document versioning.
- Treat `operationId` as a code generation contract. Once published, renaming it
  is often more disruptive than renaming a field.
- Separate input and output models when generator behavior around `readOnly` and
  `writeOnly` is ambiguous.

### 3. Roll Out Governance

- Start with 12 or fewer low-drama rules.
- Run new rules as warnings until they show zero false positives for four
  consecutive weeks.
- Group rules by policy and lifecycle stage, not by OpenAPI object type.
- Publish the "why" with every rule. If the rationale is weak, delete the rule.
- Prefer existing editor, CLI, CI, and gateway touchpoints over a new governance
  platform nobody will open.

### 4. Govern 3rd-Party APIs You Consume

- Do not judge vendor APIs by whether they match your internal style guide.
  Judge whether they are complete enough to onboard, authenticate, test,
  paginate, recover from errors, and manage change safely.
- If no usable OpenAPI exists, generate one from docs or traffic and assess
  that. Consumer governance is about dependency fitness, not ideological purity.
- Missing support, terms, pricing, rate limits, or deprecation policy is a real
  integration risk even when the endpoint definitions look clean.

## Lane-Specific Heuristics

- If a governance conversation begins with "which linter rules should we add?",
  move it up one level and ask which business policy is missing.
- If your API portal looks good but ownership data is weak, you have marketing,
  not operations.
- If a vendor dependency cannot be represented as a machine-readable contract,
  your supply chain is running on screenshots and tribal memory.
- If the only versioning argument being made is "REST people say headers," you
  are ignoring the onboarding and debugging cost paid by actual consumers.
- If default tool rules say every operation gets exactly one tag, challenge that
  default. Lane explicitly questioned `operation-singular-tag` as a default:
  tagging strategy should match how consumers discover capabilities.

## NEVER Do These

- NEVER launch governance with a huge ruleset because it feels comprehensive.
  The seductive part is that every annoyance can be encoded immediately. The
  consequence is false positives, trust collapse, and teams optimizing for
  lint-passing theater. Instead start with a tiny baseline and promote slowly.
- NEVER standardize the future before mapping the present because greenfield
  governance looks cleaner than enterprise archaeology. The consequence is that
  shadow APIs, supplier dependencies, and real ownership never enter the model.
  Instead survey first and let rules emerge from observed patterns.
- NEVER treat a generated code-first spec as the finished contract because it is
  fast and looks objective. The consequence is leaked implementation names,
  missing business/legal/support metadata, and a contract nobody outside the
  implementation team reviewed. Instead separate contract design from code.
- NEVER govern only APIs you produce because those are the ones you control. The
  seductive lie is that supplier risk is "someone else's problem." The
  consequence is blind spots in your digital supply chain. Instead apply a
  consumer-fitness baseline to every critical 3rd-party API.
- NEVER force header-based versioning on human integrators because it sounds more
  correct. The consequence is higher debugging and support cost, with consumers
  missing the version boundary entirely. Instead use explicit path major
  versions unless the audience is sophisticated enough for negotiated versions.
- NEVER ship a rule without guidance, provenance, and feedback paths because a
  YAML rule looks self-explanatory. The consequence is that future teams cannot
  tell principle from preference. Instead make every rule contestable and
  traceable.

## Fallbacks

| If this happens | Then do this |
|---|---|
| Stakeholders will not review YAML | Run a mock server and review the API behavior plus docs, then sync the contract |
| Teams resist CI blocking | Put rules in the IDE first; let teams learn before you enforce |
| You inherit dozens of inconsistent APIs | Standardize inventory and ownership first, not naming minutiae |
| A vendor has docs but no contract | Generate a draft OpenAPI, then test consumer-critical flows against reality |
| Multiple versions are mixed in one contract | Split the artifacts before discussing breaking changes |
| Deprecation timing is disputed | Pull usage by API key for at least 30 days and decide with business stakeholders, not only engineering |
