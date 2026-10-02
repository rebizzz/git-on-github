#!/usr/bin/env python3
import os
import sys
import subprocess
import shutil
import shlex
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor

WORKSPACE = Path(os.environ.get("GITHUB_WORKSPACE", os.getcwd()))
OUT_DIR = WORKSPACE / "gh-pages"
GIT_BASE = Path(os.environ.get("CGIT_REPOS_DIR", "/tmp/cgit-repos"))
CGITRC = str(WORKSPACE / "cgitrc")
CONFIG_FILE = WORKSPACE / "repos.conf"
CGIT_BIN = shutil.which("cgit") or "/usr/lib/cgit/cgit.cgi"

def run_git(repo_path, args):
    cmd = ["git", "-C", str(repo_path)] + args
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=True)
    return res.stdout.strip()

def run_cgit(path_info, query_string=""):
    env = os.environ.copy()
    env["CGIT_CONFIG"] = CGITRC
    env["PATH_INFO"] = path_info
    env["QUERY_STRING"] = query_string
    env["HTTP_HOST"] = "rebizzz.github.io"
    env["SERVER_NAME"] = "rebizzz.github.io"
    env["HTTPS"] = "on"
    env["SCRIPT_NAME"] = ""
    
    proc = subprocess.run([CGIT_BIN], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    output = proc.stdout
    
    header_end = output.find(b"\r\n\r\n")
    if header_end != -1:
        body = output[header_end + 4:]
    else:
        header_end = output.find(b"\n\n")
        if header_end != -1:
            body = output[header_end + 2:]
        else:
            body = output
    return body

def save_page(rel_path, content):
    dest = OUT_DIR / rel_path
    dest.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(content, str):
        dest.write_text(content, encoding="utf-8")
    else:
        dest.write_bytes(content)

def make_dispatcher(repo_name, kind, default_target="HEAD"):
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>cgit {kind}</title>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/cgit.css">
</head>
<body>
<div id="cgit">
  <table id="header">
    <tr><td class="main"><a href="/git-on-github/">index</a> : <a href="/git-on-github/{repo_name}.git/">{repo_name}</a></td></tr>
  </table>
  <div class="content" style="padding: 2em; font-family: monospace;">Loading {kind}...</div>
</div>
<script>
(function() {{
  function render() {{
    var p = new URLSearchParams(window.location.search);
    var id = p.get('id') || p.get('h') || '{default_target}';
    var target = id + '.html';
    fetch(target)
      .then(function(r) {{
        if (!r.ok) return fetch('HEAD.html');
        return r;
      }})
      .then(function(r) {{ return r.text(); }})
      .then(function(html) {{
        document.open();
        document.write(html);
        document.close();
      }})
      .catch(function(err) {{
        var c = document.querySelector('.content') || document.body;
        c.innerHTML = "<div class='error'>{kind.capitalize()} not found: " + id + "</div>";
      }});
  }}
  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', render);
  }} else {{
    render();
  }}
}})();
</script>
</body>
</html>
"""

def make_patch_dispatcher():
    return """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>cgit patch</title>
</head>
<body>
<p>Loading patch...</p>
<script>
(function() {
  var p = new URLSearchParams(window.location.search);
  var id = p.get('id') || 'HEAD';
  window.location.replace(id + '.patch');
})();
</script>
</body>
</html>
"""

def make_404_router():
    return """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>404 - Not Found</title>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/cgit.css">
</head>
<body>
<div id="cgit">
  <table id="header">
    <tr><td class="main"><a href="/git-on-github/">index</a></td></tr>
  </table>
  <div class="content" style="padding: 2em; font-family: monospace;">
    <div id="status-box">Loading route...</div>
  </div>
  <div class="footer">cgit on GitHub Actions</div>
</div>

