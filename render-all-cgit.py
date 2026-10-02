#!/usr/bin/env python3
import os
import subprocess
import shutil
from pathlib import Path

OUT_DIR = Path("/out")
REPO_PATH = Path("/var/git/git-on-github.git")
CGITRC = "/etc/cgitrc"

def run_git(args):
    cmd = ["git", "-C", str(REPO_PATH)] + args
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
    
    proc = subprocess.run(["/usr/lib/cgit/cgit.cgi"], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
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

def make_dispatcher(kind, default_target="HEAD"):
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>cgit {kind}</title>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/cgit.css">
<script>
(function() {{
  var p = new URLSearchParams(window.location.search);
  var id = p.get('id') || p.get('h') || '{default_target}';
  var target = id + '.html';
  fetch(target)
    .then(function(r) {{
      if (!r.ok) {{
        return fetch('HEAD.html');
      }}
      return r;
    }})
    .then(function(r) {{ return r.text(); }})
    .then(function(html) {{
      document.open();
      document.write(html);
      document.close();
    }})
    .catch(function(err) {{
      document.body.innerHTML = "<div id='cgit'><div class='content'><div class='error'>{kind.capitalize()} not found: " + id + "</div></div></div>";
    }});
}})();
</script>
</head>
<body>
<div id="cgit"><div class="content" style="padding: 2em; font-family: sans-serif;">Loading {kind}...</div></div>
</body>
</html>
"""

def make_patch_dispatcher():
    return """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>cgit patch</title>
<script>
(function() {
  var p = new URLSearchParams(window.location.search);
  var id = p.get('id') || 'HEAD';
  window.location.replace(id + '.patch');
})();
</script>
</head>
<body>
<p>Loading patch...</p>
</body>
</html>
"""

def make_404_router():
    return """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>cgit</title>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/cgit.css">
<script>
(function() {
  var path = window.location.pathname;
  var search = window.location.search;
  var params = new URLSearchParams(search);
  var id = params.get('id') || params.get('h') || 'HEAD';
  var id2 = params.get('id2');
  
  var cleanPath = path.replace(/\\/$/, "");
  var repoPrefix = "/git-on-github/git-on-github.git";
  if (!cleanPath.startsWith(repoPrefix)) {
    cleanPath = cleanPath.replace("/git-on-github", repoPrefix);
  }
  
  var target = null;
  if (cleanPath.endsWith('/commit')) {
    target = repoPrefix + '/commit/' + id + '.html';
  } else if (cleanPath.endsWith('/diff')) {
    if (id2) {
      target = repoPrefix + '/diff/' + id + '_' + id2 + '.html';
    } else {
      target = repoPrefix + '/diff/' + id + '.html';
    }
  } else if (cleanPath.endsWith('/patch')) {
    target = repoPrefix + '/patch/' + id + '.patch';
  } else if (cleanPath.endsWith('/tree')) {
    target = repoPrefix + '/tree/' + id + '.html';
  } else if (cleanPath.endsWith('/log')) {
    target = repoPrefix + '/log/' + id + '.html';
  } else if (cleanPath.includes('/tree/')) {
    var sub = cleanPath.split('/tree/')[1];
    target = repoPrefix + '/tree/' + sub + '@id=' + id + '.html';
  } else if (cleanPath.includes('/blame/')) {
    var sub = cleanPath.split('/blame/')[1];
    target = repoPrefix + '/blame/' + sub + '@id=' + id + '.html';
  } else if (cleanPath.includes('/plain/')) {
    var sub = cleanPath.split('/plain/')[1];
    target = repoPrefix + '/plain/' + sub + '@id=' + id;
  } else if (cleanPath.includes('/diff/')) {
    var sub = cleanPath.split('/diff/')[1];
    target = repoPrefix + '/diff/' + sub + '@id=' + id + '.html';
  }
  
  if (target) {
    fetch(target)
      .then(function(r) {
        if (!r.ok) {
          if (target.includes('@id=')) {
            var fallback = target.split('@id=')[0] + '.html';
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
        document.body.innerHTML = "<div id='cgit'><div class='content'><div class='error'>404 - Page not found: " + path + (search || "") + "</div></div></div>";
      });
  } else {
    document.body.innerHTML = "<div id='cgit'><div class='content'><div class='error'>404 - Unknown route: " + path + "</div></div></div>";
  }
})();
</script>
</head>
<body>
<div id="cgit"><div class="content" style="padding: 2em; font-family: sans-serif;">Loading cgit route...</div></div>
</body>
</html>
"""

def main():
    print("Initializing git repository for cgit...")
    if not REPO_PATH.exists():
        REPO_PATH.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "config", "--global", "--add", "safe.directory", "*"], check=True)
        subprocess.run(["git", "clone", "--bare", "/repo", str(REPO_PATH)], check=True)
        (REPO_PATH / "description").write_text("git-on-github - A git repository wholly rendered by real cgit on GitHub Actions\n")
        subprocess.run(["git", "-C", str(REPO_PATH), "config", "gitweb.owner", "rebizzz"], check=True)
        
    if OUT_DIR.exists():
        shutil.rmtree(OUT_DIR)
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    
    print("Rendering root cgit pages...")
    summary_html = run_cgit("/git-on-github.git/")
    save_page("git-on-github.git/index.html", summary_html)
    save_page("index.html", summary_html)
    save_page("git-on-github.git/summary/index.html", summary_html)
    
    save_page("git-on-github.git/about/index.html", run_cgit("/git-on-github.git/about/"))
    save_page("git-on-github.git/refs/index.html", run_cgit("/git-on-github.git/refs/"))
    save_page("git-on-github.git/stats/index.html", run_cgit("/git-on-github.git/stats/"))
    
    atom_feed = run_cgit("/git-on-github.git/atom/")
    save_page("git-on-github.git/atom/index.html", atom_feed)
    save_page("git-on-github.git/atom/index.xml", atom_feed)
    save_page("atom.xml", atom_feed)
    
    log_index = run_cgit("/git-on-github.git/log/")
    save_page("git-on-github.git/log/index.html", log_index)
    save_page("git-on-github.git/tree/index.html", run_cgit("/git-on-github.git/tree/"))
    
    # Branches & Tags
    branches = run_git(["for-each-ref", "--format=%(refname:short)", "refs/heads"]).splitlines()
    tags = run_git(["for-each-ref", "--format=%(refname:short)", "refs/tags"]).splitlines()
    
    for b in branches:
        b = b.strip()
        if not b: continue
        print(f"Rendering branch: {b}")
        save_page(f"git-on-github.git/log/{b}.html", run_cgit("/git-on-github.git/log/", f"h={b}"))
        save_page(f"git-on-github.git/tree/{b}.html", run_cgit("/git-on-github.git/tree/", f"h={b}"))
        
    for t in tags:
        t = t.strip()
        if not t: continue
        print(f"Rendering tag: {t}")
        save_page(f"git-on-github.git/tag/{t}.html", run_cgit("/git-on-github.git/tag/", f"h={t}"))
        save_page(f"git-on-github.git/commit/{t}.html", run_cgit("/git-on-github.git/commit/", f"id={t}"))
    
    # Commits
    commits = run_git(["rev-list", "--all"]).splitlines()
    print(f"Rendering all {len(commits)} commits...")
    latest_commit = commits[0] if commits else "HEAD"
    
    for sha in commits:
        sha = sha.strip()
        if not sha: continue
        short_sha = sha[:7]
        
        # Commit
        c_html = run_cgit("/git-on-github.git/commit/", f"id={sha}")
        save_page(f"git-on-github.git/commit/{sha}.html", c_html)
        save_page(f"git-on-github.git/commit/{short_sha}.html", c_html)
        
        # Diff
        d_html = run_cgit("/git-on-github.git/diff/", f"id={sha}")
        save_page(f"git-on-github.git/diff/{sha}.html", d_html)
        save_page(f"git-on-github.git/diff/{short_sha}.html", d_html)
        
        # Patch
        p_text = run_cgit("/git-on-github.git/patch/", f"id={sha}")
        save_page(f"git-on-github.git/patch/{sha}.patch", p_text)
        save_page(f"git-on-github.git/patch/{short_sha}.patch", p_text)
        
        # Tree
        t_html = run_cgit("/git-on-github.git/tree/", f"id={sha}")
        save_page(f"git-on-github.git/tree/{sha}.html", t_html)
        save_page(f"git-on-github.git/tree/{short_sha}.html", t_html)
        
        # Parent diffs if any
        parents = run_git(["log", "-1", "--format=%P", sha]).split()
        for p in parents:
            p_sha = p.strip()
            if p_sha:
                diff_parent = run_cgit("/git-on-github.git/diff/", f"id={sha}&id2={p_sha}")
                save_page(f"git-on-github.git/diff/{sha}_{p_sha}.html", diff_parent)
                save_page(f"git-on-github.git/diff/{short_sha}_{p_sha[:7]}.html", diff_parent)

    # HEAD aliases
    save_page("git-on-github.git/commit/HEAD.html", run_cgit("/git-on-github.git/commit/", f"id={latest_commit}"))
    save_page("git-on-github.git/diff/HEAD.html", run_cgit("/git-on-github.git/diff/", f"id={latest_commit}"))
    save_page("git-on-github.git/patch/HEAD.patch", run_cgit("/git-on-github.git/patch/", f"id={latest_commit}"))
    save_page("git-on-github.git/tree/HEAD.html", run_cgit("/git-on-github.git/tree/", f"id={latest_commit}"))
    
    # Files, Blobs and Blames
    files = run_git(["ls-tree", "-r", "--name-only", "HEAD"]).splitlines()
    print(f"Rendering {len(files)} files...")
    
    recent_commits = commits[:15]
    for filepath in files:
        filepath = filepath.strip()
        if not filepath: continue
        
        # Default (HEAD)
        save_page(f"git-on-github.git/tree/{filepath}.html", run_cgit(f"/git-on-github.git/tree/{filepath}"))
        save_page(f"git-on-github.git/tree/{filepath}@id=HEAD.html", run_cgit(f"/git-on-github.git/tree/{filepath}", "id=HEAD"))
        save_page(f"git-on-github.git/blame/{filepath}.html", run_cgit(f"/git-on-github.git/blame/{filepath}"))
        save_page(f"git-on-github.git/blame/{filepath}@id=HEAD.html", run_cgit(f"/git-on-github.git/blame/{filepath}", "id=HEAD"))
        save_page(f"git-on-github.git/plain/{filepath}", run_cgit(f"/git-on-github.git/plain/{filepath}"))
        
        for sha in recent_commits:
            save_page(f"git-on-github.git/tree/{filepath}@id={sha}.html", run_cgit(f"/git-on-github.git/tree/{filepath}", f"id={sha}"))
            save_page(f"git-on-github.git/tree/{filepath}@id={sha[:7]}.html", run_cgit(f"/git-on-github.git/tree/{filepath}", f"id={sha}"))
            save_page(f"git-on-github.git/blame/{filepath}@id={sha}.html", run_cgit(f"/git-on-github.git/blame/{filepath}", f"id={sha}"))
            save_page(f"git-on-github.git/blame/{filepath}@id={sha[:7]}.html", run_cgit(f"/git-on-github.git/blame/{filepath}", f"id={sha}"))
            save_page(f"git-on-github.git/plain/{filepath}@id={sha}", run_cgit(f"/git-on-github.git/plain/{filepath}", f"id={sha}"))
            save_page(f"git-on-github.git/diff/{filepath}@id={sha}.html", run_cgit(f"/git-on-github.git/diff/{filepath}", f"id={sha}"))

    # Dynamic dispatchers
    print("Generating query parameter dispatchers...")
    save_page("git-on-github.git/commit/index.html", make_dispatcher("commit", latest_commit))
    save_page("git-on-github.git/diff/index.html", make_dispatcher("diff", latest_commit))
    save_page("git-on-github.git/patch/index.html", make_patch_dispatcher())
    save_page("git-on-github.git/tree/index.html", make_dispatcher("tree", "main"))
    save_page("git-on-github.git/log/index.html", make_dispatcher("log", "main"))
    
    # 404 router
    save_page("404.html", make_404_router())
    
    # Static cgit CSS / assets
    print("Writing assets...")
    cgit_share = Path("/usr/share/cgit")
    save_page("cgit-css/cgit.css", (cgit_share / "cgit.css").read_bytes())
    save_page("cgit-css/cgit.png", (cgit_share / "cgit.png").read_bytes())
    save_page("cgit-css/favicon.ico", (cgit_share / "favicon.ico").read_bytes())
    if (cgit_share / "cgit.js").exists():
        save_page("cgit.js", (cgit_share / "cgit.js").read_bytes())
    
    print("All pages rendered successfully!")

if __name__ == "__main__":
    main()
