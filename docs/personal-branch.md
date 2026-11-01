# The personal branch

`personal` is a downstream branch: `main` plus local changes. It is not a
contribution branch; no PRs come from it. The rule is: **work on `personal`,
never start or build `main`.**

The helper is `scripts/personal/sync.sh`; `git psync` is an alias for
`scripts/personal/sync.sh sync`.

## One-time setup

```sh
scripts/personal/sync.sh setup
git config alias.psync '!scripts/personal/sync.sh sync'
```

`setup` turns on `rerere`, so a conflict you resolved once is resolved again
automatically.

## The daily loop

```sh
git switch personal
# hack, commit, repeat
git psync --dry-run     # ahead/behind; changes nothing
git psync               # tag, rebase onto origin/main, stop on conflict
git psync --build       # the same, then build app + CLI
```

- The script refuses a dirty tree, a branch other than `personal`, and a
  rebase already in progress.
- It tags `personal-before-<date>` before starting.
- It rebases your commits onto `origin/main`; `main` itself is never touched.
- On success it tags `personal-synced-<date>`.

## Catch up to main

`git psync` (or `scripts/personal/sync.sh sync`) is the whole answer. It
fetches `origin`, then replays `personal` onto `origin/main`.

You do not check out, pull, or build `main`. `origin/main` is the source; the
local `main` branch may stay stale. If you want to move the local `main` ref
without switching:

```sh
git fetch origin main:main
```

## Check upstream

```sh
scripts/personal/sync.sh status
```

It fetches and prints how many commits `personal` is ahead and behind, the
local commits, and any rebase in progress. Without the script:

```sh
git fetch origin
git log --oneline personal..origin/main | head
git rev-list --count personal..origin/main
```

## When it conflicts

```sh
git status
# edit the files
git add <files>
git rebase --continue
```

Back out:

```sh
git psync abort
```

The pre-rebase tag (`personal-before-<date>`) is your reset point. After a
hand-resolved rebase the script has already exited, so tag the result
yourself:

```sh
scripts/personal/sync.sh tag personal-synced-$(date +%Y%m%d-%H%M%S)
```

To reorder or drop commits, start with `git psync --interactive` (or
`git rebase -i origin/main`). Put the easiest commits first and the
`api_client.rs` one last, so a failure leaves most of the branch applied.

## Update the app

```sh
scripts/personal/sync.sh build              # debug app + CLI
scripts/personal/sync.sh build --release    # release
scripts/personal/sync.sh sync --build       # catch up, then build
```

Binaries: `target/debug/solidcraft` and `target/debug/solidcraft-cli` (or
`target/release/...`). Run the app from there; don't launch the installed
SolidCraft app for personal testing.

A clean baseline, when the build is suspect or before a risky sync:

```sh
scripts/personal/sync.sh known-good          # cargo clean + release
scripts/personal/sync.sh known-good --debug
```

It removes `target/` first, builds `-p solidcraft -p solidcraft-cli`, and tags
`personal-known-good-<date>`.

For Dutchman's history check, point it at the personal CLI:

```sh
dutchman inspect part.step --solidcraft target/release/solidcraft-cli
# or: export SOLIDCRAFT_CLI="$PWD/target/release/solidcraft-cli"
```

## Tags and rollback

| Tag | Meaning |
|---|---|
| `personal-before-<date>` | state before a rebase |
| `personal-synced-<date>` | state after a successful rebase |
| `personal-known-good-<date>` | clean build baseline |
| `personal-parked-<name>` | a commit dropped for now |

Roll back to a tag only when you mean it:

```sh
git reset --hard personal-before-<date>
```

`git reflog` has the rest.

## Parking a commit

If one commit keeps fighting `main`:

```sh
git tag personal-parked-reasoning <sha>
git rebase -i origin/main   # delete its line
```

Bring it back later:

```sh
git cherry-pick personal-parked-reasoning
```

## Don't

- Don't `git pull` on `personal`; it tracks `fork/personal`. Fetch `origin` and
  rebase.
- Don't sync with uncommitted changes; commit or stash first.
- Don't rebase on every pull; weekly or before a new feature is enough.
- Don't rewrite hot files inline. Put local logic in a new module with one or
  two hook lines; that is what keeps the rebase cheap.
- Don't open PRs from `personal`; this branch is downstream by choice.
