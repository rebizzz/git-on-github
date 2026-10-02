# git-on-github

A 100% authentic, high-speed **cgit** web interface hosted entirely on **GitHub Actions** and deployed to **GitHub Pages**.

## Highlights

- **Native cgit binary:** Uses the genuine official `cgit` C binary instead of mimicking HTML.
- **Blazing fast Rust parallel renderer:** Multi-threaded parallel rendering powered by `rayon` generating all repositories, branches, tags, commit diffs, tree snapshots, and patches concurrently.
- **Multi-repository support:** Configure any number of public or internal git repositories in `repos.conf`.
- **Automatic sync:** Periodic cron workflow updates all cached external repositories twice daily.
- **Client-side query dispatchers & 404 router:** Resolves cgit query parameters (`?id=`, `?h=`, `?id2=`) seamlessly on static hosting.

## Configuration

Add repositories to `repos.conf`:

```
<repo-name> <git-clone-url|local> [description] [owner] [shallow_depth]
```
