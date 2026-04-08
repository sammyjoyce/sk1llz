---
name: two-sigma-ml-at-scale
description: >-
  Build execution-aware financial ML systems in the style of a Two Sigma research
  factory: point-in-time data contracts, factor-construction discipline, purged
  and embargoed validation, theory-gated search, and portfolio-aware model
  selection. Use when the user mentions quant ML, alpha research, alternative
  data, feature stores, cross-sectional signals, purged CV, embargo, deflated
  Sharpe, factor neutralization, regime models, capacity, or large-scale
  backtesting.
---

# Two Sigma ML at Scale

This skill is for research-factory work, not generic "train a model" work. The edge is usually not the learner. It is the protocol that makes leakage, false discovery, and implementation drag harder than finding signal.
This file is intentionally self-contained. Do not load extra material unless the user explicitly asks for formulas, citations, or implementation code.

## Operating Mindset

Think like a meta-strategy team, not a lone PM. López de Prado's critique is useful here: asking isolated researchers to each find a standalone strategy usually produces either false positives or overcrowded academic factors. Organize work as a factory: data vintages, labels, validation, portfolio construction, and execution feedback are separate stages with separate quality checks.

Before doing anything, ask yourself:
- What is the path from prediction to PnL after costs, borrow, and capacity?
- Which timestamps matter: event time, vendor publish time, ingest time, and trade decision time?
- Is this really a signal problem, or a portfolio-construction or execution problem wearing a modeling costume?
- How many independent trials can I justify before the search itself becomes the largest risk in the system?

## Decision Tree

If the task is about features or data quality:
- Prioritize point-in-time joins, vendor revision history, and feature lineage before touching model code.

If the task is about validation:
- Use purged CV with embargo when labels or features span time windows.
- Use simple walk-forward only when deployment cadence is itself the object of study.

If the task is about model comparison:
- Compare on rank IC, turnover, drawdown shape, capacity, and deflated Sharpe, not RMSE alone.

If the task is "make it perform better":
- Reduce search-space entropy first with theory, universe constraints, and neutralization rules.
- Expand model class or hyperparameter budget only after the search protocol is defensible.

## Hard-Won Rules

### 1. Factor themes are not generic

Two Sigma showed that ostensibly similar value factors were only about 14% correlated on average once definitions changed. Treat every feature family as a specification problem: winsorization, lagging, neutralization, rebalance timing, and liquidity filters can change the economic bet more than the model.

### 2. Stationarity is expensive if you buy it with memory

Returns are stationary but often erase the persistence the model needs. In López de Prado's E-mini example, fractional differentiation reached the 95% ADF threshold near `d ~= 0.35` while retaining about `0.995` correlation with the original series. `d = 1` was statistically cleaner and economically dumber. Defaulting to first differences is often information destruction masquerading as rigor.

### 3. Search volume is a hidden risk budget

A daily strategy with annualized Sharpe `2.5`, five years of data (`T = 1250`), and ugly higher moments still falls to a deflated Sharpe near `0.90` after `N = 100` independent trials. The same setup clears about `0.95` around `N = 46`. Trial count is not metadata. It is part of the statistic.

### 4. Small construction choices create different risk books

Two Sigma's low-risk work showed that seemingly minor choices such as dollar-neutral vs beta-neutral and sector-tilted vs sector-neutral construction can flip crisis behavior. Do not call two signals "the same factor" unless their neutralization and portfolio construction rules also match.

### 5. Theory is not optional scaffolding

Without a theory gate, the variance of backtest results rises as the search widens, so the best result improves even when true skill is zero. If you must search a finite menu, the DSR paper's secretary-problem heuristic is a useful sanity check: observe roughly the first 37% to set the hurdle, then accept the first candidate that beats all prior ones. Compute budgets amplify false positives just as efficiently as true positives.

## Procedure

### A. Data and label contract

1. Define four times for every field: event time, vendor publish time, ingest time, and tradable decision time.
2. Refuse any dataset that cannot answer "what did we know at this timestamp?"
3. Version vendor mappings and corporate-action logic. Revised fundamentals, changed identifiers, and backfilled alt-data labels create fake foresight if you only store latest-state tables.
4. Store feature definitions separately from model code. Features are reusable IP; models are disposable.

