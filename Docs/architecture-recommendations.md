# Architecture Recommendations

> Based on a full session of debugging the job system, room transitions, and stale-RPC visual glitches. The question: **What can we change on the Solana program or Unity side (or both) to make development easier and spend more time on features and game feel?**

---

## Solana Program Changes

### 1. Merge TickJob + CompleteJob (high impact)

Right now the client must send TickJob to update on-chain progress, then CompleteJob to finalize. But CompleteJob already validates `progress >= base_slots`. The program could calculate elapsed progress inline (the same math TickJob does) inside `complete_job`, eliminating one TX entirely.

```rust
// Inside complete_job handler, before the progress check:
let elapsed = clock.slot.saturating_sub(room.start_slot[dir_idx]);
let effective = elapsed * (room.helper_counts[dir_idx] as u64);
room.progress[dir_idx] = effective.min(room.base_slots[dir_idx]);
```

This removes a full round-trip TX and one stale-read window from every job completion. The standalone `tick_job` instruction can stay for UI purposes (updating the timer display), but completion no longer depends on it.

### 2. Auto-claim on CompleteJob, or merge CompleteJob + ClaimJobReward (highest impact)

This is the root cause of `TooManyActiveJobs`. The player must send a separate `ClaimJobReward` TX after completing, and if they don't (disconnect, close app, switch rooms), the job slot stays occupied forever. We built an entire cross-room cleanup system just to handle this.

Two options:
- **Option A**: Have `complete_job` also call `remove_job()` and transfer tokens in the same instruction. The helper_stake account closure and token transfer happen atomically with completion.
- **Option B**: Have `complete_job` at least clear the player's active job entry (`remove_job`), so the slot is freed even if the token claim happens later. A separate `withdraw_reward` instruction can handle the token payout without blocking new jobs.

Option B is probably cleaner from a Solana account/signer perspective, since ClaimJobReward needs additional accounts (escrow, token accounts). But either would eliminate the entire "unclaimed jobs blocking new jobs" problem.

### 3. Keep TickJob as a permissionless "UI update" instruction

TickJob is useful for keeping the client's timer display accurate. Since it's permissionless, a relayer or the client can call it periodically just to update the on-chain progress field so the UI reflects reality. But it should not be a prerequisite for completion.

---

## Unity Client Changes

### 4. Decouple TX callbacks from event-driven snapshots (highest impact)

This single architectural issue caused the majority of bugs. Every LGManager method looks like:

```csharp
await ExecuteGameplayActionAsync("SomeAction", buildIx, async (sig) => {
    await RefreshAllState();  // fires events -> snapshot rebuilds -> visual glitches
});
```

The `onSuccess` callback immediately fetches state and fires events. When chaining TXs, each callback pushes an intermediate snapshot built from potentially stale data. We had to add `SuppressEventSnapshots` / `ResumeEventSnapshots` as a band-aid.

**Recommendation**: Split TX sending from state refreshing:

- LGManager methods should just send the TX and return the signature. No `onSuccess` callbacks that fetch state.
- The *caller* decides when and how to refresh state after one or more TXs complete.
- DungeonManager (or a new coordinator class) does a single controlled refresh after a logical "action" is done.

This eliminates the need for the suppress/resume pattern entirely, and removes the source of every "stale intermediate snapshot" bug.

### 5. Use a Result type instead of returning null (medium impact)

LGManager methods currently return `null` on failure without throwing. This caused the auto-completer to silently proceed through a chain of failing TXs. Either:

- Return a `TxResult` struct with `{ bool Success, string Signature, string ErrorReason }`
- Or throw on failure so callers can't accidentally ignore it

This is a broader refactor but would prevent an entire class of "silent failure" bugs.

### 6. Formalize the optimistic state pattern (medium impact)

We added three separate optimistic flags: `_optimisticJobDirection`, `_optimisticTargetRoom`, `_roomEntryDirection`. Each handles a specific case of "TX confirmed but RPC is stale." A proper pattern would be:

```csharp
// Before sending TX:
var override = optimisticLayer.Push(expectedStateChange);

// On TX failure:
optimisticLayer.Revert(override);

// On real data arriving that matches:
optimisticLayer.Confirm(override);

// On timeout:
optimisticLayer.AutoExpire();
```

This could be a small `OptimisticStateManager` class that DungeonManager consults when building snapshots, replacing the ad-hoc fields.

### 7. Add a TX pipeline for multi-step operations (lower impact, but nice)

The auto-completer and cleanup logic both chain multiple TXs (tick -> complete -> claim). A reusable pipeline would handle:

- Sequential execution with return-value checking
- Abort-on-failure
- Single state refresh at the end
- Logging of the full chain result

---

## Priority Order

If picking three changes that would give the most time back:

1. **Merge CompleteJob + ClaimJobReward on-chain** -- eliminates the entire "unclaimed jobs" problem, the cross-room cleanup system, and the `TooManyActiveJobs` error class
2. **Decouple TX callbacks from state refreshing** -- eliminates the stale-snapshot / visual-reversion / suppress-resume complexity
3. **Make CompleteJob auto-tick** -- reduces the TX chain from 3 steps to 1, making the auto-completer trivially simple

Those three changes together would let you delete most of the workaround code (optimistic overrides, cross-room cleanup, suppress/resume, return-value checking chains) and replace it with straightforward single-TX flows.
