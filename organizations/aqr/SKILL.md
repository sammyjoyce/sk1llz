---
name: aqr-factor-investing
description: Build factor-investing systems the way AQR actually runs them, with hard-won corrections to academic formulas (HML Devil, 12-1 skip-month, capped value weights, non-micro universes), honest drawdown arithmetic, and the discipline to resist factor timing. Use when building value/momentum/quality/defensive factors, designing a multi-factor long-short book, writing a backtest that must match live P&L, attributing live returns to factor exposures, or deciding whether a drawing-down factor is broken. Triggers:  factor model, Fama-French, HML, HML Devil, UMD, QMJ, BAB, betting against beta, momentum crash, value spread, factor timing, factor zoo, craftsmanship alpha, quant winter, systematic equity, long-short portfolio, tercile sort, winsorize, z-score, factor attribution, implementation shortfall.
tags: quantitative, factor-investing, risk-management, portfolio, backtesting, finance
---

# AQR Factor Investing

This skill encodes the construction details, drawdown mathematics, and implementation traps that separate a live-capital AQR-style factor book from a textbook Fama-French backtest. Almost everything here is what Asness, Frazzini, Israel, Moskowitz, Pedersen, and Ilmanen published *after* getting burned by the easy version.

## Before writing any factor code, ask yourself

1. **Would this survive removing every stock below the NYSE 20th percentile of market cap?** Most published anomalies die here. The Jensen-Kelly-Pedersen replication study dropped ~27 points of replication rate just by switching from uncapped deciles to non-micro terciles. If yours dies too, you are harvesting a micro-cap illusion, not a factor.
2. **Am I using *current* prices in the denominator of B/P, or Fama-French's 6-month-lagged ones?** Lagged prices contaminate HML with ~20% of a poorly-constructed momentum factor (Asness-Frazzini 2013, "The Devil in HML's Details"). If you want "pure value," you must rebuild it.
3. **What is my expected drawdown *duration*, not depth?** A Sharpe-0.4 factor has ~16% probability of making nothing for a full decade from a mere −1σ event. Plan for that, not for a 2-sigma catastrophe that probably never comes.
4. **If this factor stops working, how will I distinguish structural decay from a value-spread-widening drawdown?** Without a pre-committed answer, you will capitulate at the bottom.

## Craftsmanship alpha: the details that matter more than which factors you pick

Asness's "Little Things Mean a Lot" arithmetic: each construction decision below is worth roughly 0.10 Sharpe viewed alone. Ten of them together is ~0.32 Sharpe — which still has a 37% chance of subtracting in any given year and an 8% chance of subtracting over 20 years. Adopt them as a bundle of priors, not as individual bets you can evaluate.

### Value
- Composite of ≥4 ratios (B/P, E/P, CF/P, sales/P, forward E/P). Single-metric value fails from noise, not from crowding — "if the strategy is so popular that it won't work anymore, someone forgot to tell the prices" (Asness).
- **Use current market price in the denominator**, not Fama-French's 6-month-lagged price. This is the entire point of "HML Devil."
- **Intra-industry**, not cross-industry. Cross-industry value is a bet on whichever sector is structurally cheap (banks 2008, energy 2020), not on value.
- In tax-aware long-only books, value is the *expensive* factor to run because its tax drag comes from dividend income, which you cannot optimize away without destroying the signal. Momentum's drag is capital gains, which you can.

### Momentum
- **12-month return, skipping the most recent month** (the 12-1). Novy-Marx claimed 7-12 dominates 2-6; Goyal-Wahal replicated across 36 countries and found 12-1 dominated in 35. Skip-month is not optional — the last-month reversal is microstructure noise (bid-ask bounce, short-term reversal) and including it materially drags the Sharpe.
- **Momentum crashes are conditional beta risk, not tail risk.** After a bear market, momentum is long low-beta / short high-beta. A sharp upswing (Mar-May 2009, Aug 1932) crushes the short leg. Daniel-Moskowitz showed 100% of the crash comes from the short side, not the winners. The correct fix is **not** to hedge the market beta — it is to **combine with value**, which rallies in exactly those moments.
- **Momentum works best in expensive stocks**, not cheap ones (Asness 1997). Removing the 10% most expensive stocks drops momentum Sharpe with a −3.16 t-stat on a 50-year sample. Do not "cleanse" bubble stocks from a momentum book.

