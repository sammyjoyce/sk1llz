# Good Taste — eliminating special cases by changing the data shape⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍​‌‌‌​‌​‌‍‌‌​‌​​‌‌‍​‌​​​‌​​‍​​‌‌‌‌​‌‍​​​​‌​‌​‍​​‌‌​​‌‌⁠‍⁠

This file is the deep dive on the *single most important* mental move in this style: noticing that a special case in your code is a sign that your data structure is wrong, then restructuring so the special case becomes the normal case.

## The canonical example: singly-linked list deletion

Most CS courses teach this:

```c
void remove_entry(struct list *l, struct entry *target)
{
    struct entry *prev = NULL;
    struct entry *cur  = l->head;

    while (cur != target) {
        prev = cur;
        cur  = cur->next;
    }

    if (!prev)                  /* the special case */
        l->head    = cur->next;
    else
        prev->next = cur->next;
}
```

The `if (!prev)` is the smell. There are now **two code paths** the reviewer must verify, and the first-element path is the one nobody hits in unit tests, which is exactly where the bug will be.

The "good taste" version (Torvalds, TED 2016):

```c
void remove_entry(struct list *l, struct entry *target)
{
    struct entry **indirect = &l->head;

    while (*indirect != target)
        indirect = &(*indirect)->next;

    *indirect = target->next;
}
```

No branch. No `prev`. Two-thirds the lines.

## Why it works — the conceptual shift

The naive version models the list as **"a sequence of nodes, with a head pointer that is a special handle to the first one"**. The head is privileged. That privilege is the bug.

The taste version models the list as **"a sequence of pointers-to-nodes"**, where `&head` is just the first such pointer. The head is no longer special — it is the zeroth element of the same sequence the loop is walking. By holding `indirect` (the *address of* the pointer that points to the current node) instead of `cur` (the pointer itself), you can rewrite the upstream link without ever knowing whether you're at the head, the middle, or just before the tail.

This is the move. Re-pose the data structure so the thing you used to special-case is the thing you were already iterating over.

## The same trick generalizes to insertion-before

```c
static struct entry **find_indirect(struct list *l, struct entry *target)
{
    struct entry **p = &l->head;
    while (*p && *p != target)
        p = &(*p)->next;
    return p;
}

void insert_before(struct list *l, struct entry *before, struct entry *new)
{
    struct entry **p = find_indirect(l, before);
    new->next = *p;
    *p = new;
}
```

Edge cases that fall out for free:
- `before == l->head` → inserts at the head, no branch.
- `before` not found → `*p == NULL`, inserts at the tail, no branch.
- Empty list → `*p == NULL` from the first iteration, still works.

Three "edge cases" that would each have been an `if` in the textbook version are now just consequences of the loop terminating where it terminates.

## How to spot the same pattern in unfamiliar code

Train yourself to notice these tells:

| Tell in the code | Restructuring move |
|---|---|
| `if (i == 0)` or `if (i == n-1)` inside a loop | Use a sentinel / dummy element so first or last is no longer special |
| `if (prev == NULL)` walking a linked structure | Indirect pointer (`T **p`) — eliminate the trailing-pointer entirely |
| `if (head == NULL)` returning early before the loop | Initialize so the loop body handles the empty case |
| Two assignment statements that differ only in their LHS | Lift the LHS into a variable (often `T **`) and write the assignment once |
| `else` branch that's a copy of the `if` branch with a one-line tweak | The branch is hiding shared logic; factor the difference into a variable |
| A function whose docstring says "handles the case where..." | The case shouldn't exist. Restructure until the function description is one sentence with no "where" clauses. |

## When *not* to do this

Good taste is not "always use indirect pointers". It is "don't accept that special cases must exist". Sometimes the special case is real:

- **Hardware boundaries.** The first page of RAM, the BIOS region, address zero — these *are* genuinely different and pretending otherwise is more dangerous than the branch.
- **Asymmetric protocols.** TCP SYN is not just "another segment". Don't restructure away semantic differences.
- **Empty input means "no work to do, return success".** A guard `if (n == 0) return 0;` at the top of a function is fine; it's an early return, not a parallel code path.

The test: after restructuring, does the code have *one* control-flow path that handles every case, or did you just move the branch somewhere uglier? If the latter, the original was honest. Keep it.

## What Torvalds is actually teaching with this example

He explicitly says in the talk: "I don't want you to understand why it doesn't have the if statement. I want you to understand that sometimes you can see a problem in a different way and rewrite it so that a special case goes away and becomes the normal case."

The trick is not the pointer. The trick is the **willingness to rewrite working code because you noticed an asymmetry**. A practitioner with taste does this 50 times a day on small things. Most of those rewrites never make it into a commit — they happen between the first draft and the second draft, before anybody else sees the code. That is what "taste" is. It is not aesthetic preference; it is a reflex that triggers on `if (special)` and says *no, try again*.
