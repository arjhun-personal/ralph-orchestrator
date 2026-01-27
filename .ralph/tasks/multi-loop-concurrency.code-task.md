# Multi-Loop Concurrency Support via Git Worktrees

## Summary

Enable multiple Ralph orchestration loops to run simultaneously using git worktrees for automatic filesystem isolation. Parallelism is transparent—second+ invocations auto-detect a running loop and spawn into worktrees. Completed loops auto-merge via a standard `ralph run` with a merge preset.

## Problem

Ralph currently assumes single-instance execution per workspace. Multiple concurrent loops cause:

1. **Marker file collision** — `.ralph/current-events` overwritten by each loop
2. **JSONL corruption** — Concurrent appends to events files interleave lines
3. **TOCTOU races** — Read-modify-write patterns on `.agent/` files lose updates
4. **State desync** — In-memory `EventReader.position` not shared between processes

## Industry Comparison

| Tool | Author | Model | Scale | Philosophy |
|------|--------|-------|-------|------------|
| **Ralph** | — | Single orchestrator | 1-5 loops | "Fresh context is reliability" |
| **Gas Town** | Steve Yegge | Mayor → Polecats | 20-30 agents | Scale with git-backed beads |
| **Subtask** | zippoxer | Lead → Workers | 1-20 tasks | Git-native, event-sourced |
| **ccswarm** | nwiizo | ProactiveMaster | Enterprise | Specialized roles, autonomous |

**Ralph aligns with Subtask's philosophy:**
- File-first, git-native (matches "disk is state, git is memory")
- Event-sourced history for crash recovery
- Workflow stages as natural backpressure gates
- Simple over complex (skip Gas Town's 20-agent scale, ccswarm's role system)

> "The problem isn't cognitive—it's codebase overlap. Two agents touching the same files create the same merge headaches as two developers working on top of each other." — Simon Willison

**Git worktrees** are the industry consensus: each agent gets a separate working directory with complete filesystem isolation, sharing only `.git` history. Conflicts are resolved at merge time, not during execution.

## Core Behavior

```
First ralph run   → Runs in main worktree, acquires lock
Second ralph run  → Detects lock, auto-spawns into worktree
Third ralph run   → Same, another worktree
...
Loop completes    → Queues for merge, spawns merge-ralph
merge-ralph       → Standard `ralph run` with merge-loop preset
```

**No explicit `--parallel` flag needed.** Parallelism is automatic and transparent.

## Acceptance Criteria

- [ ] First `ralph run` acquires loop lock and runs in-place (no worktree overhead)
- [ ] Second+ `ralph run` auto-detects lock and spawns into git worktree
- [ ] Each worktree loop has fully isolated `.ralph/` and `.agent/` directories
- [ ] Shared memories (`.agent/memories.md`) accessible across loops with locking
- [ ] Completed loops auto-spawn `ralph run --preset merge-loop` to merge back
- [ ] merge-ralph uses user's configured backend (claude, kiro, etc.)
- [ ] merge-ralph resolves conflicts via AI, runs tests to verify
- [ ] Merge queue processes loops serially (FIFO order)
- [ ] Unresolvable conflicts marked `needs-review` with user notification
- [ ] `ralph loops` shows all active/queued/merged loops
- [ ] `ralph loops retry <id>` re-runs merge-ralph for failed merge
- [ ] `ralph loops discard <id>` abandons loop's changes
- [ ] `ralph loops stop <id>` terminates a running loop
- [ ] `--no-auto-merge` skips merge-ralph, leaves worktree for manual handling
- [ ] `--exclusive` blocks until primary loop slot available (no worktree)
- [ ] Built-in `merge-loop` preset included in distribution
- [ ] Event-sourced `history.jsonl` enables crash recovery and debugging
- [ ] Existing single-loop behavior unchanged when no concurrency

## Design

### Loop Lock Detection

```rust
// .ralph/loop.lock - held by active primary loop
pub struct LoopLock {
    pid: u32,
    started: DateTime<Utc>,
    prompt: String,
}

impl LoopLock {
    /// Try to acquire primary loop slot
    /// Returns Ok(guard) if acquired, Err(existing) if another loop holds it
    pub fn try_acquire() -> Result<LockGuard, LoopLock> {
        // Use flock() - automatically released on process exit
    }
}
```

### Startup Flow

