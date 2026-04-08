---
name: de-shaw-computational-finance
description: "Make research, portfolio, and execution decisions in a D.E. Shaw style: hypothesis-first, leak-intolerant, capacity-aware systematic trading. Use when building alpha research, stat-arb, factor portfolios, transaction-cost models, risk systems, or research infrastructure. Triggers: quant, stat arb, alpha decay, market impact, borrow fee, crowding, backtest, factor model, execution, capacity."
tags: trading, computational-finance, quantitative, stat-arb, market-microstructure, risk, portfolio-construction
---

# D.E. Shaw Computational Finance

This skill is for turning vague quant ideas into evidence that survives hostile implementation. The stance is simple: a result is not real until it survives multiple-testing controls, point-in-time data checks, borrow and impact frictions, and crowding stress.

Do not load generic factor-investing or "what is market microstructure" material for normal use. This skill is intentionally self-contained. Before writing code or approving a result, read the project's actual data-timestamp, cost, borrow, and execution interfaces; this skill tells you what to demand from them.

## Read Order

Before building research code, read the project's point-in-time data and corporate-action path first. Most quant failures are timestamp bugs wearing statistical makeup.

Before approving a backtest, read the project's cost, borrow, and execution assumptions next. If you cannot find them, treat that as a blocker rather than filling the gap with textbook defaults.

Do not load generic primers, factor encyclopedias, or market-microstructure intros unless the user explicitly wants teaching material. They lower signal density and weaken the decision boundary.

## Think Like A Skeptical Quant

Before researching a signal, ask yourself:
- What exact market mistake is being harvested, and why has competition not removed it yet?
- What is the edge clock: minutes, days, months? If the signal half-life is shorter than the time needed to source liquidity, this is an execution problem disguised as alpha.
- Which parts of the result come from the short leg, stale fundamentals, or cheap historical spreads that no longer exist?
- What would falsify the signal quickly? Write that down before running the first search.

Before trusting a backtest, ask yourself:
- How many independent trials did I really run, including feature choices, universe cuts, rebalance rules, neutralizations, and cleaning decisions?
- Are labels or features overlapping across folds? If yes, ordinary walk-forward or k-fold is leaking.
- Does the signal still work after replacing idealized fills with realistic arrival, spread, borrow, and incomplete-fill assumptions?

Before sizing a strategy, ask yourself:
- Is the alpha still attractive after crowding, not just volatility, is treated as a state variable?
- Is the short book a real source of edge, or just a backtest convenience that will disappear when names become specials or recalls hit?
- Am I comparing net alpha conditional on urgency, venue, and participation, or fooling myself with average bps costs?

## Hard Research Gates

Use these as default gates unless the user explicitly wants exploratory work:

1. Treat every research branch as multiple testing.
   A new factor should generally clear `t > 3.0`, not the old `t > 2.0` standard. Harvey, Liu, and Zhu argue finance's usual significance cutoff is too weak once hundreds of factors have been mined.
2. Record trial count, then deflate the Sharpe.
   Bailey and Lopez de Prado show that an annualized Sharpe of `1.0` over `10` years of daily data drops below 95% confidence after as few as `3` independent trials once selection bias is acknowledged.
3. Use purge-plus-embargo validation whenever labels overlap in time.
   Ordinary walk-forward is not enough when your target spans future windows or when features reuse adjacent information.
4. Demand point-in-time joins, not reconstructed truth.
   Restated fundamentals, revised corporate actions, and borrow data observed after the decision time are enough to turn mediocre signals into fake discoveries.
5. Convert alpha into economics immediately.
   A signal that only works before spread, borrow, impact, recall risk, and unfilled-order opportunity cost is not a production candidate.

## Implementation Heuristics That Matter

- Post-publication anomaly decay is brutal.
  Chen and Velikov find the average anomaly nets about `4 bps/month` after trading costs and post-publication effects; the strongest only reach about `10 bps/month`. If your idea resembles published anomaly plumbing, assume the baseline edge is near zero until you show a fresh dataset or mechanism.
- Turnover is often the hidden killer.
  In the same work, post-publication implementations show about `40%` two-sided monthly turnover and roughly `85 bps` average paid spread, enough to erase about `30 bps/month` of gross return.
- Borrow is asymmetric and stateful.
  D'Avolio reports that about `91%` of borrowed names are general collateral at roughly `17 bps/year`, but the remaining `9%` are specials averaging about `4.30%/year` and sometimes far higher. Later evidence shows that by 2023 more than `15%` of the universe had borrow fees above `10%`, and sub-`$100M` names averaged above `30%`.
- Short unavailability matters as much as short fees.
  Kim and Lee's evidence, summarized in "When Equity Factors Drop Their Shorts," attributes about `10.4 bps/month` of anomaly drag to shorting frictions, roughly `40%` of gross short profits, with both fees and outright inability to source stock.
