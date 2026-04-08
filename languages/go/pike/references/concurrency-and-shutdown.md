# Concurrency And Shutdown

Load this before adding goroutines, channels, or pipeline stages.

## Ownership rules

- Only the sending side should close a channel. If the design makes receivers close it, the ownership model is already muddy.
- Use channels when the message transfers work or ownership. Use a mutex or atomic when the problem is just shared state.
- In a multi-input `select`, once one input is known to be closed, disable that case locally or exit the loop. Closed receives stay permanently ready.
- Goroutines are cheap because user stacks start small, about 2K, and grow or shrink dynamically. Treat that as permission for many goroutines, not as permission for unbounded retained state or unbounded input.

## Pipeline rules

- Each stage closes its outbound channel when all sends are done.
- Each stage keeps receiving until inbound closes or until cancellation unblocks senders.
- Early return must have a broadcast stop path. Closing `done` works because an unknown number of blocked senders can all observe it immediately.
- A buffer is a proof obligation, not a tuning knob. Use it only when you can prove the total outstanding sends; otherwise cancellation must be explicit.

## Shutdown traps

- `for { select { default: } }` spins. If you need periodic work, block and drive it with a ticker or a timer.
- `defer` inside a worker loop does not run per iteration. In infinite or long-lived workers, put cleanup on the branch that returns.
- If a goroutine has no clear stop condition before you start it, you do not understand the design yet.
- A "successful" concurrency demo with 100,000 goroutines proves the runtime model, not your production budget. The real budget is what each goroutine captures and how cancellation propagates.

## Sanity checks

- Write down who owns each mutable value.
- Write down who closes each channel.
- Write down how the last goroutine exits on success, on cancellation, and on partial failure.
