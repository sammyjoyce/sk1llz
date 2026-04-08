# Paradigms Area Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​‌​​‌‌​‍​‌‌‌​‌‌‌‍​‌​‌‌‌‌​‍​​‌‌‌​​‌‍​​​​‌​​‌‍​‌​​‌‌​​⁠‍⁠

- For paradigm SKILL rewrites, optimize for trade-off clarity: explicitly compare competing patterns (ordering, abstraction depth, and error behavior) and explain why one choice is safer under strict production constraints.
- In monadic/functional skill documentation, prioritize anti-pattern consequences that are painful to debug (semantic drift from transformer order, accidental duplication of effects, and space-leak risk) instead of restating canonical monad laws.
- Keep references to concrete examples in `references/` subpaths when details are long or code-heavy, and keep the skill file itself focused on decision gates, failure signals, and fallback strategies.
