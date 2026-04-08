# AQR Factor Construction — Reference Code

**Read this before writing any factor construction code.** This implements the
decisions justified in the parent SKILL.md (HML Devil, 12-1 skip-month,
capped value weights, winsorize-then-zscore). The comments flag the exact
paper and decision for each choice so you can defend it in a research
review.

## 1. The winsorize → z-score pipeline (used by every factor)

```python
import numpy as np
import pandas as pd

def winsorize_and_zscore(series: pd.Series, clip_std: float = 3.0) -> pd.Series:
    """
    Standard AQR signal normalization.

    - Standardize FIRST, then clip at ±clip_std, then re-standardize.
    - Do NOT clip the raw ratio (e.g. raw B/P) before ranking — the
      extreme values of B/P are informative up to ~3σ.
    - Re-standardizing after the clip keeps the output unit-variance
      so you can sum signals across factors without re-weighting.
    """
    z = (series - series.mean()) / series.std(ddof=0)
    z = z.clip(-clip_std, clip_std)
    return (z - z.mean()) / z.std(ddof=0)
```

## 2. Value — HML Devil (current-price B/P composite)

```python
def build_value_devil(data, universe, asof_date) -> pd.Series:
    """
    HML Devil: current-price B/P composite.

    The key difference from Fama-French HML is the DENOMINATOR:
    - Fama-French uses market equity from ~6 months ago (June of prior year).
    - Devil uses the CURRENT market equity.
    Using the lagged denominator mixes ~20% of a poorly-constructed
    momentum factor into HML (Asness & Frazzini 2013). Use current price
    unless you are explicitly replicating Ken French's dataset.
    """
    current_price = data.get_prices(universe, asof_date)

    # Composite of four value metrics — NOT just B/P
    ratios = pd.DataFrame({
        'book_to_price':     data.get_fundamentals(universe, 'book_value', asof_date)      / current_price,
        'earnings_to_price': data.get_fundamentals(universe, 'trailing_eps', asof_date)    / current_price,
        'cf_to_price':       data.get_fundamentals(universe, 'operating_cf', asof_date)    / current_price,
        'sales_to_price':    data.get_fundamentals(universe, 'trailing_sales', asof_date)  / current_price,
    })

    # Normalize each metric independently, then average
    z = ratios.apply(winsorize_and_zscore, axis=0)
    composite = z.mean(axis=1)

    # CRITICAL: industry-neutralize. Cross-industry value becomes a
    # sector bet (banks-in-2008, energy-in-2020).
    industries = data.get_industries(universe)
    composite = composite.groupby(industries).transform(lambda x: x - x.mean())

    return winsorize_and_zscore(composite)
```

## 3. Momentum — 12-1 with skip-month, industry neutral

```python
def build_momentum_12_1(data, universe, asof_date) -> pd.Series:
    """
    Cross-sectional momentum: 12-month return, skipping the most recent month.

    - 'Skip-month' is NOT optional. Including the last month contaminates
      the signal with short-term reversal (bid-ask bounce, liquidity).
    - Use 12-1, not 6-1 or 7-12. Goyal-Wahal replicated 12-1 dominance
      in 35 of 36 international markets.
    - Industry-neutralize. Moskowitz-Grinblatt showed industry momentum
      exists, but we do not want to accidentally load onto sectors.
    """
    # Total return from t-252 trading days to t-21 trading days (skip ~1 month)
    p_start  = data.get_price(universe, asof_date, days_ago=252)
    p_skip   = data.get_price(universe, asof_date, days_ago=21)
    momentum = (p_skip / p_start) - 1.0

    industries = data.get_industries(universe)
    momentum   = momentum.groupby(industries).transform(lambda x: x - x.mean())

    return winsorize_and_zscore(momentum)
```

## 4. Quality Minus Junk (QMJ)

