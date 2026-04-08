---
name: hashicorp
description: Design workflow-first platform engineering, Terraform/Vault/Consul/Nomad operating models, and self-service infrastructure boundaries in a HashiCorp style. Use when defining golden paths, workspace and state topology, secrets or service-discovery architecture, scheduler trade-offs, or migration plans that must survive technology swaps. Trigger keywords: workflow-first, immutability, Terraform workspace, remote state, Vault token, Consul stale read, Nomad binpack, platform team, self-service, golden path.
---

# HashiCorp

If your golden path leaks cloud IDs, auth backends, cluster names, or scheduler
classes, you standardized an implementation, not a workflow, and you just
created migration debt for users.

## Core Lens

Before designing anything, ask yourself:

- If AWS became Azure or Nomad became Kubernetes, what user workflow must remain unchanged?
- Which boundary is carrying blast radius: a workspace, a state file, a token, a cluster, or a team?
- Where are you hiding mutation behind a supposedly declarative interface?
- Which reads can be stale for availability, and which must be consistent?
- Are you composing single API objects, or are you smuggling orchestration into the wrong layer?

HashiCorp-style systems age well when the workflow stays stable while the
implementation underneath is free to change.

## Design Rules That Matter

- Standardize the contract, not the substrate. Developers should declare intent
  such as runtime, policy, secrets, and connectivity without caring whether the
  backend is EC2, containers, Nomad, or something else later.
- Put choreography in modules and workflows, not in provider internals. A
  Terraform provider should stay close to one API or problem domain, and a
  resource should represent one API object. If one "helpful" resource secretly
  manages several subsystems, drift and destroy blast radius become opaque.
- Prefer codified replacement over mutable repair, but do not confuse purity
  with operational competence. In an outage, a tactical mutable fix can be the
  right move only if you immediately turn it into a replacement path before the
  next scale event.
- Separate trust boundaries before you separate directories. State, secret, and
  service-discovery boundaries usually dominate repo layout decisions.

## Decision Tree

| Situation | Default move | Escalate when | Wrong move |
|---|---|---|---|
| Terraform scope keeps growing | Split by blast radius, ownership, and change frequency | One plan is slow enough that review quality drops or one failure blocks unrelated teams | One mega-workspace because "one plan is easier" |
| Cross-workspace data sharing | Publish minimal outputs with explicit consumers | A consumer needs many internals, which signals a bad boundary | Broad `terraform_remote_state` access |
| Secrets for bursty short-lived workloads | Batch tokens | You need renewal, child tokens, or revocation semantics | Service tokens everywhere |
| Service discovery under load | Allow stale reads for discovery paths | A caller truly needs linearizable freshness | Forcing consistent reads cluster-wide |
| Nomad placement choice | Keep cluster default `binpack` | Specific workloads need failure-domain spread more than density | Flipping the whole cluster to `spread` |
| Vault Raft membership change | Add or replace nodes gradually with Autopilot healthy | Snapshot load time or HSM latency is unusually high | Aggressive dead-node cleanup thresholds |

## Terraform and HCP Heuristics

- The default Terraform graph walker processes up to 10 nodes concurrently.
  That number is a machine-protection semaphore, not a generic rate-limit cure.
  Do not reach for lower `-parallelism` first when providers are already doing
  backoff and retry.
- If you need to reduce concurrency, fix the reason precisely:
  - Provider API throttling: confirm provider retry behavior first.
  - Agent or runner exhaustion: reduce `-parallelism` or resize the runner.
  - Giant plans: split the workspace before tuning flags.
- In Terraform Enterprise and HCP Terraform, do not confuse run concurrency with
  per-run graph parallelism. Similar names make operators treat them as one
  knob, which causes silent non-fixes.
- In current HCP Terraform agent-era runs, `TFE_PARALLELISM` is not a reliable
  substitute for CLI `-parallelism`. If you must change graph concurrency, set
  `TF_CLI_ARGS_plan` and `TF_CLI_ARGS_apply` explicitly.
- Remote state is a trust boundary, not just a convenience channel. If a team
  only needs five values, publish five outputs; do not make them eligible to
  consume the entire producer state forever.
- When migrating workspace state sharing, renamed workspaces and organizations
  are a trap: the relationship may not be detected until both producer and
  consumer run again.
- Very large state files have second-order effects long before Terraform
  outright breaks. On Terraform Enterprise, cached state data hitting the Redis
  client limit around 512 MB creates failure modes that look like generic UI or
  planning instability. Split before you are anywhere near that class of size.
- Do not branch important behavior on `terraform.workspace` in remote-backend
  automation unless you have proved how that backend populates it. In some
  remote execution patterns it resolves to `default`, which silently routes
  logic to the wrong environment.

## Vault Heuristics

- For integrated storage, the production sweet spot is usually 5 nodes, not 7+.
  Odd numbers matter for quorum; beyond 5 you often buy write coordination cost
  faster than you buy useful fault tolerance.
- Autopilot defaults are safer than many "optimizations." Keep
  `dead_server_last_contact_threshold` high; the default is 24h for a reason.
  If you shorten it aggressively, a slow snapshot load or HSM response can make
  healthy joining nodes look dead and get them removed mid-recovery.