### Quality (QMJ)
- Four sub-components: profitability, growth stability, safety (low leverage + low volatility + low beta), payout. Drop any one and the factor becomes unstable. Safety alone is roughly BAB.
- Negative correlation to value (cheap stocks are usually junk) is a feature: a QMJ book hedges a value book during junk rallies (1999, mid-2020).

### Defensive / Betting-Against-Beta
- BAB is dollar-neutral AND **beta-neutral**: lever the low-beta long leg to match the short leg's beta. Long-only low-vol implementations lose most of the premium because they cannot apply the leverage step.
- The premium comes from leverage aversion (Frazzini-Pedersen 2014). Cheaper financing — not crowding — is what would erode it.

### Construction hygiene for all factors
- **Tercile sort with NYSE 20th-percentile breakpoint**, not raw deciles. Tercile+non-micro is the AQR-house tradability-vs-power tradeoff.
- **Capped value weighting**: weight by market cap winsorized at NYSE 80th percentile. Prevents mega-cap dominance (Nokia was ~70% of Finland in 1999-2000) without the microcap overfit of equal-weighting.
- Scale factor monthly idiosyncratic vol to 10%/√12 (~10% annualized). This normalizes comparisons across methodologies.
- Winsorize z-scores at ±3σ **after** standardizing; do not clip raw ratios before ranking. Outliers in B/P are informative up to that point.

## The drawdown arithmetic you must internalize