```python
def build_qmj(data, universe, asof_date) -> pd.Series:
    """
    Quality Minus Junk (Asness-Frazzini-Pedersen 2013).

    Four sub-components, equal-weighted after z-scoring. DO NOT drop
    any one of them — QMJ becomes unstable with fewer than four.
    'Safety' alone is essentially BAB.
    """
    profitability = data.get_gross_profits_to_assets(universe, asof_date)           # GP/A
    growth        = data.get_5y_earnings_growth_stability(universe, asof_date)      # low stdev of ΔE
    safety        = -(
                      data.get_debt_to_equity(universe, asof_date).rank(pct=True)
                    + data.get_realized_vol(universe, asof_date, days=252).rank(pct=True)
                    + data.get_market_beta(universe, asof_date, days=252).rank(pct=True)
                    ) / 3.0
    payout        = data.get_net_equity_payout_yield(universe, asof_date)           # dividends + buybacks - issuance

    components = pd.DataFrame({
        'profitability': winsorize_and_zscore(profitability),
        'growth':        winsorize_and_zscore(growth),
        'safety':        winsorize_and_zscore(safety),
        'payout':        winsorize_and_zscore(payout),
    })
    return components.mean(axis=1)
```

## 5. Portfolio construction — capped value weights, non-micro universe

```python
def build_long_short_portfolio(signal: pd.Series, market_cap: pd.Series,
                                nyse_mcap_pctiles: pd.Series) -> pd.Series:
    """
    Build a long-short portfolio from a cross-sectional signal.

    Construction rules (Jensen-Kelly-Pedersen 2022):
    1. Drop stocks below the NYSE 20th percentile of market cap (micro-caps).
       This single decision costs ~27 points of "replication rate" on
       published anomalies, which is the point — it ejects the ones that
       only worked on untradeable names.
    2. Tercile sort (top vs bottom third), NOT decile. Deciles look better
       on paper but concentrate in names you cannot build in size.
    3. Weight stocks by market cap CAPPED at the NYSE 80th percentile.
       Prevents mega-cap dominance (Nokia = 70% of Finland in 1999-2000)
       without the microcap overfit of equal-weighting.
    """
    non_micro = nyse_mcap_pctiles >= 0.20
    signal    = signal[non_micro]
    cap       = market_cap[non_micro]

    # Tercile breakpoints computed on the already-non-micro universe
    lo, hi = signal.quantile([1/3, 2/3])
    longs  = signal.index[signal >= hi]
    shorts = signal.index[signal <= lo]

    # Capped value weights: winsorize market cap at NYSE 80th percentile
    nyse_80 = cap[nyse_mcap_pctiles >= 0.80].min()
    capped  = cap.clip(upper=nyse_80)

    w_long  =  capped.loc[longs]  / capped.loc[longs].sum()
    w_short = -capped.loc[shorts] / capped.loc[shorts].sum()

    return pd.concat([w_long, w_short]).reindex(signal.index, fill_value=0.0)
```

## 6. Vol-targeting the assembled portfolio

```python
def vol_target(weights: pd.Series, cov: pd.DataFrame, target_vol: float = 0.10) -> pd.Series:
    """
    Scale a long-short portfolio to a constant annualized vol target.

    Vol-targeting is the ONLY form of timing that survives Asness's own
    skepticism. Do not lever individual positions; apply a single scalar
    to the whole book.
    """
    w = weights.values
    port_vol = float(np.sqrt(w @ cov.values @ w))
    if port_vol <= 0:
        return weights * 0.0
    scale = target_vol / port_vol
    return weights * scale
```

## Numbers to remember

| Parameter                            | Value                          | Source                          |
|--------------------------------------|--------------------------------|---------------------------------|
| Momentum formation window            | 252 trading days               | Jegadeesh-Titman, Asness 1994   |
| Momentum skip-month                  | 21 trading days                | Asness 1994                     |
| Universe cutoff                      | NYSE 20th percentile of mcap   | Jensen-Kelly-Pedersen 2022      |
| Sort granularity                     | Tercile (not decile)           | Jensen-Kelly-Pedersen 2022      |
| Market-cap cap for weighting         | NYSE 80th percentile           | Jensen-Kelly-Pedersen 2022      |
| Signal winsorization                 | ±3σ, after z-scoring           | AQR house style                 |
| Target factor idio vol               | 10% annualized                 | AQR house style                 |
| Canonical value-momentum blend       | 60/40 HML/UMD by dollar        | Asness-Frazzini 2013            |
| HML denominator (for "pure value")   | CURRENT price, not 6m-lagged   | Asness-Frazzini 2013            |
