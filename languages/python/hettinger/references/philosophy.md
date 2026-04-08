# Hettinger Patterns — Code Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌​​​‌​‍​​​​‌​​​‍‌‌​​​​​‌‍​‌‌​​‌‌​‍​​​​‌​‌​‍​​‌​‌​​‌⁠‍⁠

Load this file when you need specific before/after code transformations.

## Iterator Algebra Patterns

```python
from itertools import chain, groupby, product, combinations, islice, accumulate, starmap, repeat, tee

# Flatten one level (not recursive — use recursion or more_itertools for deep)
nested = [[1, 2], [3, 4], [5, 6]]
flat = list(chain.from_iterable(nested))  # [1, 2, 3, 4, 5, 6]

# Running totals — accumulate replaces manual running-sum loops
from itertools import accumulate
balances = list(accumulate([100, -20, 50, -10]))  # [100, 80, 130, 120]

# starmap: apply function to pre-packed argument tuples
from itertools import starmap
coords = [(2, 5), (3, 2), (10, 3)]
areas = list(starmap(pow, coords))  # [32, 9, 1000]

# tee: split a single iterator into N independent copies
# WARNING: if one copy advances far ahead, tee buffers everything — can OOM
it1, it2 = tee(range(1_000_000), 2)
# Safe: consume both in lockstep. Dangerous: consume it1 fully, then it2.

# Product replaces nested loops — but watch the cardinality:
# product(range(100), range(100), range(100)) = 1M iterations, easy to miss
for x, y, z in product(xs, ys, zs):
    process(x, y, z)
```

## collections Patterns

```python
from collections import Counter, defaultdict, deque, namedtuple, ChainMap

# Counter arithmetic — the non-obvious parts
a = Counter(cats=3, dogs=1)
b = Counter(cats=1, dogs=4)
a - b          # Counter({'cats': 2}) — drops zero/negative
a + b          # Counter({'dogs': 5, 'cats': 4})
a & b          # Counter({'cats': 1, 'dogs': 1}) — min per key
a | b          # Counter({'dogs': 4, 'cats': 3}) — max per key
+a             # Strips non-positive counts

# deque as sliding window (maxlen is the trick)
from collections import deque
def sliding_window(iterable, n):
    it = iter(iterable)
    window = deque(maxlen=n)
    for _ in range(n):
        window.append(next(it))
    yield tuple(window)
    for item in it:
        window.append(item)  # auto-evicts leftmost
        yield tuple(window)

# deque.rotate — O(1) rotation vs O(n) list slice
d = deque([1, 2, 3, 4, 5])
d.rotate(2)   # deque([4, 5, 1, 2, 3]) — right rotation
d.rotate(-1)  # deque([5, 1, 2, 3, 4]) — left rotation

# ChainMap for layered config (CLI > env > file > defaults)
defaults = {'color': 'blue', 'user': 'guest'}
env = {'user': 'admin'}
cli = {'color': 'red'}
config = ChainMap(cli, env, defaults)
config['user']   # 'admin' — falls through cli to env
config['color']  # 'red'   — found in first map
# Mutations only hit the first map:
config['new_key'] = 'val'  # goes into cli, not defaults
```

## Generator Pipeline Pattern

```python
# The canonical Hettinger pipeline: each stage is a generator,
# no intermediate lists, memory stays flat regardless of data size.
def read_lines(path):
    with open(path) as f:
        yield from f

def strip_comments(lines):
    for line in lines:
        if not line.startswith('#'):
            yield line.strip()

def parse_records(lines):
    for line in lines:
        fields = line.split(',')
        if len(fields) == 3:
            yield {'name': fields[0], 'age': int(fields[1]), 'city': fields[2]}

# Compose:
records = parse_records(strip_comments(read_lines('data.csv')))
for rec in records:  # Processes one line at a time
    process(rec)
```

## Sorting Idioms

```python
from operator import itemgetter, attrgetter

# Multi-key sort: operator functions are faster than lambda
students = [('Alice', 85), ('Bob', 90), ('Charlie', 85)]

# For descending on one key: negate works for ints, not strings
sorted(students, key=lambda s: (-s[1], s[0]))

# For objects — attrgetter is ~30% faster than lambda for attribute access
sorted(users, key=attrgetter('last_name', 'first_name'))

# itemgetter returns tuple for multiple keys — works as sort key directly
sorted(rows, key=itemgetter(2, 0))  # sort by col 2, then col 0
```

## Cooperative super() Patterns

```python
# Root class stops the chain
class Root:
    def draw(self):
        pass  # Terminates super() chain — no call to super()

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)

class Shape(Root):
    def __init__(self, color='black', **kwargs):
        self.color = color
        super().__init__(**kwargs)  # Forward unknown kwargs

    def draw(self):
        print(f'Drawing in {self.color}')
        super().draw()

class Moveable(Root):
    def __init__(self, x=0, y=0, **kwargs):
        self.x, self.y = x, y
        super().__init__(**kwargs)

    def draw(self):
        print(f'At ({self.x}, {self.y})')
        super().draw()

# Composition via MRO — no code changes to Shape or Moveable:
class MoveableShape(Shape, Moveable):
    pass

# MRO: MoveableShape -> Shape -> Moveable -> Root -> object
ms = MoveableShape(color='red', x=5, y=10)
ms.draw()  # "Drawing in red" then "At (5, 10)"
```