### B. Sampling and targets

1. Build labels around the holding period and execution rule, not around the target that is easiest to regress.
2. When labels overlap in clock time, assume samples are non-IID until proven otherwise.
3. Weight or subsample to reduce redundancy when many observations share the same future path; otherwise the model learns repeated episodes and reports fake confidence.
4. Prefer ranking or classification targets when downstream trading is a cross-sectional sort. Predicting precise returns often looks sophisticated while adding estimation noise you will throw away in portfolio construction.

### C. Validation

1. Purge any training observation whose label window overlaps the test window.
2. Embargo training observations immediately after the test fold when serial dependence can leak through feature windows. López de Prado's rule of thumb is `h ~= 0.01T` bars when you need a starting point.
3. Count independent trials, not raw jobs. Sweeping 500 correlated settings is not 500 independent discoveries.
4. Record every failed trial, not only winners. If you cannot reconstruct the graveyard, you cannot trust the champion.

### D. Selection and deployment

1. Pick models on a portfolio scorecard: IC distribution, turnover, implementation shortfall sensitivity, capacity, and deflated Sharpe.
2. Stress by regime and microstructure, not only calendar splits. Good averages with one unrecoverable crisis mode are not robust.
3. Track time-under-water, not just max drawdown. Slow recovery is often the earliest sign that the feature economics changed.
4. Keep a cheap baseline live. If the fancy model only wins before costs or only at tiny capital, treat it as research, not production.

## NEVER Do This

- NEVER optimize on raw Sharpe because it rewards lucky search breadth and non-normal payoff shapes. Instead track deflated Sharpe or an equivalent multiple-testing correction tied to independent trial count.
- NEVER difference every series to `d = 1` because stationarity feels scientifically safe. Instead find the minimum transformation that passes your stationarity test while preserving memory.
- NEVER call a dataset "point in time" just because rows have dates. Instead require arrival-time semantics and revision history, or assume the table leaks.
- NEVER compare factor variants without harmonizing neutrality, rebalance timing, and liquidity rules because the seductive shortcut is to blame the model for what is really construction drift. Instead compare signals inside one canonical portfolio recipe.
- NEVER let each researcher invent standalone pipelines because it feels fast at the start and creates an illusion of parallel alpha discovery. Instead run a shared research factory with common data contracts, labeling rules, and an evaluation ledger.
- NEVER trust cross-validation splits that ignore overlapping labels because ordinary ML tooling assumes IID samples and will overstate certainty. Instead purge overlaps and add embargo where features or labels span time.
- NEVER promote a model that only beats baselines on prediction error because the seductive story is "better forecast, better strategy." Instead require improvement after turnover, costs, neutrality, and capacity constraints.

## Failure Modes and Fallbacks

If the signal vanishes after costs:
- Check whether the model rediscovered short-horizon mean reversion or liquidity provision without paying the spread.
- Reduce rebalance frequency before changing model class.

If live results lag backtests but paper trading looked fine:
- Audit timestamp semantics and universe membership first.
- Then audit borrow, queue position assumptions, and stale vendor mappings.

If every complex model beats the baseline in research:
- Assume leakage, duplicated samples, or a broken champion-selection process until disproven.
- Re-run with fewer features, fewer trials, and stricter purging.

If a factor works only in one crisis or one sector:
- Treat it as a conditional sleeve, not a universal alpha.
- Add explicit regime or sector routing instead of averaging away the problem.

## Freedom Calibration

Use high freedom for model family choice, ensembling, and regime modeling.
Use low freedom for timestamps, label windows, neutralization rules, CV hygiene, and performance reporting. In this domain, catastrophic mistakes usually come from invisible protocol drift, not from choosing the wrong learner.

## Exit Criteria

A good outcome in this style has all of the following:
- Every feature is reproducible from point-in-time inputs.
- Every reported winner includes trial count and validation protocol.
- Portfolio construction is specified tightly enough that factor drift is detectable.
- The claimed edge survives costs, turnover, and regime inspection.
