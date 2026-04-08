# Hypothesis Code Patterns Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌​‌‌​​‍‌​‌‌‌‌‌‌‍‌​​​‌‌‌​‍​​‌​​‌​‌‍​​​​‌​‌​‍​‌‌‌​‌‌​⁠‍⁠

Load this file when writing Hypothesis tests. Contains battle-tested patterns.

## Strategy Selection Decision Tree

```python
# CHOOSING THE RIGHT STRATEGY COMBINATOR
#
# Need dependent values?  → @st.composite (draw A, then draw B based on A)
# Need to transform?     → .map() (preserves shrinking quality)
# Need to reject values? → .filter() for strategies, assume() inside test body
# Need recursive data?   → st.recursive() with max_leaves (start at 50, tune down if slow)
# Need from existing?    → st.sampled_from() for enums/small sets
# Need type-inferred?    → st.from_type() (but register custom strategies for complex types)
# Need to reuse draws?   → st.shared() with explicit key= to prevent cross-contamination
```

## Composite Strategy — The Right Way

```python
@st.composite
def valid_user(draw):
    # GOOD: Constrain at generation time, not post-hoc filtering
    name = draw(st.text(
        alphabet=st.characters(codec="ascii", categories=("L",)),
        min_size=1, max_size=50
    ))
    age = draw(st.integers(min_value=0, max_value=150))
    # Dependent generation: email domain depends on earlier draw
    domain = draw(st.sampled_from(["example.com", "test.org"]))
    email = f"{name.lower().replace(' ', '.')}@{domain}"
    return User(name=name, age=age, email=email)

# WRONG: Don't generate then filter — rejection rate kills performance
# @st.composite
# def valid_user_bad(draw):
#     name = draw(st.text())  # Generates empty strings, unicode, etc
#     assume(len(name) > 0 and name.isascii())  # >90% rejection!
```

## Stateful Testing with Bundles

```python
from hypothesis.stateful import (
    RuleBasedStateMachine, rule, invariant,
    initialize, precondition, Bundle, consumes
)
import hypothesis.strategies as st

class APIStateMachine(RuleBasedStateMachine):
    """Test an API against a reference model."""

    def __init__(self):
        super().__init__()
        self.model = {}  # Reference model
        self.api = APIClient()  # System under test

    # Use @initialize for one-time setup that produces Bundle items
    # @initialize runs exactly once before rules, but AFTER __init__
    created_ids = Bundle("created_ids")

    @initialize(target=created_ids)
    def seed_item(self):
        """Ensure at least one item exists before rules run."""
        resp = self.api.create({"name": "seed"})
        self.model[resp.id] = {"name": "seed"}
        return resp.id

    @rule(target=created_ids, data=st.dictionaries(
        st.text(min_size=1, max_size=20),
        st.text(max_size=100),
        min_size=1, max_size=5
    ))
    def create(self, data):
        resp = self.api.create(data)
        self.model[resp.id] = data
        return resp.id

    # consumes() removes item from Bundle — prevents use-after-delete bugs
    @rule(item_id=consumes(created_ids))
    def delete(self, item_id):
        self.api.delete(item_id)
        del self.model[item_id]

    # precondition prevents rule from running when condition is false
    @precondition(lambda self: len(self.model) > 0)
    @rule(item_id=created_ids)
    def read(self, item_id):
        result = self.api.get(item_id)
        assert result == self.model[item_id]

    @invariant()
    def count_matches(self):
        assert self.api.count() == len(self.model)

TestAPI = APIStateMachine.TestCase
```

## Settings Profiles — CI vs Dev

```python
from hypothesis import settings, HealthCheck, Phase, Verbosity

# Register BEFORE tests import them
settings.register_profile(
    "ci",
    max_examples=1000,
    deadline=None,  # CI machines have variable speed
    suppress_health_check=[],  # Strict in CI
    derandomize=True,  # Reproducible CI runs
)
settings.register_profile(
    "dev",
    max_examples=100,
    deadline=400,  # Generous but present
    suppress_health_check=[HealthCheck.too_slow],
)
settings.register_profile(
    "debug",
    max_examples=10,
    verbosity=Verbosity.verbose,
    phases=[Phase.explicit, Phase.reuse],  # Only replay known failures
)
# conftest.py:
# import os; settings.load_profile(os.getenv("HYPOTHESIS_PROFILE", "dev"))
```

## target() for Coverage-Guided Generation

```python
from hypothesis import given, target
import hypothesis.strategies as st

@given(st.lists(st.integers(), min_size=1))
def test_sort_performance(xs):
    """Guide Hypothesis toward harder inputs."""
    sorted_xs = sorted(xs)
    # target() biases generation toward higher values of this metric
    # Hypothesis will try to maximize it, finding worst-case inputs
    target(float(len(xs)), label="list_length")

    # You can target multiple metrics independently
    inversions = sum(1 for i in range(len(xs)-1) for j in range(i+1, len(xs))
                     if xs[i] > xs[j])
    target(float(inversions), label="inversions")
    assert sorted_xs == sorted(sorted_xs)

# target() is the closest thing to coverage-guided fuzzing in Hypothesis.
# Use it when you suspect bugs lurk in large/complex inputs but
# default generation biases toward small/simple ones.
```

## Recursive Strategies — Controlling Explosion

```python
# CRITICAL: max_leaves controls tree size and generation time
# Too high → generation timeout (HealthCheck.too_slow)
# Too low  → never finds deep-nesting bugs
json_strategy = st.recursive(
    st.none() | st.booleans() | st.integers() | st.floats(allow_nan=False) | st.text(max_size=20),
    lambda children: (
        st.lists(children, max_size=5) |
        st.dictionaries(st.text(max_size=10), children, max_size=5)
    ),
    max_leaves=30  # Start here. Increase only if you need deeper structures.
)

# WARNING: Unbounded text() inside recursive strategies explodes generation time.
# Always set max_size on text/binary/lists inside recursive strategies.
```