```rust
fn run(args: RunArgs) -> Result<()> {
    match LoopLock::try_acquire() {
        Ok(guard) => {
            // We're the primary loop - run normally in place
            run_loop_in_place(args, guard)
        }
        Err(existing) => {
            // Another loop is running - spawn into worktree
            let loop_id = generate_loop_id(); // ralph-20250124-a3f2
            let worktree = create_worktree(&loop_id)?;
            run_loop_in_worktree(args, worktree, loop_id)
        }
    }
}
```

### Git Worktree-Based Isolation

**Worktree directory is configurable** (default: `.worktrees`). On first use, Ralph auto-appends to `.gitignore` if the pattern isn't found.

```yaml
# ralph.yml
parallel:
  worktree_dir: .worktrees  # default, can be absolute or relative
```

```
repo/
├── .git/                          # Shared git directory
├── .gitignore                     # Auto-appended: .worktrees/
├── .ralph/
│   ├── loop.lock                  # Primary loop lock (flock)
│   ├── merge-queue.jsonl          # Pending merges
│   └── loops/
│       └── registry.json          # Active loop metadata
├── src/                           # Main working tree
└── ...

.worktrees/                        # Parallel worktrees (configurable)
├── ralph-20250124-a3f2/           # Loop 1's isolated workspace
│   ├── .ralph/
│   │   ├── events.jsonl
│   │   ├── history.jsonl
│   │   └── diagnostics/
│   ├── .agent/
│   │   ├── scratchpad.md
│   │   ├── tasks.jsonl
│   │   └── memories.md → ../../repo/.agent/memories.md (symlink)
│   └── src/                       # Full repo checkout
└── ralph-20250124-b7e9/           # Loop 2's isolated workspace
    └── ...
```

### Why Worktrees Over Directory Namespacing

| Aspect | Directory Namespacing | Git Worktrees |
|--------|----------------------|---------------|
| File isolation | Partial (shared codebase) | Complete |
| Agent confusion | Can read other loop's changes | Impossible |
| Merge conflicts | During execution | At merge time |
| Industry adoption | None | Cursor, Subtask, ccswarm, Uzi |
| Code changes | Interleaved | Isolated branches |

### Loop Lifecycle

```
1. ralph run -p "implement auth"
   ├── Try acquire loop.lock
   ├── LOCKED? → Create worktree, run there
   │   ├── Generate loop ID: ralph-20250124-a3f2
   │   ├── Create branch: ralph/loop/a3f2
   │   ├── Create worktree: .worktrees/ralph-20250124-a3f2
   │   ├── Symlink shared memories
   │   └── Register in .ralph/loops/registry.json
   └── UNLOCKED? → Run in place (primary loop)

2. Loop executes (in-place or worktree)
   ├── All file changes isolated (worktree) or direct (primary)
   ├── Events/tasks/scratchpad isolated
   └── Memories synced via symlink + lock

3. Loop completes → Auto-merge (unless --no-auto-merge)
   ├── Queue in .ralph/merge-queue.jsonl
   ├── Spawn: ralph run --preset merge-loop -p "Merge loop a3f2..."
   ├── merge-ralph resolves conflicts via AI
   ├── merge-ralph runs tests to verify
   ├── On success: remove worktree, delete branch
   └── On failure: mark needs-review, notify user

4. Or: ralph loops stop a3f2
   ├── Send SIGTERM to loop process
   ├── Clean up worktree (optional: --keep)
   └── Deregister from registry
```

### Shared Memories with Locking

Memories are the only shared state between loops (intentional for cross-loop learning):

```rust
// Symlink in each worktree points to main repo's memories
.agent/memories.md → ../../../repo/.agent/memories.md

// File locking via fs2/fd-lock for concurrent access
impl MemoryStore {
    fn append(&self, memory: &Memory) -> io::Result<()> {
        let lock = FileLock::exclusive(&self.lock_path)?;
        // ... read-modify-write ...
        drop(lock);
    }

    fn load(&self) -> io::Result<Vec<Memory>> {
        let lock = FileLock::shared(&self.lock_path)?;
        // ... read ...
        drop(lock);
    }
}
```

### Auto-Merge via merge-ralph

When a worktree loop completes, Ralph spawns a standard `ralph run` with the built-in `merge-loop` preset:

```rust
fn on_loop_complete(loop_id: &str, config: &Config) -> Result<()> {
    if config.auto_merge {
        MergeQueue::enqueue(loop_id)?;
        spawn_merge_ralph(loop_id)?;
    }
}

fn spawn_merge_ralph(loop_id: &str) -> Result<()> {
    // Just a normal ralph run with a preset
    Command::new("ralph")
        .args([
            "run",
            "--preset", "merge-loop",
            "-p", &format!("Merge loop {} from branch ralph/loop/{}", loop_id, loop_id),
        ])
        .env("RALPH_MERGE_LOOP_ID", loop_id)
        .spawn()?;

    Ok(())
}
```

