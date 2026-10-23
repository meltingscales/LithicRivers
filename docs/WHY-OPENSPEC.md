# Why OpenSpec instead of one big changelog

A walkthrough using a real feature in this repo: a **nuclear slime** enemy that rarely drops nuclear waste (now archived at `openspec/changes/archive/2026-10-22-add-nuclear-slime/`). Every command below was run in this repo, and the outputs are copied from it.

## The problem with a big changelog

`CHANGELOG.txt` records what shipped, one line at a time:

```
# v0.9.0.2
- make tutorial system usable

# v0.9.0.4
- choco pls fix ur infra
```

That's fine as a diary, but it can't answer the questions you have when you build the next feature:

- **What is true right now?** Line 40 might be undone by line 12. You have to replay the whole history in your head.
- **Why was it done this way?** The reasons, the rejected alternatives and the edge cases aren't there.
- **How do I know it works?** There are no acceptance criteria to test against.
- **What's in progress?** Half-finished features either aren't in it or are mixed in with finished ones.

A long design doc with every feature in it has the opposite problem. Everything is there, but to work on one feature you read all of them.

## How OpenSpec splits it up

```
openspec/
  specs/                      what the game does now, one small file per capability
    cheat-console/spec.md
    crafting/spec.md
    ...                       (14 capabilities today)
  changes/                    one folder per feature in progress
    add-nuclear-slime/
      proposal.md             why, and what changes
      specs/                  only the requirements this feature adds or changes
      design.md               how, and what was rejected
      tasks.md                checkboxes, each with how it's verified
    archive/                  finished changes, dated: the history
```

- **Specs** are the current truth, written as requirements with WHEN/THEN scenarios.
- **A change** is a self-contained plan for one feature.
- **Archiving** a finished change merges its requirements into `specs/` and moves the folder to `archive/`, so the history is still there, just out of the way.

## Walkthrough: adding the nuclear slime

**1. See what exists.**

```
$ openspec list --specs
Specs:
  cheat-console            requirements 7
  crafting                 requirements 4
  factory-blocks           requirements 6
  ...
```

The slime needs a new `spawn_enemy` kind, and `cheat-console` is the capability that owns that.

**2. Start a change.**

```
$ openspec new change add-nuclear-slime
Created change 'add-nuclear-slime' at openspec/changes/add-nuclear-slime/

$ openspec status --change add-nuclear-slime
Progress: 0/4 artifacts complete
[ ] proposal
[-] specs (blocked by: proposal)
[-] design (blocked by: proposal)
[-] tasks (blocked by: specs, design)
```

**3. Write the artifacts.** `openspec instructions <artifact> --change add-nuclear-slime --json` prints the template and rules for each artifact. In Claude Code, `/opsx:propose add a nuclear slime enemy...` writes all four. The important part is the spec delta, which holds only what changes:

```markdown
## ADDED Requirements

### Requirement: Rare nuclear waste drop
Each nuclear slime SHALL have a 5% chance to carry one nuclear waste, rolled when it
spawns from the world seed and its spawn position. ...

#### Scenario: Drop rate
- **WHEN** 2000 slimes are spawned at different positions in one world
- **THEN** between 3% and 7% of them carry nuclear waste
```

The slime also changes an existing rule: `spawn_enemy` used to support only `feral_dog`. So the change holds a **MODIFIED** copy of that one requirement, with `nuclear_slime` added, rather than a note saying "also slimes now".

**4. Check it.**

```
$ openspec validate add-nuclear-slime --strict
Change 'add-nuclear-slime' is valid

$ openspec show add-nuclear-slime --json --deltas-only    # JSON; one line per delta here
cheat-console  MODIFIED  Spawn enemy
nuclear-slime  ADDED     Slow chase
nuclear-slime  ADDED     Combat stats
nuclear-slime  ADDED     Rare nuclear waste drop
nuclear-slime  ADDED     Underground spawning
nuclear-slime  ADDED     Nuclear waste is a raw source
```

**5. Build it.** `/opsx:apply add-nuclear-slime` (or `openspec instructions apply --change add-nuclear-slime --json`) hands the agent the task list. The agent ticks each box only when its check passes: a test, a headless script, a snapshot, or a "user check" for a human.

**6. Finish it.** `/opsx:archive add-nuclear-slime`:

- merges the ADDED requirements into a new `openspec/specs/nuclear-slime/spec.md`;
- replaces `Spawn enemy` in `openspec/specs/cheat-console/spec.md` with the modified version;
- moves the folder to `openspec/changes/archive/<date>-add-nuclear-slime/`.

Everything else in `specs/` is untouched.

## Why it saves tokens

An AI agent pays for everything it reads. These numbers are from this repo (characters ÷ 4 ≈ tokens, a rough rule of thumb):

| What the agent reads | Characters | ≈ Tokens |
|---|---:|---:|
| The `add-nuclear-slime` change (all four artifacts) | 10,817 | 2,700 |
| The one main spec it touches (`cheat-console`) | 4,337 | 1,100 |
| **Total to start work on the slime** | **15,154** | **3,800** |
| Every plan in one place (all change folders, open and archived) | 286,213 | 71,600 |
| All Rust source in `crates/` | 1,349,833 | 337,500 |
| `CHANGELOG.txt` | 3,241 | 800 |

- **Small, targeted reading.** With OpenSpec, the agent reads about 3,800 tokens of exactly the right context, then opens only the code files the design names.
- **The changelog is small but not useful here.** At 800 tokens it says nothing about how enemies, loot or the cheat console work, so the agent falls back to searching the source (337,500 tokens in total).
- **One big feature doc costs 19× more.** Keeping every plan in one document makes the agent read about 71,600 tokens to find the slime's part, about 19 times more.

It saves tokens later as well:

- **Every new session starts from the same files**, instead of from a long chat history or a re-explanation.
- **Parallel features don't collide.** Each change is its own folder, so the slime plan and the conveyor-belt plan never get mixed into one document.
- **Reviews are small.** You read the plan before any code exists, and the diff of the spec delta shows exactly what behavior changes.

These are estimates. The agent still reads the code it changes; what OpenSpec cuts is the searching and the re-explaining.

## When a changelog still makes sense

Release notes for players are still useful. They're easy to write from the archive: each archived change's `proposal.md` "What Changes" section is a ready-made release note, and it carries the "why".

## Try it

```
openspec list                                   # features in progress
openspec list --specs                           # what the game does today
openspec show add-nuclear-slime                 # read one plan
openspec validate add-nuclear-slime --strict    # check a plan
```

In Claude Code: `/opsx:propose`, `/opsx:apply`, `/opsx:archive`. See [README.USING-AI.md](../README.USING-AI.md) for the full loop.
