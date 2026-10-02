#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-https://rebizzz.github.io/git-on-github}"

urls=(
  "${BASE_URL}/"
  "${BASE_URL}/cgit-css/cgit.css"
  "${BASE_URL}/cgit-css/cgit.png"
  "${BASE_URL}/curl.git/"
  "${BASE_URL}/curl.git/about/"
  "${BASE_URL}/curl.git/refs/"
  "${BASE_URL}/curl.git/log/"
  "${BASE_URL}/curl.git/tree/"
  "${BASE_URL}/curl.git/tree/src"
  "${BASE_URL}/curl.git/tree/include/curl"
  "${BASE_URL}/curl.git/tree/include/curl/multi.h"
  "${BASE_URL}/curl.git/plain/include/curl/multi.h"
  "${BASE_URL}/curl.git/blame/include/curl/multi.h"
  "${BASE_URL}/cgit.git/"
  "${BASE_URL}/cgit.git/tree/cgit.c"
  "${BASE_URL}/ripgrep.git/"
  "${BASE_URL}/ripgrep.git/tree/Cargo.toml"
)

echo "Testing live deployment at: ${BASE_URL}"
failed=0
for u in "${urls[@]}"; do
  code=$(curl -s -o /dev/null -w "%{http_code}" -L "$u")
  if [ "$code" -eq 200 ]; then
    echo "  [200 OK] $u"
  else
    echo "  [FAIL $code] $u"
    failed=$((failed + 1))
  fi
done

if [ "$failed" -eq 0 ]; then
  echo "All ${#urls[@]} verification checks passed successfully!"
else
  echo "$failed check(s) failed."
  exit 1
fi