### Built-in merge-loop Preset

```yaml
# Included in ralph binary, loaded from embedded assets
name: merge-loop
description: Merges completed parallel loop back to main branch

prompt: |
  Merge the completed Ralph loop back to the main branch.

  ## Context
  - Loop ID: {from RALPH_MERGE_LOOP_ID env var}
  - Loop branch: ralph/loop/{loop_id}
  - Worktree: .worktrees/ralph-{loop_id}

  ## Steps
  1. `git diff main...ralph/loop/{loop_id}` — Review changes
  2. `git merge ralph/loop/{loop_id}` — Attempt merge
  3. If conflicts:
     - Understand intent of both sides
     - Resolve preserving functionality from both
     - `cargo test` to verify
  4. On success:
     - `git worktree remove .worktrees/ralph-{loop_id}`
     - `git branch -d ralph/loop/{loop_id}`
     - Update merge queue status
  5. On unresolvable conflict:
     - Mark as needs-review in merge queue
     - Exit with explanation

max_iterations: 10
```

### Merge Queue

```jsonl
// .ralph/merge-queue.jsonl (append-only log)
{"ts": "...", "loop_id": "a3f2", "event": "queued", "prompt": "implement auth"}
{"ts": "...", "loop_id": "a3f2", "event": "merging", "pid": 12345}
{"ts": "...", "loop_id": "a3f2", "event": "merged", "commit": "abc123"}

// Or for failures:
{"ts": "...", "loop_id": "b7e9", "event": "needs_review", "reason": "Conflicting changes to src/auth.rs"}
```

**Queue states:**
```
queued → merging → merged
            ↓
      needs_review
```

**Serial processing:** Only one merge-ralph runs at a time. Each merge-ralph acquires a merge lock before starting. This prevents merge conflicts between merge operations themselves.

### Two Separate Locks

```
.ralph/loop.lock   — Primary user loop slot (held by first ralph run)
.ralph/merge.lock  — Merge operations (held by merge-ralph)
```

**merge-ralph does NOT acquire the loop lock.** This means:
- Users can start new loops while merges are happening
- New user loop → detects loop.lock held → spawns into worktree
- merge-ralph runs concurrently, doing housekeeping in background

```
User loop A (primary)     ──────────────────────────→
User loop B (worktree)    ─────────────→ completes, queues merge
User loop C (worktree)         ─────────────────────→
merge-ralph (B)                         ──────→ merges B
merge-ralph (C)                                      ───→ (queued, waiting)
```

**What if user starts a loop while merge-ralph is active?**
- merge-ralph holds merge.lock only, not loop.lock
- New `ralph run` checks loop.lock → might be free or held by user loop A
- If free: new loop becomes primary
- If held: new loop spawns into worktree
- Either way, merge-ralph continues uninterrupted

### Registry Format

```json
// .ralph/loops/registry.json
{
  "loops": {
    "a3f2": {
      "id": "ralph-20250124-143052-a3f2",
      "branch": "ralph/loop/a3f2",
      "worktree": "/abs/path/.worktrees/ralph-20250124-143052-a3f2",
      "pid": 12345,
      "started": "2025-01-24T14:30:52Z",
      "prompt": "implement auth",
      "state": "running"
    },
    "b7e9": {
      "id": "ralph-20250124-143108-b7e9",
      "branch": "ralph/loop/b7e9",
      "worktree": null,
      "pid": null,
      "started": "2025-01-24T14:31:08Z",
      "completed": "2025-01-24T14:45:00Z",
      "prompt": "write tests",
      "state": "merged",
      "merge_commit": "abc123"
    }
  }
}
```

### Loop States

With auto-merge, loop states are simplified:

```
running → merging → merged
             ↓
       needs_review
```

**State transitions:**
- `running → merging`: Loop completes, merge-ralph spawned
- `merging → merged`: merge-ralph succeeds, worktree cleaned up
- `merging → needs_review`: merge-ralph can't resolve conflicts

**needs_review resolution:**
```bash
ralph loops attach d4e5    # Enter worktree, fix conflicts manually
ralph loops retry d4e5     # Re-run merge-ralph
# OR
ralph loops discard d4e5   # Abandon loop's changes
```

### Event-Sourced History (from Subtask)

