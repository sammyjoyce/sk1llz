---
name: renaissance-statistical-arbitrage
description: Use when designing, reviewing, or debugging equity statistical arbitrage, market-neutral factor books, residual mean reversion, or alpha research pipelines exposed to crowding, borrow, and data-mining risk. Teaches practitioner heuristics for alpha half-life, funding stress, short-book asymmetry, point-in-time data, DSR/PBO-style validation, and capacity limits. Trigger on stat arb, market neutral, residuals, mean reversion, factor book, borrow, crowding, alpha decay, backtest overfitting, DSR, or PBO.
---

# Renaissance Statistical Arbitrage

This skill is self-contained. Do not load generic trading primers for this task unless the user explicitly wants pedagogy; they dilute the edge cases that matter.

Renaissance-style work is not "predict the market." It is "find tiny conditional edges, prove they survive adversarial validation, then assume they will die from crowding, cost, or borrow before they die from theory."

## Before You Build Anything

Before testing a signal, ask yourself:

- Is this edge actually mispricing, or am I being paid to warehouse liquidity for forced sellers?
- Would the signal still exist if I removed vendor backfills, restatements, stale fundamentals, and today's constituent list?
- Is the short-side alpha coming from normal names, or only from hard-to-borrow specials?
- Which desks would independently discover the same edge from the same feature set, and what happens when they all delever at once?
- What event breaks the residual first: earnings, borrow recall, rebalance, index membership change, M&A, or a funding squeeze?

If you cannot answer those questions, you do not have a deployable stat-arb idea yet.

## Core Operating Beliefs

- Treat every alpha as a decaying inventory-management problem, not a timeless law.
- Most short-horizon alpha is an execution business in disguise. Research without live cost modeling is fiction.
- "Market neutral" is not enough. The real blow-up channel is crowding in the same liquidation path.
- The short book is not symmetric to the long book. Borrow supply, recalls, and fees arrive exactly when the short looks best.
- A beautiful backtest usually means you optimized the history, not the future.

## Decision Tree

### 1. Classify the edge by holding period

- If the edge monetizes over 1-5 trading days, treat it as microstructure alpha. You need live slippage, queue/adverse-selection assumptions, venue behavior, and borrow state before you trust a Sharpe.
- If the edge monetizes over 1-6 weeks, treat it as residual mean reversion. You need event guards, crowding stress, and point-in-time universe construction.
- If the edge needs months to work, it is probably a factor or balance-sheet trade, not classic stat arb. Evaluate funding, factor crowding, and macro regime dependence first.

### 2. Classify the short book

- If most short-side P&L comes from specials or low-float names, you are partly trading securities-lending state. Model live borrow fees, recalls, and forced buy-ins.
- If the short book works only on paper because the vendor assumes frictionless borrow, cut the book or restate the idea as long-only / generalized-collateral only.

### 3. Classify the failure mode

- If performance collapses after cost, the signal was never alpha; it was spread capture you do not own.
- If performance collapses only during deleveraging windows, the signal is crowded liquidity provision.
- If performance collapses after removing restatements or current constituents, the "alpha" was data leakage.
- If performance survives all of that but dies after multiple-testing adjustment, you found the winner's curse.

## Numbers That Matter

- In the Avellaneda-Lee style residual framework, only trade residuals whose mean-reversion speed implies roughly less than 30 trading days of half-life, expressed there as kappa greater than about 8.4 annualized.
- The same framework used s-score entry thresholds around plus/minus 1.25, with exits much closer to zero at about +0.75 for shorts and -0.50 for longs. The point is not the exact constants; it is that exits should be easier than entries because the residual's edge decays fastest after snapback begins.
- AQR's cost study found short-term reversal far more capacity-constrained than value, size, or momentum, with break-even global capacities roughly $17B for short-term reversal versus about $122B for momentum, $811B for value, and $1,807B for size at 1% tracking error. If your alpha lives at the fastest horizon, assume scalability breaks first there.
- Bailey and Lopez de Prado's deflated Sharpe work assumes quant teams routinely run millions or billions of trials. In that setting, a raw Sharpe from the best branch means almost nothing unless you record how many materially different attempts you made.

