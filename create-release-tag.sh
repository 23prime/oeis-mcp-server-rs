#!/bin/bash

set -eu

# Check required commands
for cmd in cargo jq git; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    printf "[ERROR] Required command '%s' not found. Please install it.\n" "$cmd" >&2
    exit 1
  fi
done

# Get version from Cargo.toml
version=$(cargo metadata --no-deps --format-version 1 --quiet | jq -r '.packages[0].version')

if [ -z "$version" ]; then
  printf "[ERROR] Failed to get version from Cargo.toml\n" >&2
  exit 1
fi

tag="v${version}"

printf "Creating release tag: %s\n" "$tag"

# Check if tag already exists
if git rev-parse "$tag" >/dev/null 2>&1; then
  printf "[ERROR] Tag '%s' already exists\n" "$tag" >&2
  exit 1
fi

# Create annotated tag
git tag -a "$tag" -m "Release ${tag}"

printf "Tag '%s' created successfully\n" "$tag"
printf "To push the tag, run: git push --follow-tags\n"
