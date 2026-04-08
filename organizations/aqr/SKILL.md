---
name: aqr-factor-investing
description: AQR-style factor-investing heuristics for building and diagnosing live value, momentum, quality, defensive, and size-when-quality-controlled portfolios. Use when constructing long-only or long-short factor books, fixing backtests that look too academic, deciding how to integrate factors, model costs, or interpret a brutal drawdown, or when terms like HML Devil, 12-1 momentum, QMJ, BAB, integrated portfolio, value spread, factor timing, size minus junk, or implementation shortfall appear.
tags: quantitative, factor-investing, risk-management, portfolio, backtesting, finance
---

# AQR Factor Investing

This skill is for the gap between a pretty academic factor and a book you can
actually survive in live capital. The AQR lesson is that data lags, neutrality
choices, turnover management, portfolio integration, and governance usually
matter at least as much as the headline factor.

## Before you build anything, ask yourself

1. Am I trying to replicate Ken French or implement AQR-style live factor
   investing? If it is the latter, lag accounting data by at least six months
   but do not lag price.
2. Does the premium survive after removing microcaps? If it vanishes outside
   the bottom tail, you found a liquidity artifact, not a durable factor.
3. Is this long-only core equity or long-short factor extraction? Long-only
   wants integrated scores and tax-aware turnover; long-short wants cleaner
   factor isolation, explicit neutrality, and a risk target.
4. Am I willing to accept tracking error to the academic portfolio to save
   costs and taxes? AQR's live answer is usually yes.
5. If this factor spends five bad years getting cheaper, what exact evidence
   would make me cut it? Decide now, not inside the drawdown.

## Where the information ratio actually comes from

- AQR's large-cap appendix is the right mental model. Simple cap-weighted
  top-50% value had about 0.05 IR. Using current price lifted it to 0.14.
  Multiple value measures lifted it to 0.29. Concentrating into the top 25%
  and blending cap and signal weighting lifted it to 0.56. Adding momentum and
  profitability lifted it to 0.91. Theme selection mattered less than
  construction quality.
- Treat backtest Sharpe as a sketch, not a forecast. AQR's century study found
  factor premia roughly 30% lower out of sample than in sample even before
  governance mistakes, financing frictions, and taxes.
- "Craftsmanship alpha" should be judged as a bundle. Many 0.1-Sharpe
  decisions will look statistically weak one by one and still be indispensable
  together.
- Real trading costs can be far below legacy academic estimates. AQR's
  $1.7T live-trade study found actual costs about an order of magnitude
  smaller than many older papers suggested. Do not kill a high-turnover signal
  with toy cost assumptions.

## Construction rules that change the answer

### Value

- Lag fundamentals, not price. The six-month gap exists to avoid look-ahead on
  accounting data; applying the same lag to price is the Devil error.
- Use a composite. Default to at least four measures such as B/P, E/P, CF/P,
  and sales-to-price or enterprise-value variants. Single-ratio value is too
  noisy to deserve capital.
- Rank within industry. Cross-industry value quietly becomes a sector macro bet
  right when you most want a valuation signal.
- If you are approximating AQR's public long-short valuation series, the
  relevant defaults are monthly rebalance, best 30% versus worst 30%,
  market-cap weighting, industry neutrality, and about 7% annualized
  volatility target.

### Momentum

- Default to 12-1 with the skip-month. The last month is where reversal and
  microstructure junk leak in.
- Do not fetishize exact replication of the academic portfolio. AQR's momentum
  implementation work accepts some factor tracking error if it buys a large
  reduction in costs or taxes.
- A momentum screen can reduce turnover inside a value book rather than raise
  it. In AQR's appendix, adding momentum screens cut turnover from about 63% to
  the high-50s while turning a negative UMD loading positive.
- Do not purge the expensive tail. Momentum's best payoffs often live there;
  cleansing "obvious bubbles" feels prudent and usually amputates the strategy.

### Quality, defensive, and size

- QMJ is not just profitability. AQR's version uses profitability, growth,
  safety, and payout together because each rescues the others when accounting
  or regime noise hits.
- BAB is beta-neutral, not merely "owns low-vol stocks." If you do not lever or
  beta-match the low-beta leg, you built a defensive tilt, not BAB.
- Size only belongs after you control for junk. Plain SMB is too often a
  portfolio of low-quality distress with heroic backtest marketing wrapped
  around it.

