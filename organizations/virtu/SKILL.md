---
name: virtu-market-microstructure
description: "Design or review equity execution systems in the style of Virtu/ITG: smart order routing, queue-position models, dark and conditional routing, implementation shortfall control, markout-driven TCA, and anti-toxicity logic for fragmented markets. Use when the task mentions Virtu, ITG, POSIT, smart order routing, maker-taker vs inverted venues, queue position, implementation shortfall, adverse selection, minimum quantity, dark liquidity curation, conditional invitations, crumbling quotes, odd lots, or venue markouts."
tags: market-microstructure, smart-order-routing, execution, dark-liquidity, tca, adverse-selection, queue-position, hft
---

# Virtu Market Microstructure

This skill is for building or reviewing fragmented-market execution logic, not for teaching basic market structure. Keep it self-contained; do not waste context on generic primers unless the user explicitly asks for pedagogy.

## Operating Posture

- Optimize parent-order economics, not child-order cosmetics. Fill rate, spread capture, and rebate capture only matter if they improve implementation shortfall after missed-fill chase cost.
- Treat "dark," "lit," and "midpoint" as venue families, not quality labels. Toxicity depends on counterparty mix, segmentation rules, and how fast the venue leaks your intent.
- Queue position is capital. A few places in line can be worth several bps; routing that wins price but loses queue often loses the trade.
- Every aggressiveness knob is also a leakage knob. If a parameter makes you fill faster, assume it also makes you easier to detect.

## Before You Design Or Review

Before routing logic:
- Ask which cost dominates here: spread, impact, or opportunity cost from missing the fill.
- Ask whether the symbol is tick-constrained. Under about $20-$25, queue economics and venue fee models behave differently; treat maker-taker and inverted venues as separate effective price levels.
- Ask whether urgency is real or just a dashboard preference. Many "high urgency" settings are really fear of missing out.

Before using dark or conditional liquidity:
- Ask whether you need blocks, small clean prints, or just more fill attempts.
- Ask which venue class you are targeting: broker-curated pool, bank ATS, exchange dark, blotter-scraping block pool, or conditional-only venue. They are not interchangeable.
- Ask what minimum quantity is filtering for: fewer prints, larger counterparties, or better counterparties. Those are different goals.

Before passive posting:
- Ask how you detect quote instability. Reactive latency defenses are not enough; anticipation happens before the public quote actually flips.
- Ask whether hidden posting helps or just puts you at the back of the queue waiting to get swept when the level is toxic.

## Virtu-Style Heuristics

### 1. Start from all-in cost

- Score routes as `explicit fees - rebates - spread captured + markout + missed-fill chase cost`.
- Measure markouts at multiple horizons. `1s` catches immediate toxicity, `1m` catches leakage, `5m` catches parent-order drift.
- If a venue improves child-order markouts but increases leftover urgency later in the schedule, treat it as worse, not better.

### 2. Queue position beats abstract venue preference

- KCG/IEX cited about `4.5 bps` of slippage between first and fifth place in a Nasdaq queue. Do not route by venue label alone.
- For large-cap names, long queues are not a universal problem; Nasdaq data showed most large-cap NBBO queues cycle in under `20s`, with queues above `60s` representing less than `1%` of traded value.
- Under tick-constrained prices, especially below about `$20-$25`, inverted venues can behave like a second price level because cheaper taker economics pull fills there first. Recompute queue-time economics by symbol, not by exchange brand.
- Do not assume inverted venues are cheaper in all-in terms. Several venue-cost studies found maker-taker and inverted venues are often statistically indistinguishable once fees, rebates, spread capture, and `5m` markouts are included; below `$25`, maker-taker can even outperform because wide ticks make queue position more valuable than headline price.

### 3. Minimum quantity is a scalpel, not armor

- Bigger MQ feels safer because it reduces tiny prints. That intuition is incomplete.
- IEX found high MQ barely reduced adverse selection by trade size and can worsen it because you bias toward larger counterparties whose residual interest keeps pushing price after your fill.
- Use MQ to shape venue access, not as a blanket anti-toxicity rule.
- For block or conditional venues, high MQ can be rational. BestEx found Virtu POSIT and Liquidnet had more than `75%` of conditional liquidity accessible at `5,000` shares.
- For bank ATSs, only about `20%-40%` of liquidity was accessible at that size; a blanket `5,000` MQ mostly turns liquidity off.
- A workable starting point from practitioner data is `100-500` shares for schedule algos, around `500` for generic liquidity seeking, and venue-specific higher floors only where the venue truly specializes in blocks.

### 4. Dark routing is curation, not aggregation

- Broker-curated dark pools can leak less than exchange dark pools. In one natural experiment, broker dark pools showed lower leakage and adverse selection from `500ms` out to roughly `5m`.
- Do not use a single "dark score." Split by venue class, counterparty restrictions, firm-vs-conditional order type, and post-trade markout profile.
- Keep a live exploration budget. Historical fill probability matters, but venue composition drifts intraday and yesterday's best venue is often today's toxic venue.

### 5. Conditional orders solve one problem and create another

- Conditionals avoid many toxic immediate fills, but invitations leak intent.
- BestEx measured race conditions on block invitations about `29%` of the time. If your logic firms up only on the first invite, you are optimizing operational simplicity at the cost of missing blocks.
- Preferred pattern: spray firm-ups across as many venues as your minimum-fill budget safely allows.
- If you cannot spray all venues, maintain a venue win matrix instead of naive first-come-first-served logic.
- When a firm-up succeeds, cancel lit child orders immediately to avoid double fills and secondary leakage.

