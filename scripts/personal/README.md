# Personal branch

`personal` is a downstream branch: `main` plus local changes. It is not a
contribution branch — upstream PRs are out of scope. The goal is a branch that
rebases onto `main` quickly, builds, and stays out of the way while you hack.

## One-time

```sh
scripts/personal/sync.sh setup
```

That turns on `rerere`, so a conflict you resolved once is resolved again
automatically. Optional alias (run it on `personal`):

```sh
git config alias.psync '!scripts/personal/sync.sh sync'
```

## The loop

```sh
git switch personal
scripts/personal/sync.sh status          # ahead/behind, local commits
scripts/personal/sync.sh sync --build    # tag, rebase onto origin/main, build
```

The script refuses a dirty tree, tags the pre-rebase state as
`personal-before-<date>`, and stops with the conflicted files if the rebase
cannot finish. On success it tags `personal-synced-<date>`.

## When it conflicts

```sh
git status
# edit, then
git add <files>
git rebase --continue
```

To back out:

```sh
scripts/personal/sync.sh abort
```

To reorder or drop a commit, use `sync --interactive` (or
`git rebase -i origin/main`). Put the easiest commits first and the
`api_client.rs` one last, so a failure leaves most of the branch applied.

## Parking a commit

If one commit keeps fighting `main`, tag it and drop it for now:

```sh
git tag personal-parked-reasoning <sha>
git rebase -i origin/main   # delete its line
```

Bring it back later with `git cherry-pick personal-parked-reasoning`.

## Known-good build

A complete removal and a release build of the app and the CLI, tagged so a
later sync can be compared against it:

```sh
scripts/personal/sync.sh known-good
```

It runs `cargo clean`, then builds `-p solidcraft -p solidcraft-cli`; `--debug`
builds debug instead. The tag is `personal-known-good-<date>`.

## Build

```sh
scripts/personal/sync.sh build            # debug app and CLI
scripts/personal/sync.sh build --release
```

For local hacking, build the app crate and run the tests you touched. Full
`cargo xtask ci` is the gate for changes you intend to share.

## Notes

- Don't `git pull` on `personal`; it tracks `fork/personal`. Fetch `origin` and
  rebase onto `origin/main`.
- Rebase at the start of a session, not after adding new work.
- Weekly is enough; don't rebase every pull.
- The existing `git sync` alias still does a plain fetch + rebase. The script
  adds the tag, the clean-tree check, conflict guidance and the optional build.
