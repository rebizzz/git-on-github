#!/bin/bash
set -e

rm -rf gh-pages
mkdir -p gh-pages

echo "Checking container status..."
docker ps
docker logs cgit-app

echo "Waiting for cgit HTTP response..."
for i in {1..30}; do
    if curl -s -f http://127.0.0.1:8080/ > /dev/null 2>&1; then
        echo "cgit is UP!"
        break
    fi
    echo "Attempt $i: waiting..."
    sleep 1
done

echo "Mirroring real cgit website..."
wget \
  --recursive \
  --level=5 \
  --convert-links \
  --adjust-extension \
  --page-requisites \
  --no-parent \
  --no-host-directories \
  -P gh-pages \
  http://127.0.0.1:8080/ || true

# Copy static assets directly from container as guarantee
docker cp cgit-app:/usr/share/cgit/cgit.css gh-pages/ || true
docker cp cgit-app:/usr/share/cgit/cgit.png gh-pages/ || true
docker cp cgit-app:/usr/share/cgit/favicon.ico gh-pages/ || true

# Setup index.html
if [ -f gh-pages/git-on-github.git.html ]; then
  cp gh-pages/git-on-github.git.html gh-pages/index.html
elif [ -f gh-pages/git-on-github.git/index.html ]; then
  cp gh-pages/git-on-github.git/index.html gh-pages/index.html
fi

echo "Scraping complete!"
ls -la gh-pages
