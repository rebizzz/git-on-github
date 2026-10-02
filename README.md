# git-on-github

[![Build & Deploy](https://github.com/rebizzz/git-on-github/actions/workflows/deploy.yml/badge.svg)](https://github.com/rebizzz/git-on-github/actions/workflows/deploy.yml)
[![Live Site](https://img.shields.io/badge/Live%20Demo-GitHub%20Pages-blue?style=flat&logo=github)](https://rebizzz.github.io/git-on-github/)
[![Rust](https://img.shields.io/badge/Rust-2024%20edition-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

A 100% authentic, high-speed **cgit** web interface hosted entirely on **GitHub Actions** and deployed statically to **GitHub Pages**.

🔗 **Live Site:** [https://rebizzz.github.io/git-on-github/](https://rebizzz.github.io/git-on-github/)

---

## Live Repositories

- [cgit.git](https://rebizzz.github.io/git-on-github/cgit.git/) – *A hyperfast web frontend for git repositories written in C*
- [curl.git](https://rebizzz.github.io/git-on-github/curl.git/) – *A command line tool and library for transferring data with URLs*
- [ripgrep.git](https://rebizzz.github.io/git-on-github/ripgrep.git/) – *ripgrep combines the usability of The Silver Searcher with the raw speed of grep*

---

## Features

- **Genuine cgit binary:** Renders directly using the official `cgit` C binary instead of mimicking or faking HTML.
- **Blazing-fast parallel Rust renderer:** Multi-threaded parallel rendering powered by `rayon` executing hundreds of cgit tasks across all runner CPUs in seconds.
- **Multi-job distributed matrix:** Every configured repository compiles in its own isolated runner job concurrently.
- **Multi-repository support:** Host and view multiple git repositories under a single unified cgit index.
- **Client-side query dispatchers & 404 router:** Seamlessly resolves native cgit query parameters (`?id=`, `?h=`, `?id2=`) on static hosting.
- **Automatic synchronization:** Scheduled GitHub Actions cron automatically fetches and re-renders upstream repositories twice daily.

---

## Configuration

Add or edit repositories in `repos.conf`:

```conf
# Format: <repo-name> <git-clone-url> [description] [owner] [shallow_depth]
cgit https://git.zx2c4.com/cgit "A hyperfast web frontend for git repositories written in C" "Jason A. Donenfeld" 50
curl https://github.com/curl/curl.git "A command line tool and library for transferring data with URLs" "Daniel Stenberg" 50
ripgrep https://github.com/BurntSushi/ripgrep.git "ripgrep combines usability with raw speed" "Andrew Gallant" 50
```