Each loop maintains an append-only event log for crash recovery and debugging:

```
.worktrees/ralph-{id}/.ralph/history.jsonl
```

**Event format:**
```json
{"ts": "2025-01-23T14:30:52Z", "type": "loop.started", "data": {"prompt": "implement auth"}}
{"ts": "2025-01-23T14:30:53Z", "type": "iteration.started", "data": {"iteration": 1}}
{"ts": "2025-01-23T14:31:15Z", "type": "event.published", "data": {"topic": "build.task", "payload": "..."}}
{"ts": "2025-01-23T14:32:00Z", "type": "iteration.completed", "data": {"iteration": 1, "success": true}}
{"ts": "2025-01-23T14:35:00Z", "type": "loop.completed", "data": {"reason": "completion_promise"}}
{"ts": "2025-01-23T14:35:01Z", "type": "stage.changed", "data": {"from": "doing", "to": "review"}}
```

**Benefits:**
- **Crash recovery**: Resume from last known state after crash
- **Debugging**: Replay loop execution to understand failures
- **Auditing**: Complete trace of what happened and when
- **Source of truth**: Registry is derived from history, not vice versa

**Recovery flow:**
```bash
ralph loops resume a3f2  # Reads history.jsonl, resumes from last iteration
```

### CLI Interface

```bash
# Normal usage - parallelism is automatic
ralph run -p "implement feature"    # First loop, runs in place
ralph run -p "write tests"          # Detects lock, spawns worktree
ralph run -p "update docs"          # Another worktree

# Explicit control
ralph run --exclusive -p "..."      # Block until primary slot available
ralph run --no-auto-merge -p "..."  # Skip merge-ralph, leave worktree

# List all loops
ralph loops
# ID      STATUS        LOCATION              PROMPT
# main    running       (in-place)            implement feature
# a3f2    running       .worktrees/a3f2       write tests
# b7e9    merging       -                     update docs (merge-ralph active)
# c1d3    merged        -                     fix bug
# d4e5    needs-review  .worktrees/d4e5       refactor auth

# View loop output
ralph loops logs a3f2
ralph loops logs a3f2 --follow

# View event history
ralph loops history a3f2            # Show history.jsonl events
ralph loops history a3f2 --json     # Raw JSONL output

# Merge management
ralph loops retry d4e5              # Re-run merge-ralph after manual fix
ralph loops discard d4e5            # Abandon loop's changes, clean up worktree

# Stop running loop
ralph loops stop a3f2               # Graceful shutdown
ralph loops stop a3f2 --force       # SIGKILL

# Clean up stale loops (crashed processes)
ralph loops prune

# Attach to loop (for debugging)
ralph loops attach a3f2             # Opens shell in worktree

# Show diff of loop's changes
ralph loops diff a3f2               # git diff from merge-base
```

## Implementation Steps

### Step 1: Add loop lock mechanism

Create `crates/ralph-core/src/loop_lock.rs`:
- `LoopLock` struct using `flock()` on `.ralph/loop.lock`
- `try_acquire()` — Non-blocking attempt, returns `Ok(guard)` or `Err(existing)`
- `LockGuard` releases lock on drop (or process exit)
- Read existing lock metadata (pid, prompt, started) when locked

### Step 2: Add git worktree operations

Create `crates/ralph-core/src/worktree.rs`:
- `create_worktree(loop_id, config)` — Creates branch + worktree
- `remove_worktree(path)` — Cleans up worktree
- `list_worktrees()` — Lists existing worktrees
- `ensure_gitignore(worktree_dir)` — Appends to `.gitignore` if pattern missing
- Use `git2` crate or shell out to `git` command

### Step 3: Add loop registry

Create `crates/ralph-core/src/loop_registry.rs`:
- `LoopRegistry` struct with JSON persistence
- `register_loop()`, `deregister_loop()`, `get_loop()`, `list_loops()`
- PID-based stale loop detection
- File locking for concurrent registry access

### Step 4: Add merge queue

Create `crates/ralph-core/src/merge_queue.rs`:
- `MergeQueue` struct wrapping `.ralph/merge-queue.jsonl`
- `enqueue(loop_id)` — Add loop to queue
- `mark_merging(loop_id, pid)` — Record merge-ralph started
- `mark_merged(loop_id, commit)` — Record successful merge
- `mark_needs_review(loop_id, reason)` — Record failure
- `next_pending()` — Get next loop ready for merge (FIFO)

### Step 5: Add file locking for shared memories

