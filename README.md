# git-on-github

A proof-of-concept static Git web interface inspired by **cgit**, powered purely by **GitHub Actions** and deployed to **GitHub Pages**.

## Features

- **Summary View:** Overview of repository, README rendering, recent commits, branches.
- **Commit Log:** Full commit log history with authors, timestamps, and commit messages.
- **Commit Details & Diffs:** Syntax-colored unified diffs with statistics.
- **Raw Patch Export:** Export raw standard Git mbox format `.patch` files directly.
- **Tree Browser:** Directory navigation and file permissions matching Git modes.
- **Blob Viewer:** File inspection and raw content download.
- **Atom Feed:** Syndication feed for latest repository commits.
- **cgit Aesthetic:** Lightweight, table-based, responsive monospace theme.

## How it Works

When you push commits to GitHub, a GitHub Actions workflow executes `generate-git-web.js`. The script parses the Git database directly using low-level Git plumbing commands (`git ls-tree`, `git cat-file`, `git show`, `git log`) and renders a completely static cgit-like web frontend to `gh-pages/`. The artifact is automatically published to GitHub Pages.
