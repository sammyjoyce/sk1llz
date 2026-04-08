# Backtest Frictions — Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​​​​‌‍‌‌‌‌‌​‌​‍​​​​​​‌‌‍‌​​‌‌‌​​‍​​​​‌​​‌‍‌​‌‌​​‌​⁠‍⁠

**Read this before running any long-short factor backtest.** The gap
between paper Sharpe and live Sharpe is dominated by the items in this
file. AQR's own estimate, from ~$1T of live trades at Frazzini-Israel-
Moskowitz (2013) benchmarks, is that academic costs over-estimate drag
for large-cap liquid names and under-estimate it for small-caps.

## The three frictions you must model separately

1. **Half-spread + commission** (linear in trade size)
2. **Market impact** (square-root in trade size, relative to ADV)
3. **Borrow cost** (carried daily on the short book, NOT on trade dates)

Modeling these as one blob (e.g. "5 bps per trade") is the #1 reason
backtests overstate Sharpe.

## 1. Spread + commission (linear)

```python
def spread_cost(trade_notional: pd.Series,
                half_spread_bps: pd.Series,
                commission_bps: float = 0.5) -> pd.Series:
    """
    Linear trading cost: you cross half the bid-ask spread on average
    plus pay commission.

    `half_spread_bps` is stock-specific — pull from TAQ or Refinitiv.
    For a rough live-calibrated default (Frazzini-Israel-Moskowitz 2013):
        - Large-cap developed:  ~3-5 bps per side
        - Small-cap developed:  ~10-20 bps per side
        - Emerging markets:     ~15-40 bps per side
    Commission ~0.5 bp for institutional equity in 2020s.
    """
    return trade_notional.abs() * (half_spread_bps + commission_bps) / 1e4
```

## 2. Market impact (square-root)

```python
def market_impact_cost(trade_notional: pd.Series,
                       adv_notional: pd.Series,
                       daily_vol: pd.Series,
                       lam: float = 0.1) -> pd.Series:
    """
    Almgren-Chriss / square-root impact model as calibrated to AQR live
    trading (Frazzini-Israel-Moskowitz 2013).

        impact_bps = lam * daily_vol_bps * sqrt(trade / ADV)

    Typical `lam` ≈ 0.10 for large-cap US, ≈ 0.20 for small-cap/international.

    CRITICAL: impact scales with sqrt(participation rate), NOT linearly.
    This is why AQR pays so much attention to trade scheduling (VWAP
    over the day) and WHY passive rebalance beats trigger rebalance —
    triggers force concentrated trading windows.
    """
    participation = (trade_notional.abs() / adv_notional).clip(upper=1.0)
    impact_bps    = lam * (daily_vol * 1e4) * np.sqrt(participation)
    return trade_notional.abs() * impact_bps / 1e4
```

## 3. Borrow cost (the one people forget)

```python
def borrow_cost(short_positions_notional: pd.Series,
                borrow_rate_annual: pd.Series,
                days_held: int = 1) -> pd.Series:
    """
    Daily borrow fee on short positions.

    - General collateral (GC) stocks: ~25-50 bps annualized.
    - "Hard-to-borrow" (HTB) names: can exceed 10-50% annualized.
    - Borrow is paid DAILY while the position is held, not per trade.
      A backtest that charges it "per trade" will dramatically underestimate
      drag on a low-turnover short book.

    Short-side crowding in your strategy will show up HERE before it
    shows up in the return series. A sudden jump in borrow rates on
    your short book is a leading indicator of decaying alpha.
    """
    return short_positions_notional.abs() * borrow_rate_annual * (days_held / 252)
```

## 4. The backtest loop skeleton

```python
def realistic_backtest(strategy, start, end, initial_capital=1e8):
    capital   = initial_capital
    positions = pd.Series(dtype=float)
    rows      = []

    for t in trading_days(start, end):
        target = strategy.generate_positions(t, capital)
        trades = (target - positions).fillna(target)

        spread    = spread_cost(trades, data.half_spread(t))
        impact    = market_impact_cost(trades, data.adv(t), data.daily_vol(t))
        shorts    = positions[positions < 0]
        borrow    = borrow_cost(shorts, data.borrow_rate(t))
        trade_fee = (spread + impact).sum()

        capital  -= trade_fee
        positions = target

        px_ret    = data.returns(positions.index, t)
        gross_pnl = (positions * px_ret).sum()
        net_pnl   = gross_pnl - trade_fee - borrow.sum()
        capital  += net_pnl

        rows.append({
            't': t, 'gross_pnl': gross_pnl, 'spread': spread.sum(),
            'impact': impact.sum(), 'borrow': borrow.sum(),
            'net_pnl': net_pnl, 'capital': capital,
            'turnover': trades.abs().sum() / capital,
        })
    return pd.DataFrame(rows)
```

## Sanity checks for a finished backtest

If any of these fail, the backtest is wrong — not the strategy:

- **Turnover sanity.** Monthly-rebalanced 12-1 momentum should produce
  ~200-400% annual turnover. Value ~40-80%. A "momentum" strategy with
  50% turnover is not momentum.
- **Impact ≈ half-spread for a large book.** On a $10B book in liquid
  names, impact cost should be on the same order as spread cost. If
  impact is 10x spread, your `lam` is mis-calibrated.
- **Borrow cost ≈ 20-50 bps annualized on the short book.** If much
  higher, your short book is concentrated in HTB names and the strategy
  cannot scale.
- **Gross vs net Sharpe gap ≈ 0.2-0.4** for a ~$1B long-short book in
  liquid large-caps. If the gap is <0.1, you are under-costing. If >0.6,
  the strategy is not implementable at scale.
- **"Paper-to-real" Sharpe discount.** After all frictions, further
  discount by 30-50% when setting client expectations. This is standard
  AQR practice (Asness has published it).

## The "implementation shortfall" framing

Measure drag in the same units as alpha:

```
implementation_shortfall = gross_return - net_return
                         = spread + impact + borrow + missed_trades
                         ≈ 1-3% annualized for a $1B long-short book in
                           liquid US stocks
                         ≈ 3-6% annualized for the same strategy in EM
```

An "alpha" of 2% annualized that looks beautiful gross of costs is
break-even-to-negative after realistic frictions in EM. This is why
the capacity of a factor is not a single number — it is a function of
the universe's liquidity.
