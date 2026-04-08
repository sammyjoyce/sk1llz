# Isolation anomalies: the full catalog and what your database actually gives you⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌​​​​‍‌​‌​​‌​​‍​​‌‌‌​​‌‍‌‌‌​‌‌‌​‍​​​​‌​‌​‍​‌‌‌​​‌‌⁠‍⁠

Source: Berenson, Bernstein, Gray, Melton, O'Neil, O'Neil. *A Critique of ANSI SQL Isolation Levels*. SIGMOD 1995.

The ANSI SQL-92 standard defined isolation in terms of three phenomena (dirty read, fuzzy read, phantom) and four levels. Gray and coauthors proved the standard is **incomplete and ambiguous**. The complete taxonomy has eight anomalies, six levels, and one crucial ordering surprise (Snapshot Isolation is incomparable with Repeatable Read).

## The eight anomalies

| Code  | Name            | The story |
|-------|-----------------|-----------|
| P0    | Dirty Write     | T1 writes x; T2 writes x before T1 commits. Rolling back either one now loses the other's write. The ANSI standard **did not prohibit this** at any level below Serializable — a defect. |
| P1    | Dirty Read      | T1 reads a value T2 wrote but hasn't committed. T2 may still abort. |
| P4    | Lost Update     | T1 reads x; T2 reads x, writes x+1, commits; T1 writes x+2 based on its stale read. T2's update is lost. |
| P4C   | Cursor Lost Update | Same but with an open cursor; fixable by holding a cursor-local lock. |
| P2    | Fuzzy Read (non-repeatable read) | T1 reads x twice and gets different values because T2 wrote x in between. |
| P3    | Phantom         | T1 evaluates `WHERE P` twice; T2 inserts/deletes a row matching P in between. |
| A5A   | Read Skew       | T1 reads x (sees old), T2 updates x and y atomically, T1 reads y (sees new). The pair `(x,y)` violates a cross-row invariant. |
| **A5B** | **Write Skew**  | T1 and T2 both read `(x,y)`, each decides the invariant holds, then T1 writes y and T2 writes x. Neither transaction saw the other's write, and both committed — but the resulting state violates the invariant. |

**Write skew is the anomaly that bites production systems the most often** and is the one most engineers have never heard of.

## The canonical write skew example

A hospital rule: *at least one doctor must be on-call at any time.*

    Initial: Alice on-call = true, Bob on-call = true
    
    T1 (Alice wants to sign off):
      SELECT COUNT(*) FROM doctors WHERE on_call = true   -- returns 2
      IF count >= 2 THEN
        UPDATE doctors SET on_call = false WHERE name = 'Alice'
      COMMIT
    
    T2 (Bob wants to sign off), running concurrently:
      SELECT COUNT(*) FROM doctors WHERE on_call = true   -- returns 2 (snapshot)
      IF count >= 2 THEN
        UPDATE doctors SET on_call = false WHERE name = 'Bob'
      COMMIT
    
    Result: zero doctors on call. Invariant violated. Nobody fails, nobody retries.

**Snapshot Isolation permits this.** PostgreSQL's `REPEATABLE READ` (which is actually SI) permits this. Oracle permits this. MySQL InnoDB permits this.

Mitigations (any one works):
1. `SELECT ... FOR UPDATE` on the rows you read (converts the read to a write lock).
2. `SERIALIZABLE` in PostgreSQL (SSI detects and aborts one of the conflicting transactions).
3. Write a sentinel row that every transaction in the conflict set also writes, forcing serialization.
4. Restructure the invariant into a single-row check (e.g., a counter column that can be conditionally updated atomically).

## What your database actually provides

| Database | Default level | What "REPEATABLE READ" means | What "SERIALIZABLE" means |
|----------|---------------|------------------------------|---------------------------|
| PostgreSQL | Read Committed | Snapshot Isolation (allows write skew!) | SSI — detects conflicts at commit, raises `serialization_failure`; your app **must** retry |
| Oracle   | Read Committed | *Does not exist* — maps to Serializable | SI, **not** true serializable; write skew possible |
| MySQL InnoDB | Repeatable Read | SI + gap locks; phantom behavior depends on exact query shape | 2PL-based; high contention, blocks on reads |
| SQL Server | Read Committed | 2PL-based repeatable read | 2PL-based serializable; very high lock contention |
| CockroachDB | Serializable | (RR not supported) | SSI |
| Spanner | Serializable | (RR not supported) | External consistency via TrueTime |

