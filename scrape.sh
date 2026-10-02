#!/bin/bash
set -e

rm -rf gh-pages
mkdir -p gh-pages

echo "Waiting for cgit to start..."
until curl -s http://localhost:8080/ > /dev/null; do
    sleep 1
done

echo "Mirroring cgit..."
# Mirror whole cgit repository view
wget \
  --mirror \
  --convert-links \
  --adjust-extension \
  --page-requisites \
  --no-parent \
  --no-host-directories \
  -P gh-pages \
  http://localhost:8080/ || true

# Also grab specific key views to ensure they exist
for path in \
  "" \
  "cgit.css" \
  "cgit.png" \
  "favicon.ico" \
  "git-on-github.git/" \
  "git-on-github.git/log/" \
  "git-on-github.git/tree/" \
  "git-on-github.git/refs/" \
  "git-on-github.git/about/" \
  "git-on-github.git/stats/"; do
  wget -q -P gh-pages -nH -x -k -E "http://localhost:8080/$path" || true
done

# Setup entry index.html to redirect or serve the repo directly
if [ -f gh-pages/git-on-github.git.html ]; then
  cp gh-pages/git-on-github.git.html gh-pages/index.html
elif [ -d gh-pages/git-on-github.git ]; then
  cp gh-pages/git-on-github.git/index.html gh-pages/index.html || true
fi

echo "Scraping complete!"
