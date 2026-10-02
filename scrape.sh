#!/usr/bin/env bash
set -e

rm -rf gh-pages
mkdir -p gh-pages

RUNTIME="docker"
if command -v podman >/dev/null 2>&1 && ! command -v docker >/dev/null 2>&1; then
    RUNTIME="podman"
fi

echo "Running comprehensive cgit rendering inside container using $RUNTIME..."
$RUNTIME exec cgit-app python3 /render-all-cgit.py

echo "Copying rendered cgit files out of container..."
$RUNTIME cp cgit-app:/out/. gh-pages/

echo "Build complete! Top-level files in gh-pages:"
ls -la gh-pages/
echo "Commit pages:"
ls -la gh-pages/git-on-github.git/commit/