<script>
(function() {
  function handleRoute() {
    var path = window.location.pathname;
    var search = window.location.search;
    var params = new URLSearchParams(search);
    var id = params.get('id') || params.get('h') || 'HEAD';
    var id2 = params.get('id2');
    
    var cleanPath = path.replace(/\\/$/, "");
    
    function showError(msg) {
      document.title = "404 - Not Found";
      var box = document.getElementById("status-box") || document.body;
      box.innerHTML = "<div class='error'>404 - Page not found: <code>" + path + (search || "") + "</code></div>" +
                      (msg ? "<p>" + msg + "</p>" : "") +
                      "<p><a href='/git-on-github/'>&larr; Return to repository index</a></p>";
    }

    var match = cleanPath.match(/\\/git-on-github\\/([^\\/]+)\\.git(\\/.*)?$/);
    if (!match) {
      showError("Unknown path.");
      return;
    }
    
    var repoName = match[1];
    var subPath = match[2] || "";
    var repoPrefix = "/git-on-github/" + repoName + ".git";
    var target = null;
    
    if (subPath === "/commit" || subPath.endsWith("/commit")) {
      target = repoPrefix + "/commit/" + id + ".html";
    } else if (subPath === "/diff" || subPath.endsWith("/diff")) {
      target = id2 ? (repoPrefix + "/diff/" + id + "_" + id2 + ".html") : (repoPrefix + "/diff/" + id + ".html");
    } else if (subPath === "/patch" || subPath.endsWith("/patch")) {
      target = repoPrefix + "/patch/" + id + ".patch";
    } else if (subPath === "/tree" || subPath.endsWith("/tree")) {
      target = repoPrefix + "/tree/" + id + ".html";
    } else if (subPath === "/log" || subPath.endsWith("/log")) {
      target = repoPrefix + "/log/" + id + ".html";
    } else if (subPath.startsWith("/tree/")) {
      var file = subPath.replace("/tree/", "");
      target = repoPrefix + "/tree/" + file + "@id=" + id + ".html";
    } else if (subPath.startsWith("/blame/")) {
      var file = subPath.replace("/blame/", "");
      target = repoPrefix + "/blame/" + file + "@id=" + id + ".html";
    } else if (subPath.startsWith("/plain/")) {
      var file = subPath.replace("/plain/", "");
      target = repoPrefix + "/plain/" + file + "@id=" + id;
    } else if (subPath.startsWith("/diff/")) {
      var file = subPath.replace("/diff/", "");
      target = repoPrefix + "/diff/" + file + "@id=" + id + ".html";
    }
    
    if (target) {
      fetch(target)
        .then(function(r) {
          if (!r.ok) {
            if (target.includes("@id=")) {
              var fallback = target.split("@id=")[0] + ".html";
              return fetch(fallback);
            }
            throw new Error(r.status);
          }
          return r;
        })
        .then(function(r) { return r.text(); })
        .then(function(html) {
          document.open();
          document.write(html);
          document.close();
        })
        .catch(function(err) {
          showError("Target page could not be loaded.");
        });
    } else {
      showError("Unknown repository subpath.");
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', handleRoute);
  } else {
    handleRoute();
  }
})();
</script>
</body>
</html>
"""

def parse_repos_conf():
    repos = []
    if not CONFIG_FILE.exists():
        repos.append({
            "name": "git-on-github",
            "url": "local",
            "desc": "git-on-github - A git repository wholly rendered by real cgit on GitHub Actions",
            "owner": "rebizzz",
            "depth": None
        })
        return repos

    lines = CONFIG_FILE.read_text(encoding="utf-8").splitlines()
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = shlex.split(line)
        if len(parts) < 2:
            continue
        repo = {
            "name": parts[0],
            "url": parts[1],
            "desc": parts[2] if len(parts) > 2 else f"Git repository {parts[0]}",
            "owner": parts[3] if len(parts) > 3 else "Unknown",
            "depth": int(parts[4]) if len(parts) > 4 else None
        }
        repos.append(repo)
    return repos

def setup_repos(repos):
    GIT_BASE.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "config", "--global", "--add", "safe.directory", "*"], check=True)
    
    for r in repos:
        name = r["name"]
        repo_dir = GIT_BASE / f"{name}.git"
        if not repo_dir.exists():
            print(f"Setting up repository '{name}' from {r['url']}...")
            if r["url"] == "local":
                subprocess.run(["git", "clone", "--bare", str(WORKSPACE), str(repo_dir)], check=True)
            else:
                clone_cmd = ["git", "clone", "--bare"]
                if r["depth"]:
                    clone_cmd += ["--depth", str(r["depth"])]
                clone_cmd += [r["url"], str(repo_dir)]
                subprocess.run(clone_cmd, check=True)
                
            (repo_dir / "description").write_text(r["desc"] + "\n")
            subprocess.run(["git", "-C", str(repo_dir), "config", "gitweb.owner", r["owner"]], check=True)
            if r["url"] != "local":
                subprocess.run(["git", "-C", str(repo_dir), "config", "gitweb.clone-url", r["url"]], check=True)
            else:
                subprocess.run(["git", "-C", str(repo_dir), "config", "gitweb.clone-url", "https://github.com/rebizzz/git-on-github.git"], check=True)
        else:
            print(f"Updating cached repository '{name}'...")
            if r["url"] == "local":
                subprocess.run(["git", "-C", str(repo_dir), "remote", "set-url", "origin", str(WORKSPACE)], check=False)
                subprocess.run(["git", "-C", str(repo_dir), "fetch", "origin", "+refs/heads/*:refs/heads/*", "--tags", "--prune"], check=False)
            else:
                subprocess.run(["git", "-C", str(repo_dir), "fetch", "--all", "--tags", "--prune"], check=False)
            (repo_dir / "description").write_text(r["desc"] + "\n")
            subprocess.run(["git", "-C", str(repo_dir), "config", "gitweb.owner", r["owner"]], check=True)

def render_task(task):
    rel_path, path_info, query_string = task
    content = run_cgit(path_info, query_string)
    save_page(rel_path, content)

def render_repository(repo_info, pool):
    name = repo_info["name"]
    repo_dir = GIT_BASE / f"{name}.git"
    prefix = f"/{name}.git"
    rel_root = f"{name}.git"
    
    print(f"\n--- Queueing tasks for '{name}' ---")
    tasks = []
    
    # 1. Base views
    tasks.append((f"{rel_root}/index.html", f"{prefix}/", ""))
    tasks.append((f"{rel_root}/summary/index.html", f"{prefix}/", ""))
    tasks.append((f"{rel_root}/about/index.html", f"{prefix}/about/", ""))
    tasks.append((f"{rel_root}/refs/index.html", f"{prefix}/refs/", ""))
    tasks.append((f"{rel_root}/stats/index.html", f"{prefix}/stats/", ""))
    tasks.append((f"{rel_root}/atom/index.html", f"{prefix}/atom/", ""))
    tasks.append((f"{rel_root}/atom/index.xml", f"{prefix}/atom/", ""))
    tasks.append((f"{rel_root}/log/index.html", f"{prefix}/log/", ""))
    tasks.append((f"{rel_root}/tree/index.html", f"{prefix}/tree/", ""))
    
    # 2. Branches and Tags
    branches = run_git(repo_dir, ["for-each-ref", "--format=%(refname:short)", "refs/heads"]).splitlines()
    tags = run_git(repo_dir, ["for-each-ref", "--format=%(refname:short)", "refs/tags"]).splitlines()
    
    for b in branches:
        b = b.strip()
        if not b: continue
        tasks.append((f"{rel_root}/log/{b}.html", f"{prefix}/log/", f"h={b}"))
        tasks.append((f"{rel_root}/tree/{b}.html", f"{prefix}/tree/", f"h={b}"))
        
    for t in tags[:25]:
        t = t.strip()
        if not t: continue
        tasks.append((f"{rel_root}/tag/{t}.html", f"{prefix}/tag/", f"h={t}"))
        tasks.append((f"{rel_root}/commit/{t}.html", f"{prefix}/commit/", f"id={t}"))

    # 3. Commits
    commit_depth = 40 if repo_info["depth"] else 150
    commits = run_git(repo_dir, ["rev-list", f"-n{commit_depth}", "--all"]).splitlines()
    latest_commit = commits[0] if commits else "HEAD"
    
    for sha in commits:
        sha = sha.strip()
        if not sha: continue
        short_sha = sha[:7]
        
        tasks.append((f"{rel_root}/commit/{sha}.html", f"{prefix}/commit/", f"id={sha}"))
        tasks.append((f"{rel_root}/commit/{short_sha}.html", f"{prefix}/commit/", f"id={sha}"))
        tasks.append((f"{rel_root}/diff/{sha}.html", f"{prefix}/diff/", f"id={sha}"))
        tasks.append((f"{rel_root}/diff/{short_sha}.html", f"{prefix}/diff/", f"id={sha}"))
        tasks.append((f"{rel_root}/patch/{sha}.patch", f"{prefix}/patch/", f"id={sha}"))
        tasks.append((f"{rel_root}/patch/{short_sha}.patch", f"{prefix}/patch/", f"id={sha}"))
        tasks.append((f"{rel_root}/tree/{sha}.html", f"{prefix}/tree/", f"id={sha}"))
        tasks.append((f"{rel_root}/tree/{short_sha}.html", f"{prefix}/tree/", f"id={sha}"))
        
        parents = run_git(repo_dir, ["log", "-1", "--format=%P", sha]).split()
        for p in parents:
            p_sha = p.strip()
            if p_sha:
                tasks.append((f"{rel_root}/diff/{sha}_{p_sha}.html", f"{prefix}/diff/", f"id={sha}&id2={p_sha}"))
                tasks.append((f"{rel_root}/diff/{short_sha}_{p_sha[:7]}.html", f"{prefix}/diff/", f"id={sha}&id2={p_sha}"))

    tasks.append((f"{rel_root}/commit/HEAD.html", f"{prefix}/commit/", f"id={latest_commit}"))
    tasks.append((f"{rel_root}/diff/HEAD.html", f"{prefix}/diff/", f"id={latest_commit}"))
    tasks.append((f"{rel_root}/patch/HEAD.patch", f"{prefix}/patch/", f"id={latest_commit}"))
    tasks.append((f"{rel_root}/tree/HEAD.html", f"{prefix}/tree/", f"id={latest_commit}"))

    # 4. Files
    files = run_git(repo_dir, ["ls-tree", "-r", "--name-only", "HEAD"]).splitlines()
    capped_files = files if len(files) < 60 else files[:50]
    sample_commits = commits[:5]
    
    for filepath in capped_files:
        filepath = filepath.strip()
        if not filepath: continue
        tasks.append((f"{rel_root}/tree/{filepath}.html", f"{prefix}/tree/{filepath}", ""))
        tasks.append((f"{rel_root}/tree/{filepath}@id=HEAD.html", f"{prefix}/tree/{filepath}", "id=HEAD"))
        tasks.append((f"{rel_root}/blame/{filepath}.html", f"{prefix}/blame/{filepath}", ""))
        tasks.append((f"{rel_root}/blame/{filepath}@id=HEAD.html", f"{prefix}/blame/{filepath}", "id=HEAD"))
        tasks.append((f"{rel_root}/plain/{filepath}", f"{prefix}/plain/{filepath}", ""))
        
        for sha in sample_commits:
            tasks.append((f"{rel_root}/tree/{filepath}@id={sha}.html", f"{prefix}/tree/{filepath}", f"id={sha}"))
            tasks.append((f"{rel_root}/tree/{filepath}@id={sha[:7]}.html", f"{prefix}/tree/{filepath}", f"id={sha[:7]}"))
            tasks.append((f"{rel_root}/blame/{filepath}@id={sha}.html", f"{prefix}/blame/{filepath}", f"id={sha}"))
            tasks.append((f"{rel_root}/blame/{filepath}@id={sha[:7]}.html", f"{prefix}/blame/{filepath}", f"id={sha[:7]}"))
            tasks.append((f"{rel_root}/plain/{filepath}@id={sha}", f"{prefix}/plain/{filepath}", f"id={sha}"))
            tasks.append((f"{rel_root}/diff/{filepath}@id={sha}.html", f"{prefix}/diff/{filepath}", f"id={sha}"))

    print(f"Executing {len(tasks)} parallel render tasks for '{name}'...")
    list(pool.map(render_task, tasks))

    # 5. Dispatchers
    save_page(f"{rel_root}/commit/index.html", make_dispatcher(name, "commit", latest_commit))
    save_page(f"{rel_root}/diff/index.html", make_dispatcher(name, "diff", latest_commit))
    save_page(f"{rel_root}/patch/index.html", make_patch_dispatcher())
    save_page(f"{rel_root}/tree/index.html", make_dispatcher(name, "tree", "main" if "main" in branches else "master"))
    save_page(f"{rel_root}/log/index.html", make_dispatcher(name, "log", "main" if "main" in branches else "master"))

def main():
    print(f"cgit binary: {CGIT_BIN}")
    print("Reading repository configuration (repos.conf)...")
    repos = parse_repos_conf()
    print(f"Configured repositories: {[r['name'] for r in repos]}")
    
    setup_repos(repos)
    
    if OUT_DIR.exists():
        shutil.rmtree(OUT_DIR)
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    
    # Thread pool for fast parallel cgit execution across all CPU cores
    num_workers = min(32, (os.cpu_count() or 4) * 4)
    print(f"Starting parallel execution pool with {num_workers} worker threads...")
    
    with ThreadPoolExecutor(max_workers=num_workers) as pool:
        for r in repos:
            render_repository(r, pool)
        
    print("\nRendering global cgit repository index...")
    site_index = run_cgit("/")
    save_page("index.html", site_index)
    save_page("404.html", make_404_router())
    
    # Assets
    print("Copying cgit static assets...")
    for share_dir in [Path("/usr/share/cgit"), Path("/nix/var/nix/profiles/default/share/cgit"), WORKSPACE / "gh-pages"]:
        if (share_dir / "cgit.css").exists():
            save_page("cgit-css/cgit.css", (share_dir / "cgit.css").read_bytes())
            save_page("cgit-css/cgit.png", (share_dir / "cgit.png").read_bytes())
            save_page("cgit-css/favicon.ico", (share_dir / "favicon.ico").read_bytes())
            if (share_dir / "cgit.js").exists():
                save_page("cgit.js", (share_dir / "cgit.js").read_bytes())
            break
            
    print("\nAll repositories and views rendered successfully!")

if __name__ == "__main__":
    main()