- `last_contact_threshold` defaults to 10s and `max_trailing_logs` to 1000.
  Changing them without evidence is usually a self-inflicted stability problem.
  Raise `max_trailing_logs` only when you have proven high write load is slowing
  voter promotion.
- Match token type to lifecycle:
  - Batch tokens: cheapest for high-volume, short-lived, read-mostly clients.
  - Service or periodic tokens: required when you need renewal, revocation,
    child tokens, or durable service identity.
- If lease count is exploding, the real issue is often issuance pattern, not
  the expiration manager. Fix token or secret churn before you start tuning
  cleanup behavior.

## Consul and Nomad Heuristics

- Consul service discovery should usually bias toward availability. Stale reads
  are cheaper because followers can answer them; forcing consistency across the
  board pushes load to the leader and can turn a busy control plane into an
  outage amplifier.
- If you use blocking queries, treat index regression as normal after snapshots,
  restarts, or leadership changes. Reset the index and continue instead of
  assuming data corruption.
- Use blocking-query `wait` values in the rough 2-5 minute range unless you have
  a measured reason not to. Short waits create avoidable request churn; invalid
  carried-forward indexes can create non-blocking hot loops that look like
  "Consul is slow" when the client is the problem.
- Nomad's default scheduler algorithm is `binpack` for a reason: it preserves
  free nodes for larger later allocations. Cluster-wide `spread` feels safer but
  often creates fragmentation and worse headroom.
- Use `spread` at the job, group, or node-pool boundary when failure-domain
  isolation matters more than density. Do not pay the cost everywhere for one
  workload's requirement.
- The `spread` stanza is not the same thing as the cluster `spread` algorithm.
  It can also be much more expensive: without `spread`, service jobs score about
  `log2(nodes)` feasible nodes with a floor of 2; with `spread`, scoring can
  rise to task-group count with a cap of 100 per allocation.
- Scheduler dry-runs are advisory. `nomad plan` can miss placement outcomes that
  later change due to preemption, quotas, or moving capacity. Treat it as a
  preview, not a guarantee.

## NEVER Do This

- NEVER build a platform abstraction around today's orchestrator because it is
  seductive to expose product-specific features early. The consequence is that a
  future migration becomes a user-facing rewrite. Instead standardize the intent
  developers express and keep the substrate behind the contract.
- NEVER hide multiple API objects behind one Terraform resource because it feels
  ergonomic. The non-obvious consequence is import pain, drift ambiguity, and
  destructive operations with unclear blast radius. Instead keep provider
  resources close to the API and compose higher-order behavior in modules.
- NEVER leave remote state broadly readable because "it is only internal." The
  seductive part is speed. The consequence is permanent over-sharing of values
  and accidental coupling through state internals. Instead publish minimal
  outputs and explicitly grant consumers.
- NEVER shorten Vault Autopilot cleanup thresholds to make failover "faster"
  unless you have measured snapshot and HSM latency. The consequence is healthy
  nodes being removed during recovery. Instead keep the threshold conservative
  and tune only from observed timings.
- NEVER use service tokens for massive short-lived fan-out because one token
  type everywhere feels simpler. The consequence is unnecessary storage,
  renewal, and revocation load. Instead use batch tokens for ephemeral clients
  and reserve service or periodic tokens for durable identities.
- NEVER force Consul consistency cluster-wide because stale data sounds scary.
  The non-obvious consequence is leader overload right when the cluster is under
  stress. Instead use stale reads by default and request consistency only for
  the narrow calls that truly need it.
- NEVER flip Nomad globally to `spread` to fix one team's resilience issue
  because it is an easy one-line cluster change. The consequence is fragmentation
  and reduced scheduling efficiency for everyone else. Instead scope spread to
  the affected jobs or pools.
- NEVER keep one Terraform workspace until it becomes a political problem. The
  seductive part is centralized visibility. The consequence is slower plans,
  lower review quality, coupled failures, and state growth that becomes an
  operational incident. Instead split by ownership, volatility, and blast radius
  while the change is still boring.

## Fallbacks

- If the workflow-first answer and the product-first answer disagree, choose the
  workflow unless the product shortcut is explicitly temporary and has an exit.
- If a boundary is unclear, draw the trust boundary first. State sharing,
  secret scope, and scheduler failure domains usually settle the design.
- If teams are arguing about tool choice, rewrite the debate as a workflow
  contract question: what must users declare, what must the platform guarantee,
  and what can change invisibly underneath?

## Loading Guidance

- MANDATORY before changing Terraform provider or provider-like abstractions:
  read the official HashiCorp provider design principles. The key question is
  whether the abstraction belongs in a provider resource or in a module.
- MANDATORY before touching Terraform state topology, remote state sharing, or
  workspace fan-out: read the current Terraform and HCP Terraform docs for
  state-sharing behavior in the exact product tier you are operating.
- MANDATORY before tuning Vault integrated storage or Autopilot: read the
  version-matched Vault Autopilot documentation. Threshold defaults and
  semantics matter.
- Do NOT load Vault, Consul, or Nomad product detail for a purely Terraform
  module-shape task. Do NOT load Terraform provider internals for an org-design
  discussion about golden paths.