- Sharpe 0.3 (roughly the long-run market itself) can make nothing for a decade from a single −1σ event. Sharpe 0.4 is ~16% probability over a decade. Sharpe 1.0 multi-factor books still have ~24% chance of a down 5-year period.
- Every factor has had ≥1 multi-year drawdown. Value: 1999-2000, 2018-2020 (the worst in Samonov's 220-year dataset). Momentum: Mar-May 2009, Aug 1932. If your strategy cannot survive 2018-2020 *emotionally and financially* (margin calls, redemptions, staff cuts), you are not running the strategy you think.
- AQR itself went from $226B (2018) to $98B (2020), cut staff from ~1,000 to ~600, then printed +43.5% in 2022. Asness's own postmortem: "the best we've got is that value holistically lost." There was no mechanical explanation.
- **Never publish the backtest Sharpe as the expected Sharpe.** Every experienced AQR researcher privately discounts backtest Sharpes by 30-50% before setting client expectations. Asness has admitted this in print for decades.

## NEVER / INSTEAD

- **NEVER time factors on macroeconomic regimes**, because the macro→factor link is noise in virtually every implementation (Ilmanen et al 2021, Asness-Chandra-Ilmanen-Israel 2017). Factor timing requires being "right twice" — forecast the regime *and* the factor's sensitivity to it — and the second part barely exists. **INSTEAD**, hold a disciplined multi-factor portfolio and let cross-factor diversification do the work.

- **NEVER use a trigger-threshold (±20%) rebalance rule** on a factor book, because triggers fire during trend moves and force trades *against* time-series momentum. AQR's 43-year rebalancing study shows fixed-calendar annual rebalancing beats trigger-based on Sharpe *and* on turnover cost. **INSTEAD**, rebalance on a fixed schedule (monthly or annually), widen only if turnover cost exceeds your budget.

- **NEVER drop "bubble" or "expensive" stocks from the universe**, because momentum works best exactly in that tail (expensive-stock momentum has roughly twice the Sharpe of cheap-stock momentum; the −3.16 t-stat drop from cleansing them is the single largest construction mistake in the literature). **INSTEAD**, let value underweight and momentum overweight the same bubble name; the offsetting exposures are the point.

- **NEVER use Fama-French HML off Ken French's website for a "pure value" book**, because its 6-month lag on price mixes ~20% momentum into HML. Your stated "60/40 value/momentum" allocation is then actually ~50/50 of pure value and pure momentum (which is why AQR uses 60/40 HML+UMD, not 50/50 — the weights are *already* compensating for the contamination). **INSTEAD**, rebuild HML with current prices ("HML Devil") if you need clean factor interpretation.

- **NEVER treat a wide value-spread as a go-long timing signal**, because Asness's own "Contrarian Factor Timing Is Deceptively Difficult" showed value-spread timing has out-of-sample Sharpe indistinguishable from zero. Spreads can and do widen further — 2020 set a 220-year record. **INSTEAD**, use spreads to set *expected return* (wider = higher prospective Sharpe), but keep *risk targeting constant*. Only in a genuine once-a-century extreme should you make a small (<10%) "venial value-timing sin," and Asness himself only did it twice in 30 years (1999, 2019).

- **NEVER run a long-only multi-factor portfolio by blending single-factor sleeves** (50% value sleeve + 50% momentum sleeve), because at the stock level a cheap-junk name and an expensive-quality name cancel, destroying the cross-factor interaction. Fitzgibbons-Friedman-Pomorski-Serban (2017), "Don't Just Mix, Integrate." **INSTEAD**, compute a composite per-stock score across all factors and build one portfolio off it.

- **NEVER equate "the factor is down" with "the factor is broken."** A Sharpe-0.4 factor needs only ~55% of months positive over the long run. Real decay would show as a slow premium compression with a *narrowing* value spread; real drawdowns show as a *widening* spread. **INSTEAD**, check three things: (1) Is the spread wider or narrower than at drawdown entry? (2) Have 13F filings of systematic funds actually grown materially in the same strategy? (3) Are other quant shops also drawing down? If spread wider + AUM flat + peers also down, it is a normal painful drawdown — hold the position.

## Decision tree: a factor is drawing down, what do I do?

```
Is the factor's spread (value spread, momentum strength) at a historical
extreme of CHEAPNESS vs its own history?
├── YES (spread wider than entry)
│   └── The factor is COMPRESSING, not DECAYING. Rebalance on schedule,
│       do not cut. If spread > 2σ wide, consider a small (<10%) tilt up
│       — a "venial" factor-timing sin. Do NOT scale to the spread; scale
│       at most once.
└── NO (spread narrower than entry, or unchanged)
    ├── Has AUM in known factor funds grown materially? (Check 13F.)
    │   └── YES → possible crowding decay; trim by 10-20%, do not zero.
    ├── Is there a real economic story for structural decay (e.g.,
    │   rates regime change with a mechanical link)?
    │   └── Almost always NO. Macro-factor links are noise.
    └── Is the drawdown concentrated in the short leg during a sharp
        market upswing?
        └── YES → this is a momentum-crash pattern (Daniel-Moskowitz).
            Hold; your value leg is almost certainly up in the same
            window and is your natural hedge.
```

## Multi-factor portfolio construction rules

1. Weight factors by **risk contribution**, not dollar. A 60/40 HML/UMD by dollar is ~50/50 by risk because momentum vol is higher.
2. The canonical 60/40 HML/UMD benchmark exists **because HML already contains ~20% momentum** (the Devil). 60/40 HML/UMD is really ~50/50 of pure value and pure momentum. Do not generalize the 60/40 to other pairs.
3. Target total portfolio vol with a floating leverage factor; do not lever individual positions. Vol-targeting is the only form of timing that survives Asness's own skepticism.
4. For long-only: **integrate, don't mix.** One composite score per stock, one portfolio.

## Signals your skill is broken

- Backtest Sharpe > 2.0 gross of costs on liquid stocks → you have a bug. The real ceiling for a tangency portfolio of factors across all markets and asset classes is ~1.5 (Ilmanen et al 2021).
- Your "value" factor makes money during a momentum crash (e.g., Mar-May 2009), but the magnitude is <20% of momentum's loss → the sign is right but your "value" has contamination; check for Devil-in-HML issues.
- Your momentum factor has <100%/year turnover on monthly rebalance → it is a long-term trend factor, not momentum; it will decorrelate from published UMD.
- Your long-short book has large sector tilts → you forgot to industry-neutralize; you are running a sector bet with a factor label.

## Heavy references — load only when implementing

- `references/construction-code.md` — Python reference for HML Devil (current-price B/P), 12-1 momentum with skip-month, QMJ composite, capped value weights, and the winsorize→z-score pipeline. **READ this before writing any factor construction code.** Do not re-derive from academic papers.
- `references/backtest-frictions.md` — transaction cost, borrow cost, and market impact models calibrated to Frazzini-Israel-Moskowitz's $1T-of-live-trades paper. **READ this before running any long-short backtest.**
- `references/attribution.md` — factor attribution and spread diagnostics. Load only when decomposing live P&L or diagnosing a drawdown.

Do **NOT** load these files for conceptual discussions, philosophy questions, or "should we use factors at all" conversations. They are strictly implementation artifacts.
