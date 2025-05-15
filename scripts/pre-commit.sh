#!/usr/bin/env bash
set -euo pipefail

# This script is used to run pre-commit hooks for the project.

# Loop through all staged files
for file in $(git diff --cached --name-only); do
  if [[ $file == *.rs ]]; then
    echo "Checking $file with cargo fmt"
    cargo fmt -- "$file"
    git add "$file"
  elif [[ $file == *.sh ]]; then
    echo "Checking $file with shellcheck"
    shellcheck "$file"
  else
    echo "Not checking $file"
    continue
  fi
done
