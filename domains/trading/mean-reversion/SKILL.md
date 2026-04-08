---
name: scarface-mean-reversion
description: "Trade dislocations in the style of Scarface Trades: fade statistically stretched moves only after anchor quality, regime, catalyst, and time-decay checks. Use when evaluating Bollinger/%B/z-score/VWAP/weekly-open/pairs-trading mean-reversion setups, deciding whether an extreme is exhaustion or a band-walk continuation, sizing a fade, or building a short-horizon reversion model. Trigger on: mean reversion, Bollinger Bands, z-score, VWAP reversion, weekly open, overextended move, buy the flush, sell the rip, pair spread, cointegration, half-life, fade extreme."
---

# Scarface Mean Reversion⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌‌​‌‌​‍​‌​‌‌‌​​‍‌‌‌​‌‌‌‌‍​‌​​​​​​‍​​​​‌​‌​‍​‌​‌‌​​‌⁠‍⁠

Mean reversion is a dislocation business, not an "oversold" business. The edge is not that price looks extreme; the edge is that price is extreme relative to an anchor other participants actually defend, and the move is occurring in a regime where the anchor still matters.

This skill is self-contained. Do not load generic RSI or Bollinger primers for normal use; they dilute the signal. Load external material only if the user explicitly asks for indicator math or implementation code.

## Operating Stance

- A moving average is not automatically a destination. If the mean is drifting faster than the expected snap-back, you do not have a reversion trade; you have a trend pretending to be cheap.
- Fixed anchors beat floating anchors when you need clean execution logic. For index futures, weekly open and session VWAP often outperform generic rolling means because everyone sees the same level and there is no parameter drift.
- High-win-rate reversion systems naturally hide catastrophic loss. Judge setups by worst-case path, not average signal quality.
- Reversion is front-loaded. If the move does not start snapping back quickly, probability usually decays faster than traders admit.

## Before You Fade Anything, Ask Yourself

1. What is the anchor?
   Fixed anchor like weekly open, prior close, or session VWAP; rolling anchor like Bollinger mean; or spread residual from a hedged pair. The more subjective the anchor, the less aggressive the trade.
2. Is this distance or information?
   A panic flush inside a stable regime is distance. Earnings, guidance cuts, macro surprises, broken pegs, secondary offerings, or exchange outages are information. Information does not have to revert.
3. Is the anchor stationary enough to matter?
   Single-name price series after a catalyst are usually poor mean-reversion candidates. Pair spreads and index dislocations behave better because the anchor is less narrative-dependent.
4. Where is the clock?
   Estimate the expected decay window before entry. If you cannot state when the trade should start working, you cannot know when the thesis is dead.
5. Can the trade pay friction?
   Mean-reversion targets are smaller than momentum targets. If the expected snap-back is only a small multiple of fees, spread, borrow, and funding, the edge is fake.

## Anchor Selection

| Situation | Preferred anchor | Why it works | Stand aside when |
|---|---|---|---|
| Index futures weekly dislocation | Weekly open | Objective, widely watched, no lookback drift | The distance is so large that price is repricing, not wobbling |
| Intraday exhaustion in liquid index products | Session VWAP or prior day VWAP | Real intraday inventory anchor for dealers and execution algos | The open drive is still being accepted and price is walking away from VWAP |
| Mature range in a liquid single name | Bollinger mean or %B | Useful only when bandwidth is stable and the tape is rotational | The band is expanding and price keeps accepting outside it |
| Relative-value pair | Spread residual with dynamic hedge ratio | Edge comes from spread stationarity, not raw price cheapness | Hedge ratio drifts or cointegration only exists in one regime window |

For NQ-style weekly-open trades, distance matters more than the mere existence of a gap. In one 565-week study, Tuesday opened back to the weekly open 92.9% of the time when the deviation was under 0.25%, but only 45.5% when the deviation exceeded 1.5%. Most successful touches happened early: roughly two-thirds on Tuesday, nearly half of first touches in the first 30 minutes of RTH, and if the level had not been touched by the end of Wednesday the remaining Thursday-Friday probability was only about 16%. Treat large dislocations as possible trend weeks, not "better bargains."

## Regime Filters That Actually Matter

- Do not short or buy because price merely tagged a band. John Bollinger's own "walk the band" behavior matters more: repeated closes on the outer band during expanding bandwidth are continuation, not reversal.
- The first close outside the band after a squeeze is usually a directional release, not a fade. Fade only after the move stops gaining acceptance: failed follow-through, rejection back inside the band, and loss of impulse quality.
- The default 20-period / 2-sigma Bollinger setting is not sacred. If you shorten the lookback to around 10, tighten the deviation multiplier toward 1.9; if you lengthen toward 50, widen it toward 2.1. Using 2.0 everywhere creates false comparability across timeframes.
- For long-only equity index dip buying, the Connors-style filter is a useful template: price above the 200-day average, VIX at least 5% above its 10-day average for 3 or more days, and exit on RSI(2) strength above 65. The point is not the exact numbers; the point is that volatility shock inside an intact bull regime is a different trade from generic oversold.