### 6. Protect passive quotes from prediction, not just speed

- Stale-quote protection is table stakes. Predictive pickoff is harder.
- IEX's crumbling-quote work shows NBBO moves often unfold over about `1-2 ms`; a venue can be safe from pure reactive sniping and still be vulnerable to traders predicting the flip.
- When your instability signal fires, reprice or back away by one tick rather than waiting for the visible quote change. Virtu-style logic should treat quote-stability modeling as a first-class input to passive placement.

### 7. Use aggression bands as starting anchors, not constants

- Virtu quick guides are useful because they expose real desk priors:
- `Oasis` liquidity-seeking dark-with-take logic uses minimum POV anchors around `25% / 15% / 5%` for aggressive / neutral / passive.
- `Catch` implementation-shortfall logic uses guideline POV around `20% / 10% / 5%`, excluding blocks.
- `Opportunistic` favorable-price capture lives much lower, about `4-10% / 3-7% / 1-4%`, and explicitly accepts non-completion.
- Use these only as seed ranges. Override them when imbalance, spread instability, or venue fade rates say the market is no longer normal.

### 8. Odd lots are not harmless stealth

- Tiny clips feel invisible, but `99` shares can be structurally worse than `100`.
- Research summarized by Betterment/Bartlett found odd lots received about `10%` less price improvement than round lots, with `99`-share trades notably disadvantaged and more than `30%` of Amazon odd lots estimated to have done better as round lots.
- If you can aggregate slices into mixed lots with at least one round lot without blowing urgency, do it.

## Decision Tree

- Need certainty of completion now: use queue-aware SOR, sweep dark/SI before displayed only if the order is marketable, and hard-cap lit participation with an "I would" price or equivalent toxicity budget.
- Need blocks without advertising: use curated dark plus conditionals. Raise MQ only on venues that are actually block-heavy. Expect race conditions and design firm-up logic for them.
- Need benchmarked execution with finite time: run an implementation-shortfall schedule with adaptive POV. Use lit and dark simultaneously. Let value and imbalance signals pull volume forward or hold it back.
- Need price improvement with low urgency: stay passive, use quote-stability signals, and accept non-completion. This is where low-POV opportunistic logic belongs.
- Toxicity proxy spikes or cross-venue imbalance appears: assume market makers may withdraw together. Reduce passive exposure, widen venue filters, and stop trusting stale fill-rate estimates.

## Fallbacks

- If curated dark and conditional venues dry up, fall back to a queue-aware lit schedule with smaller child orders; do not blindly raise urgency and turn a block problem into a signaling problem.
- If venue-level toxicity signals disagree with fee-model rankings, trust markouts and queue outcomes over explicit fee economics.
- If quote-instability detection degrades, bias toward protected displayed logic and shorter resting times rather than hidden posting.
- If you cannot estimate missed-fill cost reliably, run a conservative benchmark schedule first and widen aggressiveness only after parent-level TCA proves the baseline is too slow.

## NEVER Do This

- NEVER maximize fill rate because it is the easiest metric to show clients. It is seductive because the dashboard turns green. The consequence is toxic fills that worsen parent implementation shortfall. Instead optimize all-in cost including missed-fill chase cost.
- NEVER set one global minimum quantity because larger prints feel institutionally "clean." The consequence is lost bank-ATS liquidity and sometimes worse adverse selection against larger counterparties. Instead size MQ by venue class, urgency, and order type.
- NEVER treat all dark liquidity as equally anonymous because "dark" sounds protected. The consequence is routing into venue classes that leak more than broker-curated pools. Instead score dark venues by participant mix, segmentation rules, and markouts.
- NEVER rank venues on historical fill rate alone because that model is stable in backtests and dead in production. The consequence is chasing liquidity that has already become toxic. Instead combine live queue state, spread, imbalance, instability, and a small exploration budget.
- NEVER post hidden solely to avoid display. Hidden orders often sit behind displayed interest and get filled when the price level is being swept, which is exactly when toxicity is worst. Instead use protected displayed logic or explicit quote-stability models.
- NEVER firm up only the first conditional invitation because it feels operationally safe. The consequence is losing blocks in race conditions and underestimating true available liquidity. Instead spray within the minimum-fill budget or use a venue win matrix.
- NEVER assume a speed bump or low-latency stack alone protects passive orders. The consequence is getting predicted, not just reacted to, during crumbling quotes. Instead run predictive instability signals and reprice before the public quote flips.
- NEVER default to odd-lot slicing for stealth. The consequence is systematically worse price improvement and lower venue priority. Instead aggregate to round-lot or mixed-lot thresholds when urgency permits.

## Diagnostics That Matter

- Separate `fill toxicity`, `information leakage`, and `completion risk`; they are different failure modes.
- Track per-venue-class markouts, not only consolidated dark vs lit.
- Record queue entry rank, queue exit reason, and cancel latency. A venue that looks good without queue context is hiding the real cost.
- Track conditional invitation rate, simultaneous-invitation rate, firm-up win rate, and fade rate.
- Re-estimate symbol buckets after regime changes: open/close, volatility shocks, fee changes, and sub-`$25` price transitions often invalidate yesterday's venue policy.

## Delivery Standard

When you implement or review code in this style, produce:
- the routing objective function,
- the venue segmentation rules,
- the toxicity and instability signals,
- the fallback when block logic fails,
- and the TCA fields needed to prove the design works.
