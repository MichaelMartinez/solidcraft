#!/usr/bin/env bash
# Keep the downstream `personal` branch on top of upstream `main`.
#
# `personal` is not a contribution branch: it is `main` plus local changes.
# This helper tags the state before a rebase, stops with instructions on a
# conflict, and can build the app afterwards.
#
#   scripts/personal/sync.sh status
#   scripts/personal/sync.sh sync [--build] [--release] [--interactive] [--dry-run]
#   scripts/personal/sync.sh known-good [--debug]
#   scripts/personal/sync.sh abort
#   scripts/personal/sync.sh tag [name]
#   scripts/personal/sync.sh build [--release]
#   scripts/personal/sync.sh setup
#
# Environment: PERSONAL_BRANCH, PERSONAL_REMOTE, PERSONAL_UPSTREAM,
#              PERSONAL_BUILD_PKG, PERSONAL_CLI_PKG
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

branch=${PERSONAL_BRANCH:-personal}
remote=${PERSONAL_REMOTE:-origin}
upstream=${PERSONAL_UPSTREAM:-main}
build_pkg=${PERSONAL_BUILD_PKG:-solidcraft}
cli_pkg=${PERSONAL_CLI_PKG:-solidcraft-cli}
target="$remote/$upstream"

say() { printf '%s\n' "$*"; }
warn() { printf 'warning: %s\n' "$*" >&2; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }
stamp() { date +%Y%m%d-%H%M%S; }

in_rebase() {
  [ -d "$(git rev-parse --git-path rebase-merge)" ] ||
    [ -d "$(git rev-parse --git-path rebase-apply)" ]
}

require_branch() {
  local current
  current=$(git branch --show-current)
  [ "$current" = "$branch" ] || die "on '$current', not '$branch'; run: git switch $branch"
}

require_clean() {
  if [ -n "$(git status --porcelain)" ]; then
    git status --short
    die "working tree is dirty; commit or stash it first"
  fi
}

latest_tag() {
  git tag --list "$1*" --sort=-creatordate | sed -n '1p'
}

fetch() {
  say "fetching $remote..."
  git fetch "$remote" --prune --quiet
}

counts() {
  ahead=$(git rev-list --count "$target"..HEAD)
  behind=$(git rev-list --count HEAD.."$target")
}

show_counts() {
  counts
  say "$branch: $ahead commit(s) ahead, $behind behind $target"
}

build_apps() {
  local release=$1
  if [ "$release" -eq 1 ]; then
    cargo build --release -p "$build_pkg" -p "$cli_pkg"
  else
    cargo build -p "$build_pkg" -p "$cli_pkg"
  fi
}

cmd=${1:-status}
if [ $# -gt 0 ]; then shift; fi

case "$cmd" in
  setup)
    git config rerere.enabled true
    git config rerere.autoupdate true
    say "rerere enabled: recorded conflict resolutions are reused"
    ;;

  known-good)
    require_branch
    require_clean
    release=1
    for arg in "$@"; do
      case "$arg" in
        --debug) release=0 ;;
        *) die "unknown option: $arg" ;;
      esac
    done
    say "complete removal: cargo clean"
    cargo clean
    build_apps "$release"
    tag="personal-known-good-$(stamp)"
    git tag "$tag"
    say "known good; tagged $tag"
    ;;

  status)
    require_branch
    fetch
    show_counts
    if in_rebase; then warn "a rebase is in progress"; fi
    if [ -n "$(git status --porcelain)" ]; then warn "working tree has changes"; fi
    say ""
    say "local commits:"
    git log --oneline "$target"..HEAD | sed 's/^/  /'
    if [ -n "$(git tag --list 'personal-before-*')" ]; then
      say ""
      say "last pre-rebase tag: $(latest_tag personal-before-)"
    fi
    ;;

  sync)
    build=0
    release=0
    interactive=0
    dry=0
    for arg in "$@"; do
      case "$arg" in
        --build) build=1 ;;
        --release) build=1; release=1 ;;
        --interactive|-i) interactive=1 ;;
        --dry-run) dry=1 ;;
        *) die "unknown option: $arg" ;;
      esac
    done
    require_branch
    require_clean
    if in_rebase; then die "a rebase is already in progress; run: $0 abort"; fi
    if [ "$(git config --get rerere.enabled || true)" != "true" ]; then
      warn "rerere is off; run: $0 setup"
    fi
    fetch
    show_counts
    if [ "$behind" -eq 0 ]; then
      say "already up to date"
      exit 0
    fi
    tag="personal-before-$(stamp)"
    if [ "$dry" -eq 1 ]; then
      say "would tag $tag and rebase onto $target"
      exit 0
    fi
    git tag "$tag"
    say "tagged $tag"
    if [ "$interactive" -eq 1 ]; then
      rebase_args=(-i "$target")
    else
      rebase_args=("$target")
    fi
    if git rebase "${rebase_args[@]}"; then
      synced="personal-synced-$(stamp)"
      git tag "$synced"
      say "rebased onto $(git rev-parse --short "$target")"
      say "tagged $synced"
      if [ "$build" -eq 1 ]; then
        build_apps "$release" || die "build failed; the rebase is applied and tagged $synced"
        say "build ok"
      fi
    else
      say ""
      say "conflicts:"
      git diff --name-only --diff-filter=U | sed 's/^/  /'
      say ""
      say "resolve them, then: git add <files> && git rebase --continue"
      say "to back out:        $0 abort"
      say "pre-rebase state:   $tag"
      exit 1
    fi
    ;;

  abort)
    require_branch
    if ! in_rebase; then die "no rebase in progress"; fi
    git rebase --abort
    say "rebase aborted; back at $(git rev-parse --short HEAD)"
    ;;

  tag)
    require_branch
    name=${1:-personal-$(stamp)}
    if git rev-parse -q --verify "refs/tags/$name" >/dev/null; then
      die "tag $name already exists"
    fi
    git tag "$name"
    say "tagged $name at $(git rev-parse --short HEAD)"
    ;;

  build)
    require_branch
    release=0
    for arg in "$@"; do
      case "$arg" in
        --release) release=1 ;;
        *) die "unknown option: $arg" ;;
      esac
    done
    build_apps "$release"
    say "build ok"
    ;;

  *)
    die "unknown command: $cmd (status, sync, known-good, abort, tag, build, setup)"
    ;;
esac