**The key takeaway:** "SERIALIZABLE" is not a portable setting. In Postgres it aborts transactions at commit. In SQL Server it holds locks for the whole transaction. In Oracle it silently lets write skew through. Portable code must either (a) pick the strongest available level and handle aborts everywhere, or (b) use application-level locking to bypass the isolation level entirely.

## The isolation-level lattice (partial order)

    Read Uncommitted ≤ Read Committed ≤ Cursor Stability ≤ Repeatable Read ≤ Serializable
                            │
                            └─► Snapshot Isolation  ────►  Serializable

- **Snapshot Isolation is strictly stronger than Read Committed** (prevents P1, P4, A5A).
- **Snapshot Isolation is INCOMPARABLE with Repeatable Read.** SI prevents phantoms that RR allows; RR prevents write skew that SI allows. Neither dominates.
- **Only SERIALIZABLE (or SSI) prevents all eight anomalies.**

## Decision tree: choosing the level

```
Does any transaction do a read-modify-write on data that might be written by another transaction?
  └─ No:  Read Committed is fine.
  └─ Yes: Does the RMW depend on a single row's current value?
        └─ Yes: Read Committed + SELECT FOR UPDATE, OR use an atomic UPDATE ... WHERE value = X.
        └─ No (cross-row invariant): Does your database support real SERIALIZABLE (SSI or 2PL)?
              └─ Yes: Use SERIALIZABLE; handle `serialization_failure` with a retry loop.
              └─ No: Use advisory locks or a sentinel row to force serialization of the conflict set.
```

## The RMW audit checklist

For every read-modify-write in your codebase, ask:

1. **What rows does the read observe?** (All rows you read, not just the one you write.)
2. **What is the invariant you're enforcing?** Write it down in one sentence.
3. **If another transaction concurrently reads the same rows and writes different ones, can the invariant be violated?** If yes, you have a potential write skew.
4. **What protects you?** A row-level write lock, a `FOR UPDATE`, an advisory lock, a sentinel, or true serializable isolation. If the answer is "nothing" or "the isolation level" and the level is SI, you have a bug.

## Common write-skew bugs that ship to production

- **Unique constraint by SELECT-then-INSERT.** "Does this username exist? No? Insert it." Two concurrent transactions both see "no" and both insert. Fix: actual UNIQUE constraint, or `INSERT ... ON CONFLICT`.
- **Bank account "must stay ≥ 0".** Two concurrent withdrawals each see a balance of $100, each debits $60. Final balance: $-20. Fix: atomic `UPDATE ... WHERE balance >= 60` (row lock) or SSI.
- **Inventory decrement.** Two concurrent orders see stock=1, both decrement, oversell by one. Fix: same as above.
- **Rate limiter.** "Has this user hit 100 requests in the last minute? No? Let them through." Two requests race the check. Fix: atomic counter increment with a conditional UPDATE or a Lua script in Redis.
- **On-call schedule.** The doctor example above. Fix: SSI or a sentinel row holding "current on-call count" that every mutator updates.

## Snapshot isolation's redeeming features

SI is not bad — it's the right default for most workloads. It has genuine advantages:
- **Readers never block writers and vice versa.** This is the whole point.
- **Lost updates are prevented by first-committer-wins.** If two transactions write the same row, one aborts.
- **Read skew (A5A) is prevented.** The snapshot is a consistent point-in-time view.
- **Phantoms within a single snapshot are impossible.** The predicate evaluates against a fixed view.

Write skew is the *only* anomaly SI introduces beyond Serializable. Audit your code for it specifically; do not re-audit everything.

## Testing for isolation bugs

- **Jepsen-style testing** (Aphyr's tool): run many concurrent clients, record observed histories, check against an invariant. Finds write-skew, lost-update, and ordering bugs that unit tests miss.
- **Property-based tests** with injected interleavings: use a library that controls transaction scheduling (e.g., Elle for Clojure, Hermitage for comparing DBs).
- **Hermitage** (github.com/ept/hermitage) is a set of test cases that show which anomalies each database allows at each level. Run your own — vendor documentation is often wrong.
