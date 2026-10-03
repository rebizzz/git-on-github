#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-https://rebizzz.github.io/git-on-github}"

endpoints=(
  "/"
  "/cgit-css/cgit.css"
  "/cgit-css/cgit.png"
  "/cgit-css/favicon.ico"
  "/cgit.js"
  "/404.html"

  "/cgit.git/"
  "/cgit.git/about/"
  "/cgit.git/refs/"
  "/cgit.git/log/"
  "/cgit.git/tree/"
  "/cgit.git/stats/"
  "/cgit.git/atom/index.xml"
  "/cgit.git/commit/HEAD.html"
  "/cgit.git/diff/HEAD.html"
  "/cgit.git/patch/HEAD.patch"
  "/cgit.git/tree/cgit.c"
  "/cgit.git/plain/cgit.c"
  "/cgit.git/blame/cgit.c"
  "/cgit.git/tree/cgit.h"
  "/cgit.git/plain/cgit.h"
  "/cgit.git/tree/Makefile"
  "/cgit.git/plain/Makefile"
  "/cgit.git/tree/filters"
  "/cgit.git/tree/filters/syntax-highlighting.py"

  "/curl.git/"
  "/curl.git/about/"
  "/curl.git/refs/"
  "/curl.git/log/"
  "/curl.git/tree/"
  "/curl.git/stats/"
  "/curl.git/atom/index.xml"
  "/curl.git/commit/HEAD.html"
  "/curl.git/diff/HEAD.html"
  "/curl.git/patch/HEAD.patch"
  "/curl.git/tree/CMakeLists.txt"
  "/curl.git/plain/CMakeLists.txt"
  "/curl.git/tree/src"
  "/curl.git/tree/src/"
  "/curl.git/tree/src/tool_main.c"
  "/curl.git/plain/src/tool_main.c"
  "/curl.git/blame/src/tool_main.c"
  "/curl.git/tree/lib"
  "/curl.git/tree/lib/urldata.h"
  "/curl.git/plain/lib/urldata.h"
  "/curl.git/tree/include"
  "/curl.git/tree/include/curl"
  "/curl.git/tree/include/curl/"
  "/curl.git/tree/include/curl/curl.h"
  "/curl.git/plain/include/curl/curl.h"
  "/curl.git/tree/include/curl/multi.h"
  "/curl.git/plain/include/curl/multi.h"
  "/curl.git/blame/include/curl/multi.h"

  "/ripgrep.git/"
  "/ripgrep.git/about/"
  "/ripgrep.git/refs/"
  "/ripgrep.git/log/"
  "/ripgrep.git/tree/"
  "/ripgrep.git/stats/"
  "/ripgrep.git/atom/index.xml"
  "/ripgrep.git/commit/HEAD.html"
  "/ripgrep.git/diff/HEAD.html"
  "/ripgrep.git/patch/HEAD.patch"
  "/ripgrep.git/tree/Cargo.toml"
  "/ripgrep.git/plain/Cargo.toml"
  "/ripgrep.git/tree/crates"
  "/ripgrep.git/tree/crates/core"
  "/ripgrep.git/tree/crates/core/main.rs"
  "/ripgrep.git/plain/crates/core/main.rs"
  "/ripgrep.git/blame/crates/core/main.rs"
)

echo "Testing live deployment at: ${BASE_URL} (in parallel)..."

check_endpoint() {
  local base="$1"
  local ep="$2"
  local url="${base}${ep}"
  local code
  code=$(curl -s -o /dev/null -w "%{http_code}" -L "$url")
  if [ "$code" -eq 200 ]; then
    echo "PASS 200 $ep"
  else
    echo "FAIL $code $ep"
    return 1
  fi
}
export -f check_endpoint

printf "%s\n" "${endpoints[@]}" | xargs -n 1 -P 16 -I {} bash -c "check_endpoint \"$BASE_URL\" \"{}\"" | sort

echo ""
echo "Verifying content integrity..."
curl -s -L "$BASE_URL/curl.git/plain/include/curl/multi.h" | grep -q "CURLINC_MULTI_H"
echo "✓ curl multi.h plain content verified"

curl -s -L "$BASE_URL/ripgrep.git/plain/crates/core/main.rs" | grep -q "fn main"
echo "✓ ripgrep main.rs plain content verified"

curl -s -L "$BASE_URL/cgit.git/plain/cgit.c" | grep -q "main("
echo "✓ cgit cgit.c plain content verified"

curl -s -L "$BASE_URL/curl.git/patch/HEAD.patch" | grep -q "diff --git"
echo "✓ patch/HEAD.patch format verified"

curl -s -L "$BASE_URL/curl.git/atom/index.xml" | grep -q "<feed xmlns="
echo "✓ atom/index.xml feed verified"

curl -s -L "$BASE_URL/curl.git/tree/include/curl/multi.h" | grep -q "<td class=.lines.>"
echo "✓ tree syntax/line view verified"

# Also verify markdown-body on about page
curl -s -L "$BASE_URL/cgit.git/about/" | grep -q "class='markdown-body'"
echo "✓ cgit about markdown-body verified"

curl -s -L "$BASE_URL/curl.git/about/" | grep -q "class='markdown-body'"
echo "✓ curl about markdown-body verified"

curl -s -L "$BASE_URL/ripgrep.git/about/" | grep -q "class='markdown-body'"
echo "✓ ripgrep about markdown-body verified"

echo ""
echo "ALL TESTS & INTEGRITY CHECKS PASSED!"