## Execution Rules

- Probe, do not full-size, on the first touch. First touch is where reversion feels safest and is most likely to be a continuation trap.
- Add only on a preplanned ladder. A valid add is part of the initial map; an emotional add is disguised loss aversion.
- Use time stops as seriously as price stops. On OU-style spreads, estimate half-life first. Faster reversion supports tighter entry/exit spacing; higher volatility and higher transaction costs require wider thresholds. If there is no believable half-life, there is no trade.
- As a working rule, if the position has not meaningfully reverted within about 2 to 3 half-lives, assume the relationship changed and get out. Waiting longer usually converts a mean-reversion thesis into an unpriced regime bet.
- Increase target distance when volatility or friction rises. High volatility does not mean "easier reversion"; it often means you must demand more room before entering and more distance before exiting.
- For pair trades, do not trust a relationship built on a tiny sample. Use roughly a year of daily data as a minimum sanity window, then re-check the relationship on rolling sub-windows and refresh the pair list periodically; even good pairs often decay within a year or two.

## Decision Tree

If the move was caused by earnings, macro news, a hard catalyst, or a structural break:
- Do not run mean reversion by default.
- Fallback: switch to post-event momentum or wait for a new balance area to form.

If the anchor is fixed and nearby, the regime is stable, and the move is small relative to recent realized range:
- Mean reversion is allowed.
- Fallback: if price does not start reverting in the expected session window, cut size or exit; front-loaded edges decay fast.

If the tape is walking the band, bandwidth is expanding, and every pullback is shallow:
- Do not fade the extreme.
- Fallback: wait for a failed continuation and reclaim back inside the structure.

If a pair spread looks cheap but the hedge ratio is drifting or cointegration vanishes outside one backtest window:
- Do not assume stationarity.
- Fallback: re-estimate hedge ratio on a rolling basis, test stability across sub-windows, or abandon the pair.

If borrow, funding, or spread costs widen enough that the expected snap-back barely clears friction:
- Do not trade the setup just because the chart looks statistically clean.
- Fallback: widen thresholds, reduce size, or wait for a larger dislocation.

## NEVER

- NEVER fade the first close outside a band after compression because it feels like "maximum stretch." That setup is seductive precisely because the chart looks extreme, but it is often the start of range expansion. Instead wait for failed continuation and acceptance back inside the band structure.
- NEVER use a rolling mean as if it were an objective magnet because moving averages make every chart look tidy. The seduction is visual neatness; the consequence is fading a mean that is actively running away from you. Instead prefer fixed anchors when available and demand stronger confirmation when the anchor floats.
- NEVER average down just because the z-score got larger. That feels mathematically smarter because the average entry improves, but one structural break can erase months of small wins. Instead define probe levels, add levels, and invalidation before the first order.
- NEVER treat event-driven dislocations as "extra oversold." The move feels safer because the candle is large and indicators are pinned, but the market may be repricing new information rather than overshooting. Instead let the event digest or trade a different framework.
- NEVER hold a reversion trade without a clock because high win rates train traders to say "it always comes back." The consequence is turning a short-horizon statistical trade into an open-ended fundamental opinion. Instead use half-life or session-window decay and exit when the thesis ages out.
- NEVER trust pair stationarity from one historical sample because the backtest looks clean. The seductive part is a smooth equity curve; the consequence is trading yesterday's hedge ratio in today's regime. Instead re-check stability across rolling windows and monitor hedge-ratio drift.
- NEVER ignore friction because reversion targets look numerically close and achievable. The consequence is a strategy that is right often and still loses live after fees, slippage, funding, and borrow. Instead require materially more gross edge as costs rise.

## If You Are Building a System

- Bucket results by distance-from-anchor, not just signal/no-signal. Edge usually changes nonlinearly with distance.
- Separate event days from normal days. A strategy that looks strong in aggregate can be carried by quiet regimes and destroyed by catalysts.
- Inspect the worst 1% of losses before optimizing entries. Reversion systems die from tail events, not from average trade quality.
- For pairs, test cointegration and hedge-ratio stability on rolling sub-windows, not only the full sample.
- Walk forward by regime. A parameter set that survives only one volatility regime is not robust.

## Practical Heuristics

- The best fade is usually a stable regime, a respected anchor, and a move that is stretched enough to matter but not so large that the market is repricing.
- Small dislocations around objective anchors often outperform dramatic chart extremes around subjective means.
- If a setup needs multiple narratives to justify why the reversion is "still coming," it is usually no longer a Scarface-style trade.