Update `crates/ralph-core/src/memory_store.rs`:
- Add `fs2` or `fd-lock` dependency
- Wrap read operations with shared lock
- Wrap write operations with exclusive lock
- Symlink detection for worktree mode

### Step 6: Create LoopContext for path resolution

Create `crates/ralph-core/src/loop_context.rs`:
- `LoopContext` struct with worktree paths
- `LoopContext::primary()` — For in-place primary loop
- `LoopContext::worktree(loop_id, path)` — For worktree-based loops
- Path resolution for events, tasks, scratchpad, memories

### Step 7: Plumb LoopContext through core

Update path resolution in:
- `EventLoop` — Use context for event reader/logger paths
- `EventReader` — Accept base path from context
- `EventLogger` — Accept base path from context
- `TaskStore` — Accept base path from context
- `SummaryWriter` — Accept base path from context

### Step 8: Update CLI startup flow

Update `crates/ralph-cli/src/main.rs`:
```rust
fn run(args: RunArgs) -> Result<()> {
    match LoopLock::try_acquire() {
        Ok(guard) => run_primary_loop(args, guard),
        Err(_) if args.exclusive => {
            // --exclusive: wait for lock
            let guard = LoopLock::acquire_blocking()?;
            run_primary_loop(args, guard)
        }
        Err(_) => {
            // Auto-spawn into worktree
            let loop_id = generate_loop_id();
            let worktree = create_worktree(&loop_id)?;
            run_worktree_loop(args, worktree, loop_id)
        }
    }
}
```

### Step 9: Add loop completion handler

Create `crates/ralph-cli/src/loop_completion.rs`:
- Called when loop completes successfully
- If worktree loop and auto_merge enabled:
  - Enqueue in merge queue
  - Spawn `ralph run --preset merge-loop -p "Merge loop {id}..."`
- If `--no-auto-merge`: just log completion, leave worktree

### Step 10: Add built-in merge-loop preset

Add to `crates/ralph-core/src/presets/merge_loop.rs`:
- Embed preset YAML as const string
- Register in preset loader as built-in
- Preset prompts merge, conflict resolution, test verification, cleanup

### Step 11: Add `loops` subcommand

Add to `crates/ralph-cli/src/commands/loops.rs`:
- `ralph loops` — Table of all loops (active, merging, merged, needs-review)
- `ralph loops logs <id>` — Tail loop output
- `ralph loops history <id>` — Show history.jsonl
- `ralph loops retry <id>` — Re-run merge-ralph
- `ralph loops discard <id>` — Abandon and cleanup
- `ralph loops stop <id>` — Terminate loop process
- `ralph loops prune` — Clean stale loops
- `ralph loops attach <id>` — Open shell in worktree
- `ralph loops diff <id>` — git diff from merge-base

### Step 12: Add event-sourced history

Create `crates/ralph-core/src/loop_history.rs`:
- `LoopHistory` struct wrapping append-only JSONL file
- `append(event: HistoryEvent)` — Thread-safe append
- `read_all()` — Parse full history
- `last_iteration()` — Find last completed iteration for resume

Event types:
```rust
enum HistoryEventType {
    LoopStarted { prompt: String },
    IterationStarted { iteration: u32 },
    EventPublished { topic: String, payload: String },
    IterationCompleted { iteration: u32, success: bool },
    LoopCompleted { reason: String },
    LoopResumed { from_iteration: u32 },
    MergeQueued,
    MergeStarted { pid: u32 },
    MergeCompleted { commit: String },
    MergeFailed { reason: String },
}
```

### Step 13: Handle loop termination

Update `crates/ralph-cli/src/loop_runner.rs`:
- On clean exit: trigger completion handler
- On SIGTERM: clean up and deregister (no auto-merge)
- Write exit status to registry and history

## Configuration

```yaml
# ralph.yml
parallel:
  auto_merge: true          # default; set false to require manual merge
  worktree_dir: .worktrees  # default; can be absolute path
  merge_preset: merge-loop  # default; can specify custom preset
```

## Files to Modify/Create

**New files:**
- `crates/ralph-core/src/loop_lock.rs` — Primary loop lock (flock-based)
- `crates/ralph-core/src/worktree.rs` — Git worktree operations + gitignore handling
- `crates/ralph-core/src/loop_registry.rs` — Loop metadata persistence
- `crates/ralph-core/src/loop_context.rs` — Path resolution for isolated loops
- `crates/ralph-core/src/loop_history.rs` — Event-sourced history (append-only JSONL)
- `crates/ralph-core/src/merge_queue.rs` — Merge queue management
- `crates/ralph-core/src/presets/merge_loop.rs` — Built-in merge-loop preset
- `crates/ralph-cli/src/commands/loops.rs` — `ralph loops` subcommand
- `crates/ralph-cli/src/loop_completion.rs` — Completion handler, merge spawning

