# Regex: NFA vs. Backtracking — Expert Decision Guide

Load this file when: writing regex that will see untrusted input, debugging a regex that "hangs", or choosing between stdlib regex and a DFA engine.

## The rule, first

**Any regex that touches untrusted input must run on a guaranteed-linear-time engine.** Period. No audit process catches every pathological pattern. Pick the engine; don't police the patterns.

## The two worlds

| | **Backtracking (PCRE, Perl, Python `re`, Java, JS V8, Ruby, .NET)** | **NFA lock-step (RE2, Go `regexp`, Rust `regex`, Hyperscan)** |
|---|---|---|
| Worst case | Exponential (`O(2^n)`) | Linear (`O(mn)`) |
| Backreferences `\1` | Yes | No |
| Lookaround | Usually | RE2: limited; Rust: no |
| Submatch extraction | Yes | Yes (Thompson/Pike 1985 technique) |
| Pathological patterns | `^(a+)+$`, `(a|a)*`, `(.*)*`, nested unbounded | None — truly linear |
| Implementation size | 10k–100k LOC | Thompson's 1968 version: < 400 LOC in C |

## How to spot ReDoS-vulnerable patterns

Backtrackers explode on *ambiguity*: when the same input can be matched multiple ways, the engine tries them all. The red-flag shapes:

- **Nested quantifiers**: `(a+)+`, `(a*)*`, `(x+x+)+y` — the inner quantifier's boundary is undecidable, so every split is tried.
- **Overlapping alternations under a quantifier**: `(a|a)*`, `(a|ab)*`, `(\w|\d)*` — each character has multiple matching branches.
- **Quantifier + anchor**: `^(a+)+$` — the anchor forces the whole engine to explore every partition of the input before giving up.
- **`.*` sandwiches**: `.*foo.*bar.*baz.*` on input without `baz`. Each `.*` backtracks independently.

**The 30-character rule**: on a vulnerable pattern, input length 30 usually takes seconds; 35 takes a minute; 40 takes an hour. If your attacker can send 40 characters, you have an outage vector.

## Why Thompson's 1968 algorithm is linear

The insight from the CACM paper: **maintain the full set of NFA states the machine could be in after reading `k` characters**, and advance all of them in lock-step when reading character `k+1`. There are at most `m` NFA states (where `m` is pattern length), so each character costs `O(m)`, total `O(mn)`. No position in the input is ever revisited. No choice is ever "tried and rolled back."

Thompson's 1968 implementation compiled the NFA into IBM 7094 *machine code* on the fly — each NFA state was a small instruction sequence, each transition a jump. Russ Cox's modern C translation (Plan 9 / `re1`) does the same job in under 400 LOC and beats Perl/PCRE by 10⁶× on `(a?)^n a^n` pathological inputs. The algorithm has been public for 57 years; every language that still ships a backtracker is a choice, not an inevitability.

## Concrete migration guide (Python example)

```python
# BAD: Python's re is a backtracker. ReDoS if pattern or input is attacker-controlled.
import re
if re.match(user_pattern, user_input):  # outage in one request
    ...

# GOOD: google/re2 Python binding. Drop-in for most patterns; rejects backreferences.
import re2
try:
    if re2.match(user_pattern, user_input):
        ...
except re2.RegexError:
    # re2 rejects patterns it can't compile to DFA — this is good; log and reject.
    ...
```

Go and Rust are already safe by default (`regexp` = RE2; `regex` crate = NFA). Java has `RE2/J`. JavaScript has no good answer in Node's stdlib — use `re2` npm bindings if the input is untrusted.

## When backtracking is actually OK

- **Fixed patterns on fixed-size inputs** where you have measured the worst case.
- **Offline tooling** (codegen, linting) where an infinite loop is annoying but not a security incident.
- **Patterns that genuinely need backreferences**, e.g. HTML tag matching `<(\w+)>.*</\1>`. Even then, cap input size and add a timeout.

## Thompson's own inconsistency (and why)

Thompson invented the linear-time NFA algorithm in 1968 but used **backtracking** in `ed` and `grep` when he wrote them in 1971/1973. Why? Because real regex patterns in the Unix shell rarely triggered the exponential case, and the backtracker was half the code. This is the core Thompson move: **know the pathological case exists, know you could fix it, do the simpler thing until measurement says otherwise.** The modern update: measurement now says otherwise, because the internet sends you adversarial inputs. Pick the linear engine.

## Submatch extraction in an NFA

The "obvious" objection is that backtrackers naturally track `$1`, `$2`, … while NFAs don't. Rob Pike solved this in the 1985 `sam` editor: each NFA thread carries an auxiliary array of saved input positions, and the `save i` bytecode instruction records the current position into slot `i`. The technique disappeared from textbooks for ~25 years — Russ Cox rediscovered it when writing RE2. When someone tells you "NFAs can't do submatching," they are citing 1970s pedagogy, not 1985 engineering.

## References

- Thompson, K. (1968). *Regular expression search algorithm*. CACM 11(6). https://doi.org/10.1145/363347.363387
- Cox, R. (2007). *Regular Expression Matching Can Be Simple And Fast*. https://swtch.com/~rsc/regexp/regexp1.html
- Cox, R. (2009). *Regular Expression Matching: the Virtual Machine Approach*. https://swtch.com/~rsc/regexp/regexp2.html
- Cox, R. (2010). *Regular Expression Matching in the Wild*. https://swtch.com/~rsc/regexp/regexp3.html
