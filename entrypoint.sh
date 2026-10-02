#!/bin/sh
set -e

# Setup directories
mkdir -p /run /var/git/git-on-github.git
git clone --bare /repo /var/git/git-on-github.git
git -C /var/git/git-on-github.git config gitweb.description "A real cgit instance running inside Docker and scraped via GitHub Actions"
git -C /var/git/git-on-github.git config gitweb.owner "rebizzz"

# Start fcgiwrap
spawn-fcgi -s /run/fcgiwrap.sock -u nginx -g nginx -- /usr/bin/fcgiwrap
chmod 777 /run/fcgiwrap.sock

# Start nginx in background
nginx

# Keep script waiting on nginx
wait -n