**Modified files:**
- `crates/ralph-core/src/lib.rs` — Export new modules
- `crates/ralph-core/src/memory_store.rs` — Add file locking
- `crates/ralph-core/src/event_loop/mod.rs` — Accept LoopContext, emit history events
- `crates/ralph-core/src/event_reader.rs` — Use context paths
- `crates/ralph-core/src/event_logger.rs` — Use context paths
- `crates/ralph-core/src/task_store.rs` — Accept base path
- `crates/ralph-core/src/summary_writer.rs` — Accept base path
- `crates/ralph-core/src/config.rs` — Add parallel config section
- `crates/ralph-cli/src/main.rs` — Add lock detection, --exclusive, --no-auto-merge
- `crates/ralph-cli/src/loop_runner.rs` — Worktree-aware execution
- `Cargo.toml` — Add git2, fs2 dependencies

## Testing

### Unit Tests
- LoopLock acquire/release, blocking vs non-blocking
- Worktree creation/removal
- Gitignore auto-append detection
- Registry CRUD operations
- MergeQueue enqueue/dequeue/status transitions
- File locking acquire/release for memories
- LoopContext path resolution (primary vs worktree)
- LoopHistory append/read/recovery

### Integration Tests
- First loop acquires lock, runs in place
- Second loop detects lock, spawns into worktree
- Two loops running concurrently complete without corruption
- Memory append from two loops doesn't lose data
- Loop completion triggers merge-ralph spawn
- merge-ralph merges and cleans up worktree
- merge-ralph handles conflicts, marks needs-review
- `--exclusive` blocks until lock available
- `--no-auto-merge` skips merge-ralph
- Stale loop detection and cleanup
- SIGTERM handling cleans up properly

### Manual Testing
- Run `ralph run` twice, verify second auto-spawns to worktree
- Verify `.worktrees` added to `.gitignore` on first use
- Verify merge-ralph runs after loop completes
- Verify user can start new loop while merge-ralph active
- Kill loop process, verify `ralph loops prune` cleans up
- Create conflicting changes, verify merge-ralph marks needs-review
- Run `ralph loops retry` after manual conflict resolution

## Dependencies

```toml
[dependencies]
git2 = "0.19"           # Git operations (or shell out)
fs2 = "0.4"             # Cross-platform file locking
# OR
fd-lock = "4.0"         # Alternative file locking
```

## References

- [Anthropic: Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
- [GitHub: Subtask](https://github.com/zippoxer/subtask) — Claude skill for worktree-based subagents (primary inspiration)
- [GitHub: Gas Town](https://github.com/steveyegge/gastown) — Steve Yegge's 20-30 agent orchestrator
- [GitHub: ccswarm](https://github.com/nwiizo/ccswarm) — Multi-agent orchestration with worktrees
- [Simon Willison: Parallel Coding Agents](https://simonwillison.net/2025/Oct/5/parallel-coding-agents/)
- [Git Worktrees for AI Agents](https://nx.dev/blog/git-worktrees-ai-agents)
- [GasTown and the Two Kinds of Multi-Agent](https://paddo.dev/blog/gastown-two-kinds-of-multi-agent/) — Architecture comparison

## Future Enhancements

- **Loop templates** — Pre-configured loop types (refactor, test, docs)
- **Loop dependencies** — Loop B waits for Loop A to complete
- **Remote worktrees** — SSH to remote machine for loop execution
- **Dashboard** — Web UI showing all active loops (integrate with ralph-web)
- **Custom merge presets** — User-defined merge strategies
- **Merge notifications** — Slack/webhook on merge complete or needs-review

## Explicitly Out of Scope

Per Ralph's "thin orchestrator" philosophy, we're NOT implementing:

- **Gas Town scale (20-30 agents)** — Optimized for 1-5 parallel loops
- **Specialized agent roles** (ccswarm) — All loops use same Ralph orchestration
- **Autonomous task prediction** (ccswarm ProactiveMaster) — User controls dispatch
- **Complex coordination protocols** — File-based state, not message buses
- **$100/hour token costs** — Parallel loops should be cost-conscious
- **Manual merge workflow** — Auto-merge is default; use `--no-auto-merge` for manual control
