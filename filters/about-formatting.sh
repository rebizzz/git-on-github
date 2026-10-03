#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
for bin in \
  "${SCRIPT_DIR}/../bin/cgit-about-filter" \
  "${SCRIPT_DIR}/../renderer/target/release/cgit-about-filter" \
  "${SCRIPT_DIR}/../renderer/target/debug/cgit-about-filter"; do
  if [ -x "$bin" ]; then
    exec "$bin" "$@"
  fi
done
exec cgit-about-filter "$@"
