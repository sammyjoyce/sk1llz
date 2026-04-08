# Factor Attribution and Drawdown Diagnostics⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌‌‌‌‌​‍‌‌​‌​‌​​‍‌​‌​​‌​‌‍​​‌‌‌‌​‌‍​​​​‌​‌​‍‌‌​​‌‌​​⁠‍⁠

**Load only** when decomposing live P&L or diagnosing a drawing-down
factor. This is the math AQR uses to answer "is my alpha real, or am I
just riding factor beta?" and "is this drawdown a compression or a
decay?"

## 1. The canonical decomposition

```
R_p(t) = α + Σᵢ βᵢ · Fᵢ(t) + ε(t)
```

Where `Fᵢ(t)` is the return of factor i at time t, `βᵢ` is the
portfolio's exposure to factor i, `α` is residual (alpha), and `ε` is
noise.

**The mistake to avoid:** running an unconstrained OLS regression of
portfolio returns on factor returns and calling the intercept "alpha."
This conflates static exposure with dynamic exposure. For an AQR-style
book the exposures change at every rebalance, so you need holding-
based attribution.

## 2. Holding-based attribution (the honest version)

```python
def attribute_returns(portfolio_returns: pd.Series,
                       factor_exposures:  pd.DataFrame,  # beta at each date
                       factor_returns:    pd.DataFrame) -> dict:
    """
    Portfolio return is attributed by multiplying each date's known
    exposures (from the holdings) by that date's factor returns.
    Residual is alpha.

    This is more honest than rolling regressions because the exposures
    come from the actual portfolio weights, not from a historical fit.
    """
    common = portfolio_returns.index.intersection(factor_returns.index)
    p = portfolio_returns.loc[common]
    B = factor_exposures.loc[common]   # shape (T, K)
    F = factor_returns.loc[common]     # shape (T, K)

    contributions = (B * F).sum(axis=0)          # per-factor total
    factor_total  = contributions.sum()
    alpha         = p.sum() - factor_total

    return {
        'total_return':         p.sum(),
        'factor_contributions': contributions.to_dict(),
        'alpha':                alpha,
        'avg_exposures':        B.mean().to_dict(),
        'factor_returns_total': F.sum().to_dict(),
    }
```

## 3. Value-spread diagnostic (the "is it broken or compressed" test)

The single most useful diagnostic during a drawdown. If the spread is
WIDER at the bottom than at the top of the drawdown, the factor is
**compressing**, not decaying.

```python
def value_spread(cheap_portfolio_price_ratios: pd.Series,
                  expensive_portfolio_price_ratios: pd.Series) -> float:
    """
    Ratio of how 'cheap' the cheap leg is versus how 'cheap' the
    expensive leg is. For B/P: ratio of median B/P of cheap leg over
    median B/P of expensive leg. >1 means the spread is wide.

    Measure this in a COMPOSITE (B/P + E/P + CF/P + S/P), not B/P
    alone. The 2020 value drawdown set historical records on all
    four measures simultaneously, while single-metric B/P spread is
    distorted by intangibles accounting.
    """
    return (cheap_portfolio_price_ratios.median()
            / expensive_portfolio_price_ratios.median())
```

Rules of thumb for value spread on a composite measure:

| Spread vs 40-year median | Interpretation                               |
|--------------------------|----------------------------------------------|
| < 0.8×                   | Compressed — expected premium below average  |
| 0.8–1.2×                 | Normal                                       |
| 1.2–1.5×                 | Rich — expected premium above average        |
| 1.5–2.0×                 | Extreme — historical top decile              |
| > 2.0×                   | Once-per-generation; late 2020 peaked ~2.4×  |

**Timing rule:** never scale exposure to the spread. You may make a
single one-time tilt up (≤10%) at > 2.0×, and must hold it until the
spread reverts below 1.2×. This is the "venial value-timing sin"
Asness described in 2019.

## 4. Decomposing a momentum crash

```python
def momentum_crash_decomposition(long_leg_return: pd.Series,
                                   short_leg_return: pd.Series,
                                   market_return:    pd.Series) -> pd.DataFrame:
    """
    Daniel-Moskowitz (2013): momentum crashes come 100% from the short
    leg during sharp market upswings after a bear market. If your
    momentum drawdown has the long leg contributing, it is NOT a
    classical momentum crash — check for construction bugs.
    """
    return pd.DataFrame({
        'long_contribution':  long_leg_return,
        'short_contribution': -short_leg_return,  # short return = -position return
        'market_return':      market_return,
    })
```

A classical momentum crash has:
- Short leg (losers) rallying violently (negative to short book).
- Long leg (winners) roughly flat or slightly up.
- Market rising sharply (+10-30% over 2-3 months).
- Momentum's conditional beta during the window goes sharply negative.

If you see a crash with the LONG leg losing, the crash is not a
Daniel-Moskowitz event; it is probably a crowding unwind (e.g.
Aug 2007). Treat it differently — crowding unwinds do mean-revert
within weeks, and you can add into them.

## 5. Exposures to look for that indicate a broken implementation

| Observed exposure            | Diagnosis                                             |
|------------------------------|-------------------------------------------------------|
| Momentum β > 0.2 on a value book    | Using Fama-French HML off Ken French — it has the Devil contamination. Rebuild with current prices. |
| Market β > 0.1 (abs)         | Forgot to dollar-neutralize, or hedging leg dropped.  |
| Large sector tilt            | Forgot to industry-neutralize the signals.            |
| Size β > 0.3                 | Accidentally running a small-cap bet; your universe screen is too weak. |
| BAB β > 0 on a "pure momentum" book | Expected — momentum is mildly long low-beta *except* during crashes. |
| Quality β > 0 on a "pure value" book | Pure value loads NEGATIVELY on quality. Check your construction. |

## 6. The monthly diagnostic pack AQR researchers actually look at

For any live factor book, compute and review monthly:

1. Value spread on composite measure (see table in §3).
2. Rolling 12-month Sharpe of each factor, vs its full-history Sharpe.
3. Rolling correlation of value and momentum returns (normal is
   strongly negative; convergence to zero is a warning).
4. Turnover vs budget.
5. Short-book borrow cost distribution (mean + p95).
6. Gross-to-net Sharpe gap; alarm if it widens materially.
7. Factor exposures from holdings-based attribution.
8. Contribution to drawdown: per-factor, per-region, per-sector.

If all eight diagnostics are "normal for a drawdown" and the spread
is wide, the correct action is **do nothing**. Capitulating here is
the single most expensive mistake in factor investing.
