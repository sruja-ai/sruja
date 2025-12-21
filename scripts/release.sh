#!/usr/bin/env bash
set -euo pipefail

semver_compare() {
  local a=$1 b=$2
  printf '%s\n%s' "$a" "$b" | sort -V | head -n1 | grep -qx "$a"
}

latest_stable_tag() {
  git fetch --tags >/dev/null 2>&1 || true
  git tag --list | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -n1
}

rc_tags_for_base() {
  local base="$1"
  git tag --list | grep -E "^${base}-RC[0-9]+$" | sort -V
}

next_rc_tag() {
  local base="$1"
  local last_rc
  last_rc=$(rc_tags_for_base "$base" | tail -n1 || true)
  if [[ -z "$last_rc" ]]; then
    echo "${base}-RC1"
  else
    local n
    n=$(echo "$last_rc" | sed -E 's/^.*-RC([0-9]+)$/\1/')
    echo "${base}-RC$((n+1))"
  fi
}

derive_release_line() {
  # From version like 0.1.0 -> release/0.1.x
  local v="$1"
  local major minor patch
  IFS='.' read -r major minor patch <<< "$v"
  echo "release/${major}.${minor}.x"
}

create_hotfix_branch() {
  local patch_version="$1"
  local base_tag
  base_tag=$(latest_stable_tag)
  if [[ -z "$base_tag" ]]; then
    echo "No stable tags found" >&2
    exit 1
  fi
  git checkout -b "hotfix/${patch_version}" "${base_tag}"
}

tag_and_push() {
  local version="$1"
  git tag -a "$version" -m "Release $version"
  git push origin "$version"
}

usage() {
  echo "Usage: release.sh <command> [args]"
  echo "Commands:"
  echo "  latest-stable                 Print latest stable tag"
  echo "  next-rc <base>               Print next RC tag for base (e.g., 0.1.0)"
  echo "  hotfix-branch <patch>        Create hotfix/<patch> from latest stable"
  echo "  tag <version>                Create and push tag"
}

cmd=${1:-}
case "$cmd" in
  latest-stable)
    latest_stable_tag ;;
  next-rc)
    base=${2:-}
    if [[ -z "$base" ]]; then usage; exit 1; fi
    next_rc_tag "$base" ;;
  hotfix-branch)
    pv=${2:-}
    if [[ -z "$pv" ]]; then usage; exit 1; fi
    create_hotfix_branch "$pv" ;;
  tag)
    v=${2:-}
    if [[ -z "$v" ]]; then usage; exit 1; fi
    tag_and_push "$v" ;;
  *)
    usage; exit 1 ;;
esac

