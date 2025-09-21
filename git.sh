#!/bin/bash
# This script removes the esp.img file from the entire Git history.

echo "Rewriting history to remove the esp.img file..."

# The core command to filter the history.
# It iterates through every commit and removes the specified file.
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch thatte-starter/build/esp.img' \
  --prune-empty --tag-name-filter cat -- --all

echo "History rewritten."
echo "Cleaning up repository..."

# These commands remove the backup refs created by filter-branch
# and run garbage collection to finalize the removal of the old data.
git for-each-ref --format='delete %(refname)' refs/original | git update-ref --stdin
git reflog expire --expire=now --all
git gc --prune=now --aggressive

echo "Cleanup complete. The repository is now ready for a force-push."