### Long-only portfolio construction

- Integrate factors before portfolio construction. AQR's U.S. large-cap example
  showed integrated value-momentum-profitability beating the sleeve-mix
  approach on both Sharpe and volatility.
- Blend cap and signal weighting. Pure cap-weight dilutes the signal; pure
  equal-weight imports microcap noise, tax drag, and capacity lies.
- In international books, country neutrality is not cosmetic. AQR notes that
  holding country weights closer to benchmark increases the useful negative
  correlation between value and momentum.

## Decision tree: what are you actually building?

- If the goal is factor research or exposure attribution, build clean long-short
  portfolios with explicit neutrality, monthly rebalancing, and a volatility
  target.
- If the goal is a taxable core-equity product, build one integrated long-only
  book, allow some tracking error to the academic factor, and optimize trading
  and tax lots before chasing purer signals.
- If the goal is public-factor replication, copy the paper exactly and label it
  replication, not implementation.
- If the goal is live implementation, start from AQR-style defaults and only
  simplify when you can defend the lost information ratio in dollars, taxes, or
  governance capacity.
- If you cannot model borrow, impact, and financing honestly, do not fake
  precision. Shrink toward larger-cap names or move to long-only until the
  short-book economics are credible.
- If governance cannot survive multi-year droughts, solve that with lower
  leverage or broader factor diversification, not with improvised factor
  timing.

## NEVER / INSTEAD

- NEVER lag price along with accounting data because the lag is only justified
  for stale fundamentals; lagging price quietly injects stale momentum
  avoidance into value. Instead lag the accounting variables and recompute the
  denominator with current price every rebalance.
- NEVER reject a signal because older cost papers say it is "untradeable"
  because that false prudence is built on crude cost models and academic
  rebalance mechanics; the consequence is throwing away implementable alpha.
  Instead model spread, impact, and borrow separately and allow controlled
  tracking error to the paper factor.
- NEVER combine long-only factor sleeves by averaging their portfolios because
  the seductive part is governance simplicity, but the consequence is that
  stocks attractive on multiple styles cancel against one-style names. Instead
  aggregate the signals first and build one portfolio from the composite score.
- NEVER add size raw because the seductive part is the famous SMB chart, but
  the consequence is a portfolio dominated by junky small firms with ugly
  crashes, borrow pain, and weak live capacity. Instead add size only after
  explicit junk or quality control.
- NEVER "fix" momentum by excluding very expensive stocks because the seductive
  part is narrative comfort during bubbles, but the consequence is destroying a
  part of the payoff AQR found especially important. Instead let value fight
  the expensive tail and let momentum own it when it is still working.
- NEVER run continuous valuation-based factor timing because the seductive part
  is the truism that "price matters," but the consequence is paying turnover to
  fight a value premium you already own. Instead keep risk targets steady and
  reserve any timing sin for rare, explicit extremes.
- NEVER use trigger-band rebalancing as your default because it looks adaptive,
  but the consequence is trading against time-series momentum exactly when
  trends are strongest. Instead use a calendar schedule and only widen it when
  your explicit turnover budget requires it.

## Drawdown triage

1. Did the factor get cheaper because prices moved or because fundamentals
   worsened? Price-only pain is compression; worsening fundamentals is a
   different problem.
2. Did the value spread widen versus the start of the drawdown? Wider usually
   means higher expected return, not a broken signal.
3. Is the damage concentrated in the short leg during a violent rebound? That
   is usually a momentum-crash signature, not proof the factor died.
4. Did borrow costs or financing frictions jump on the short book? Crowding
   shows up there before it fully shows up in returns.
5. Are you tempted to change signal weights because of pain rather than
   precommitted evidence? That is the -5 Sharpe strategy Asness warns about.
6. If you truly must de-risk, cut total portfolio risk first. Changing leverage
   is usually less destructive than rewriting the signal stack in the middle of
   a panic.

## Mandatory references

- Before writing factor-construction code, READ `references/construction-code.md`.
- Before running any long-short backtest or capacity memo, READ
  `references/backtest-frictions.md`.
- Before explaining live P&L or deciding whether a drawdown is decay versus
  compression, READ `references/attribution.md`.
- Do NOT load the reference files for conceptual debates, asset-allocation
  conversations, or "should we believe factors at all?" discussions. They are
  implementation documents and will bias you toward premature concreteness.
