#!/usr/bin/env bash
set -e

rm -rf gh-pages
mkdir -p gh-pages/cgit-css

RUNTIME="docker"
if command -v podman >/dev/null 2>&1 && ! command -v docker >/dev/null 2>&1; then
    RUNTIME="podman"
fi

echo "Scraping cgit using runtime: $RUNTIME"

$RUNTIME exec cgit-app bash -c "
set -e
rm -rf /tmp/scraped && mkdir -p /tmp/scraped

until curl -s http://127.0.0.1/git-on-github.git/ > /dev/null; do
    sleep 0.5
done

# Mirror the entire repo view
wget \
  --recursive \
  --level=3 \
  --convert-links \
  --adjust-extension \
  --page-requisites \
  --no-parent \
  --no-host-directories \
  -P /tmp/scraped \
  http://127.0.0.1/git-on-github.git/ || true

# Grab static resources
wget -q -P /tmp/scraped/cgit-css/ http://127.0.0.1/cgit-css/cgit.css || true
wget -q -P /tmp/scraped/cgit-css/ http://127.0.0.1/cgit-css/cgit.png || true
wget -q -P /tmp/scraped/cgit-css/ http://127.0.0.1/cgit-css/favicon.ico || true
wget -q -P /tmp/scraped/ http://127.0.0.1/cgit.js || true
"

$RUNTIME cp cgit-app:/tmp/scraped/. gh-pages/

# Fallback direct file copy
$RUNTIME cp cgit-app:/usr/share/cgit/cgit.css gh-pages/cgit-css/ || true
$RUNTIME cp cgit-app:/usr/share/cgit/cgit.png gh-pages/cgit-css/ || true
$RUNTIME cp cgit-app:/usr/share/cgit/cgit.js gh-pages/ || true
$RUNTIME cp cgit-app:/usr/share/cgit/favicon.ico gh-pages/cgit-css/ || true

# Fix root index
if [ -f gh-pages/git-on-github.git.html ]; then
  cp gh-pages/git-on-github.git.html gh-pages/index.html
elif [ -f gh-pages/git-on-github.git/index.html ]; then
  cp gh-pages/git-on-github.git/index.html gh-pages/index.html
fi

# Convert any absolute http://127.0.0.1 references to relative paths for GitHub Pages
find gh-pages -type f -name "*.html" -exec sed -i 's|http://127.0.0.1/git-on-github.git/|./|g' {} +
find gh-pages -type f -name "*.html" -exec sed -i 's|http://127.0.0.1/cgit-css/|cgit-css/|g' {} +
find gh-pages -type f -name "*.html" -exec sed -i 's|http://127.0.0.1/cgit.js|cgit.js|g' {} +
find gh-pages -type f -name "*.html" -exec sed -i 's|/cgit-css/|cgit-css/|g' {} +
find gh-pages -type f -name "*.html" -exec sed -i 's|/cgit.js|cgit.js|g' {} +

echo "cgit scrape complete!"