- Impact is not globally linear.
  On ANcerno metaorders, impact is approximately linear only for very small volume fractions `phi < 1e-3`; from about `1e-3` to `1e-1` of daily volume it is better modeled by square-root behavior. Treat anything near or above `1e-1 ADV` as a stress regime, not a calibrated expectation.
- Participation rate has its own threshold.
  Bucci et al. estimate the crossover participation rate around `eta* ~= 3.15e-3`, with duration dependence near `T^-1/2`. Small child orders can look cheap because fast liquidity absorbs them; larger metaorders run into slow liquidity and a different cost regime.
- Closing auctions are often cheaper than continuous trading for non-urgent rebalances.
  Recent evidence shows materially lower impact at the close than in continuous trading, while the opening auction is usually the worst place to lean on size because overnight information concentrates informed flow there.
- Crowding should be treated like a live risk factor.
  MSCI reports that factor crowding scores above `1` were historically associated with a meaningfully higher frequency of subsequent drawdowns, above `25%` over the following year in their 2025 review.

## Decision Tree

If you are evaluating a new signal:
- If the result is `t <= 2`, reject it as noise unless the task is explicitly exploratory.
- If `2 < t <= 3`, keep it in research only if there is a strong causal prior and a clean untouched holdout.
- If `t > 3`, move to cost, borrow, and capacity translation before spending time on model refinement.

If the signal is short-horizon:
- If alpha decays faster than you can complete the order, optimize execution first.
- If the strategy still looks attractive only under midpoint or linear-impact fills, reject it.
- If close-auction liquidity matches the holding horizon, benchmark against close execution, not all-day VWAP mythology.

If the edge comes mostly from the short leg:
- If names are small-cap, high-dispersion, high-turnover, or message-board crowded, assume borrow fragility first and alpha second.
- If borrow data is missing, rerun the economics as long-only plus index or sector hedge.
- If removing the individual-stock short destroys the edge, you do not yet have a robust strategy.

If portfolio construction is the bottleneck:
- If optimizer output changes violently under small covariance changes, shrink and regularize before adding more factors.
- If the optimizer neutralizes every known exposure, check whether you have neutralized the alpha itself.
- If multiple teams would share the same risk model and neutralization stack, explicitly model crowding and liquidation correlation.

## NEVERs

- NEVER accept `t ~= 2` evidence because that threshold is still common in finance papers. It is seductive because it looks "published-grade," but in practice it promotes false discoveries into production queues. Instead demand `t > 3`, a declared trial count, and a deflated performance check.
- NEVER compare strategies only on reported transaction-cost bps because low cost can simply mean slow trading against fast-decaying alpha. That mistake selects pretty execution statistics and poor net PnL. Instead compare net alpha conditional on decay rate, participation, venue choice, completion rate, and opportunity cost.
- NEVER assume the short book is symmetric with the long book because the names that are most attractive to short are often the ones that become specials or unavailable precisely when you need them. That shortcut creates phantom alpha and surprise recalls. Instead use stateful borrow, fee, and recall assumptions and test long-only-plus-hedge fallbacks.
- NEVER extrapolate linear impact from tiny child orders into portfolio-scale trading because fast liquidity makes small prints look cheap while slow liquidity sets the real bill. The consequence is systematic underestimation of capacity and liquidation risk. Instead use piecewise impact with square-root stress once size gets beyond tiny volume fractions.
- NEVER trust post-publication anomaly economics because the chart still looks smooth. That is seductive because the in-sample story remains coherent, but the live result is often spread-and-crowding tax with no residual edge. Instead assume publication decay plus current spreads remove most of the edge until a fresh source of exclusivity is proven.
- NEVER neutralize every observable risk factor because shared neutralization schemes create identical trades and synchronized exits. The consequence is crowding-driven drawdowns that look like "model error" after the fact. Instead preserve intentional exposures and treat crowding as a separate constraint.

## Fallbacks When Reality Is Messy

- Missing borrow data:
  Price the strategy with the short leg removed, then with a synthetic hedge, then with a punitive specials schedule. If it only works in the optimistic case, stop.
- Missing impact model:
  Bound economics between a close-auction benchmark and a stressed square-root impact curve. If the trade only survives under linear or spread-only costs, stop.
- Weak sample length:
  Prefer fewer, theory-linked parameters and demand a longer untouched holdout instead of widening the search.
- Regime instability:
  Re-estimate on subperiods defined by microstructure changes, decimalization, fee regime shifts, and crowding spikes. A signal that depends on one market regime is a conditional tactic, not a platform strategy.

## Freedom Calibration

- High freedom:
  Hypothesis generation, factor combinations, alternative hedges, and execution design. Explore broadly, but keep a written falsification rule and a live economics check.
- Low freedom:
  Timestamp handling, validation splits, borrow assumptions, cost accounting, and reporting claims. These are not creative surfaces. Use explicit gates and reject "close enough" shortcuts.
