# Inject Only Relevant Hat Instructions

## Problem Statement

**Issue 1: Context Bloat**
Currently, ALL hat instructions are included in Ralph's prompt regardless of which events are pending. With 10+ hats, this wastes massive token budget on irrelevant context.

```rust
// Current behavior in hatless_ralph.rs:239-249
for hat in &topology.hats {
    if !hat.instructions.trim().is_empty() {
        section.push_str(&format!("### {} Instructions\n\n", hat.name));
        section.push_str(&hat.instructions);  // ❌ ALL instructions included
    }
}
```

**Issue 2: Iteration Log Clarity**
The iteration separator always shows "🎭 ralph" because `next_hat()` returns "ralph" in multi-hat mode. Users can't see which hat role is active.

```
═══════════════════════════════════════════════════════════════════════════════
 ITERATION 2 │ 🎭 ralph │ 1m 5s elapsed │ 2/50
═══════════════════════════════════════════════════════════════════════════════
```

Should show:
```
═══════════════════════════════════════════════════════════════════════════════
 ITERATION 2 │ 🔒 security_reviewer │ 1m 5s elapsed │ 2/50
═══════════════════════════════════════════════════════════════════════════════
```

## Solution Design

### Approach: Determine Active Hat from Pending Events

```rust
// In EventLoop::build_prompt()
pub fn build_prompt(&mut self, hat_id: &HatId) -> Option<String> {
    if hat_id.as_str() == "ralph" {
        // 1. Collect all pending events
        let all_events = self.collect_all_events();

        // 2. Determine which hats are triggered by these events
        let active_hats = self.determine_active_hats(&all_events);

        // 3. Build prompt with ONLY active hat instructions
        let events_context = self.format_events(&all_events);
        return Some(self.ralph.build_prompt_with_active_hats(
            &events_context,
            &active_hats
        ));
    }
    // ...
}
```

### API Changes

#### 1. EventLoop gains methods to determine active hats

```rust
impl EventLoop {
    /// Collects all pending events from all hats.
    fn collect_all_events(&mut self) -> Vec<Event> {
        let all_hat_ids: Vec<HatId> = self.bus.hat_ids().cloned().collect();
        let mut all_events = Vec::new();
        for id in all_hat_ids {
            all_events.extend(self.bus.take_pending(&id));
        }
        all_events
    }

    /// Determines which hats should be active based on pending events.
    /// Returns list of Hat objects that are triggered by any pending event.
    fn determine_active_hats(&self, events: &[Event]) -> Vec<&Hat> {
        let mut active_hats = Vec::new();
        for event in events {
            if let Some(hat) = self.registry.get_for_topic(&event.topic) {
                if !active_hats.iter().any(|h: &&Hat| h.id == hat.id) {
                    active_hats.push(hat);
                }
            }
        }
        active_hats
    }

    /// Returns the primary active hat ID for display purposes.
    /// Returns the first active hat, or "ralph" if no specific hat is active.
    pub fn get_active_hat_id(&self) -> HatId {
        // Peek at pending events (don't consume them)
        for hat_id in self.bus.hat_ids() {
            if let Some(events) = self.bus.peek_pending(hat_id) {
                if !events.is_empty() {
                    // Return the hat ID that this event triggers
                    if let Some(event) = events.first() {
                        if let Some(active_hat) = self.registry.get_for_topic(&event.topic) {
                            return active_hat.id.clone();
                        }
                    }
                }
            }
        }
        HatId::new("ralph")
    }
}
```

#### 2. EventBus gains peek method

```rust
impl EventBus {
    /// Returns a reference to pending events for a hat without consuming them.
    pub fn peek_pending(&self, hat_id: &HatId) -> Option<&Vec<Event>> {
        self.pending.get(hat_id)
    }
}
```

#### 3. HatlessRalph modified to accept active hats

```rust
impl HatlessRalph {
    /// Builds Ralph's prompt with only the specified active hats' instructions.
    pub fn build_prompt_with_active_hats(
        &self,
        context: &str,
        active_hats: &[&Hat]
    ) -> String {
        let mut prompt = self.core_prompt();

        // Include pending events BEFORE workflow
        if !context.trim().is_empty() {
            prompt.push_str("## PENDING EVENTS\n\n");
            prompt.push_str(context);
            prompt.push_str("\n\n");
        }

        prompt.push_str(&self.workflow_section());

        if let Some(topology) = &self.hat_topology {
            // ✅ NEW: Pass active_hats to hats_section
            prompt.push_str(&self.hats_section(topology, active_hats));
        }

        prompt.push_str(&self.event_writing_section());
        prompt.push_str(&self.done_section());

        prompt
    }

    fn hats_section(&self, topology: &HatTopology, active_hats: &[&Hat]) -> String {
        let mut section = String::from("## HATS\n\nDelegate via events.\n\n");

        // Include starting_event instruction if configured
        if let Some(ref starting_event) = self.starting_event {
            section.push_str(&format!(
                "**After coordination, publish `{}` to start the workflow.**\n\n",
                starting_event
            ));
        }

        // Build hat table (keep full topology - it's compact and useful)
        section.push_str("| Hat | Triggers On | Publishes |\n");
        section.push_str("|-----|-------------|----------|\n");

        for hat in &topology.hats {
            let subscribes = hat.subscribes_to.join(", ");
            let publishes = hat.publishes.join(", ");
            section.push_str(&format!("| {} | {} | {} |\n", hat.name, subscribes, publishes));
        }

        section.push('\n');

        // ✅ CHANGED: Only include instructions for active hats
        if !active_hats.is_empty() {
            section.push_str("### Active Hat Instructions\n\n");
            for hat_ref in active_hats {
                if !hat_ref.instructions.trim().is_empty() {
                    section.push_str(&format!("#### {}\n\n", hat_ref.name));
                    section.push_str(&hat_ref.instructions);
                    if !hat_ref.instructions.ends_with('\n') {
                        section.push('\n');
                    }
                    section.push('\n');
                }
            }
        }

        section
    }
}
```

