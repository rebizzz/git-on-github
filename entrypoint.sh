#!/bin/sh
set -e

git config --global --add safe.directory '*'

mkdir -p /run /var/git/git-on-github.git
git clone --bare /repo /var/git/git-on-github.git
echo "git-on-github - A git repository wholly rendered by real cgit on GitHub Actions" > /var/git/git-on-github.git/description
git -C /var/git/git-on-github.git config gitweb.owner "rebizzz"
chown -R www-data:www-data /var/git

spawn-fcgi -s /run/fcgiwrap.sock -u www-data -g www-data -- /usr/sbin/fcgiwrap
chmod 777 /run/fcgiwrap.sock

exec nginx -g "daemon off;"
