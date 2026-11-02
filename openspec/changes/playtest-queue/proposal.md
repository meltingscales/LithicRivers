# Proposal

## Why

Several changes are merged with every task done except a "user check": a playtest only the user can do, because it judges feel or needs real hardware. Those checks are scattered across the changes' task lists, so it's hard to see what's waiting, and the changes stay open long after their code shipped. The user wants one list to work through in their free time, and wants future user checks to land on it too.

## What Changes

- **One playtest queue:** this change's `tasks.md` lists every change that is waiting only on a user check, with what to check, how to set it up quickly in the real game (cheats, debug worlds), and where to note a failure.
- **Closing a change out:** when an entry passes, the user ticks it here and in the change, and the change is archived in its own PR (as before). A failed check gets a note and becomes a fix branch or a change of its own; the entry stays until the fix passes.
- **Future checks join the queue:** a project rule in `openspec/config.yaml` says that when a change is merged with only user checks left, its checks get an entry here. Long checklists (`check-windows-and-deck`, `polish-android`) get a one-line entry pointing at their own tasks rather than a copy.
- This change is never "done" while entries remain. When the queue is empty it is archived, and the next change waiting on a playtest starts a new one with the same name.

Out of scope: running the checks (only the user can), and changes that still have unfinished agent tasks.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None: the queue tracks checks of behavior other changes already specify, so this change sets `skip_specs`.

## Impact

- `openspec/changes/playtest-queue/` and one task rule in `openspec/config.yaml`.
- No code changes.