#### 4. main.rs uses active hat for display

```rust
// In main.rs event loop
let hat_id = match event_loop.next_hat() {
    Some(id) => id.clone(),
    None => {
        // ... fallback logic
    }
};

// ✅ NEW: Get the active hat for display
let display_hat = if hat_id.as_str() == "ralph" {
    event_loop.get_active_hat_id()
} else {
    hat_id.clone()
};

let iteration = event_loop.state().iteration + 1;

print_iteration_separator(
    iteration,
    display_hat.as_str(),  // ✅ Use display_hat instead of hat_id
    event_loop.state().elapsed(),
    config.event_loop.max_iterations,
    use_colors,
);
```

## Benefits

### Issue 1 Resolution: Massive Token Savings

**Before** (10 hats, avg 500 tokens per instruction):
```
Prompt size: ~5,000 tokens of hat instructions (all 10 hats)
```

**After** (only 1-2 active hats):
```
Prompt size: ~500-1,000 tokens of hat instructions (1-2 active hats)
Savings: ~4,000 tokens per iteration
```

### Issue 2 Resolution: Clear Iteration Display

**Before:**
```
ITERATION 2 │ 🎭 ralph │ 1m 5s elapsed │ 2/50
```

**After:**
```
ITERATION 2 │ 🔒 security_reviewer │ 1m 5s elapsed │ 2/50
```

Users can immediately see which hat role is active.

## Edge Cases

### Multiple Active Hats

If multiple events trigger different hats:
```
Event: review.security - {...}
Event: review.architecture - {...}
```

**Solution**: Include instructions for ALL active hats (security + architecture), but still better than including all 10 hats.

**Display**: Show the first active hat in the separator.

### No Active Hat (Orchestration)

When Ralph is coordinating but no specific hat is triggered:
```
Event: task.start - "Review PR #123"
```

**Solution**: No hat instructions included, just the topology table.

**Display**: Show "🎭 ralph" in the separator.

## Implementation Plan

1. **Add `peek_pending()` to EventBus** (ralph-proto)
2. **Add `get_active_hat_id()` and `determine_active_hats()` to EventLoop** (ralph-core)
3. **Modify `build_prompt()` to pass active hats** (ralph-core)
4. **Update `hats_section()` to filter by active hats** (ralph-core)
5. **Modify main.rs to use active hat for display** (ralph-cli)
6. **Update tests** to verify only active instructions are included

## Testing

### Unit Tests

```rust
#[test]
fn test_only_active_hat_instructions_included() {
    // Given: 3 hats configured, only security_reviewer has pending event
    let config = /* ... 3 hats ... */;
    let mut event_loop = EventLoop::new(config);

    // Publish security event
    event_loop.bus.publish(Event::new("review.security", "..."));

    // When: Building prompt
    let prompt = event_loop.build_prompt(&HatId::new("ralph")).unwrap();

    // Then: Only security_reviewer instructions included
    assert!(prompt.contains("Security Reviewer Instructions"));
    assert!(!prompt.contains("Correctness Reviewer Instructions"));
    assert!(!prompt.contains("Architecture Reviewer Instructions"));
}

#[test]
fn test_active_hat_display() {
    // Given: security_reviewer has pending event
    let config = /* ... */;
    let mut event_loop = EventLoop::new(config);
    event_loop.bus.publish(Event::new("review.security", "..."));

    // When: Getting active hat
    let active = event_loop.get_active_hat_id();

    // Then: Returns security_reviewer, not ralph
    assert_eq!(active.as_str(), "security_reviewer");
}
```

### Integration Test

Run the PR review preset and verify:
1. ITERATION 1 shows "🎭 ralph" (orchestration)
2. ITERATION 2 shows "🔒 security_reviewer" (after review.security published)
3. Prompt only contains security_reviewer instructions, not all 4 reviewers

## Acceptance Criteria

- [ ] Prompts include ONLY active hat instructions (not all hats)
- [ ] Iteration separator shows active hat name (not "ralph")
- [ ] Hat topology table still shows all hats (for context)
- [ ] When no specific hat is active, shows "🎭 ralph"
- [ ] When multiple hats are active, includes all their instructions
- [ ] Tests verify token savings (only active instructions included)
- [ ] Backward compatible with solo mode (no hats configured)