## Validation Procedure

1. Define the edge in one sentence with its economic path to monetization.
2. Rebuild the dataset as the desk would have seen it then, not as the vendor repaired it later.
3. Count the real trial tree: features, labels, universe filters, lag choices, cleaning rules, stop logic, and ranking rules.
4. Reject the signal on deflated metrics first; only then look at pretty charts.
5. Run crowding stress separately from market stress. A stat-arb book often fails when peers unwind, not when the index moves.
6. Stress the short book with borrow-fee spikes, recalls, and untradeable names.
7. Stress liquidation horizon. If expected profit improves when you lengthen the holding period during stress, you are probably harvesting a liquidity premium and carrying funding risk.

## Anti-Patterns

- NEVER ship a signal because the best slice has p < 0.01; that is seductive because it feels scientific, but after enough feature, universe, and cleaning choices the winner's curse dominates and live Sharpe regresses toward zero. Instead log the full trial tree and gate the idea on deflated-Sharpe and probability-of-backtest-overfitting checks.
- NEVER call a book safe because beta is near zero; that is seductive because market neutrality looks like hedging, but 2007-style unwinds hit books that shared the same long-value, short-momentum, and liquidity exposures. Instead monitor overlap in factor tilts, funding sensitivity, and liquidation-path correlation with peer books.
- NEVER treat the short book as symmetric to the long book; that is seductive because the historical short leg often carries the prettiest alpha, but hard-to-borrow fees, recalls, and supply withdrawal appear exactly when the mispricing looks widest. Instead split results into generalized-collateral versus special names and assume the short book is the first place paper alpha disappears.
- NEVER trust stationarity or cointegration tests by themselves; that is seductive because the statistics look formal, but residual relationships fail around earnings, ETF and index rebalances, M&A, balance-sheet restatements, and borrow shocks. Instead require event-robustness and explicit kill rules for structural breaks.
- NEVER extrapolate fast mean reversion to large capacity; that is seductive because gross Sharpe is often highest at the shortest horizon, but the fastest horizons are usually where cost, queue position, and adverse selection erase the edge. Instead size from realized cost curves and assume short-term reversal is an execution business.
- NEVER average "uncorrelated" signals without checking whether they fail through the same dealer balance sheets; that is seductive because low historical correlation looks diversified, but crowding shows up as synchronized exits, not as pretty covariance matrices. Instead cluster signals by liquidity source and liquidation dependency, then diversify across those clusters.

## What Experienced Practitioners Watch Live

- Alpha half-life, not just hit rate. A signal that still predicts but monetizes too slowly is dead capital.
- Profit after borrow and impact by bucket, especially on the short tail.
- Whether the edge migrates into fewer names over time. Concentration is often the first sign of crowding.
- Whether stress periods reward slower liquidation. That is a warning that you are financing dislocation, not harvesting pure prediction.
- Whether factor-neutral books are quietly reloading the same hidden exposures after every rebalance.

## Fallback Rules

- If you do not have live or defensible borrow assumptions, do not claim short-side anomaly alpha. Restrict the idea to long-only or GC-only variants.
- If you do not have point-in-time constituents or restatement-aware fundamentals, drop fundamental alpha claims and work only with explicitly lagged price/volume data.
- If you cannot model realized costs well enough for the target horizon, move the horizon out or keep the result in research-only status.
- If you cannot measure crowding directly, assume lower gross leverage, wider liquidation slippage, and a shorter useful life than the backtest suggests.

## What Good Output Looks Like

When using this skill, produce:

- The edge hypothesis in one sentence.
- The exact decay path: cost, crowding, borrow, or data leakage.
- The validation stack used to reject overfitting.
- The live kill-switches that retire the signal before the market does it for you.
