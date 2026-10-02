use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

struct Config {
    workspace: PathBuf,
    out_dir: PathBuf,
    git_base: PathBuf,
    cgitrc: PathBuf,
    cgit_bin: PathBuf,
    cgit_share: PathBuf,
    repos_conf: PathBuf,
}

impl Config {
    fn new() -> Self {
        let workspace = PathBuf::from(
            env::var("GITHUB_WORKSPACE")
                .unwrap_or_else(|_| {
                    if Path::new("/repo/.git").exists() {
                        "/repo".into()
                    } else {
                        env::current_dir().unwrap().to_string_lossy().into()
                    }
                }),
        );
        let out_dir = PathBuf::from(
            env::var("CGIT_OUT_DIR").unwrap_or_else(|_| workspace.join("gh-pages").to_string_lossy().into()),
        );
        let git_base = PathBuf::from(
            env::var("CGIT_REPOS_DIR").unwrap_or_else(|_| "/tmp/cgit-repos".into()),
        );
        let cgit_bin = PathBuf::from(
            env::var("CGIT_BIN").unwrap_or_else(|_| {
                for p in &["/usr/lib/cgit/cgit.cgi", "/usr/libexec/cgit/cgit.cgi"] {
                    if Path::new(p).exists() {
                        return p.to_string();
                    }
                }
                "cgit".into()
            }),
        );
        let cgit_share = PathBuf::from(
            env::var("CGIT_SHARE").unwrap_or_else(|_| {
                for p in &["/usr/share/cgit", "/usr/lib/cgit"] {
                    if Path::new(p).exists() {
                        return p.to_string();
                    }
                }
                "/usr/share/cgit".into()
            }),
        );
        let cgitrc = workspace.join("cgitrc");
        let repos_conf = workspace.join("repos.conf");
        Config { workspace, out_dir, git_base, cgitrc, cgit_bin, cgit_share, repos_conf }
    }
}

#[derive(Clone)]
struct Repo {
    name: String,
    url: String,
    desc: String,
    owner: String,
    depth: Option<u32>,
}

fn parse_repos_conf(path: &Path) -> Vec<Repo> {
    if !path.exists() {
        return vec![Repo {
            name: "git-on-github".into(),
            url: "local".into(),
            desc: "git-on-github - A git repository wholly rendered by real cgit on GitHub Actions".into(),
            owner: "rebizzz".into(),
            depth: None,
        }];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .filter_map(|line| {
            let parts: Vec<&str> = shell_split(line);
            if parts.len() < 2 { return None; }
            Some(Repo {
                name: parts[0].to_string(),
                url: parts[1].to_string(),
                desc: parts.get(2).copied().unwrap_or("Git repository").to_string(),
                owner: parts.get(3).copied().unwrap_or("Unknown").to_string(),
                depth: parts.get(4).and_then(|d| d.parse().ok()),
            })
        })
        .collect()
}

fn shell_split(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c.is_whitespace() { continue; }
        if c == '"' {
            let start = i + 1;
            let mut end = start;
            while let Some((j, ch)) = chars.next() {
                if ch == '"' { end = j; break; }
            }
            parts.push(&s[start..end]);
        } else {
            let start = i;
            let mut end = s.len();
            while let Some((j, ch)) = chars.peek().copied() {
                if ch.is_whitespace() { end = j; break; }
                chars.next();
            }
            parts.push(&s[start..end]);
        }
    }
    parts
}

fn run_git(repo_path: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("git failed"));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn run_cgit(cfg: &Config, path_info: &str, query_string: &str) -> Vec<u8> {
    let out = Command::new(&cfg.cgit_bin)
        .env("CGIT_CONFIG", &cfg.cgitrc)
        .env("PATH_INFO", path_info)
        .env("QUERY_STRING", query_string)
        .env("HTTP_HOST", "rebizzz.github.io")
        .env("SERVER_NAME", "rebizzz.github.io")
        .env("HTTPS", "on")
        .env("SCRIPT_NAME", "")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .expect("cgit.cgi failed");

    let body = &out.stdout;
    // Strip HTTP headers (find double newline)
    if let Some(pos) = body.windows(4).position(|w| w == b"\r\n\r\n") {
        return body[pos + 4..].to_vec();
    }
    if let Some(pos) = body.windows(2).position(|w| w == b"\n\n") {
        return body[pos + 2..].to_vec();
    }
    body.to_vec()
}

fn save_page(out_dir: &Path, rel_path: &str, content: &[u8]) {
    let dest = out_dir.join(rel_path);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&dest, content).ok();
}

fn save_str(out_dir: &Path, rel_path: &str, content: &str) {
    save_page(out_dir, rel_path, content.as_bytes());
}

fn make_dispatcher(repo_name: &str, kind: &str, default_target: &str) -> String {
    format!(r#"<!DOCTYPE html>
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
  <div class="content" style="padding:2em;font-family:monospace">Loading {kind}...</div>
</div>
<script>
(function() {{
  var p = new URLSearchParams(window.location.search);
  var id = p.get('id') || p.get('h') || '{default_target}';
  fetch(id + '.html')
    .then(function(r) {{ return r.ok ? r : fetch('HEAD.html'); }})
    .then(function(r) {{ return r.text(); }})
    .then(function(html) {{ document.open(); document.write(html); document.close(); }})
    .catch(function() {{ document.querySelector('.content').innerHTML = '<div class="error">{kind} not found: ' + id + '</div>'; }});
}})();
</script>
</body>
</html>
"#, kind=kind, repo_name=repo_name, default_target=default_target)
}

fn make_patch_dispatcher() -> &'static str {
    r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>patch</title></head>
<body><p>Loading patch...</p>
<script>(function(){var id=new URLSearchParams(location.search).get('id')||'HEAD';location.replace(id+'.patch');})();</script>
</body></html>
"#
}

fn make_404() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>404</title>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/cgit.css">
</head>
<body>
<div id="cgit">
  <table id="header"><tr><td class="main"><a href="/git-on-github/">index</a></td></tr></table>
  <div class="content" style="padding:2em;font-family:monospace">
    <div id="status-box">
      <div class="error">404 - Page not found</div>
      <p><a href="/git-on-github/">&larr; Return to repository index</a></p>
    </div>
  </div>
  <div class="footer">cgit on GitHub Actions</div>
</div>
<script>
(function() {
  function handleRoute() {
    var path = location.pathname.replace(/\/$/, "");
    var params = new URLSearchParams(location.search);
    var id = params.get('id') || params.get('h') || 'HEAD';
    var id2 = params.get('id2');
    var m = path.match(/\/git-on-github\/([^\/]+)\.git(\/.*)?$/);
    if (!m) { showError(); return; }
    var rp = "/git-on-github/" + m[1] + ".git";
    var sub = m[2] || "";
    var t = null;
    if (/\/commit$/.test(sub)) t = rp + "/commit/" + id + ".html";
    else if (/\/diff$/.test(sub)) t = rp + "/diff/" + (id2 ? id+"_"+id2 : id) + ".html";
    else if (/\/patch$/.test(sub)) t = rp + "/patch/" + id + ".patch";
    else if (/\/tree$/.test(sub)) t = rp + "/tree/" + id + ".html";
    else if (/\/log$/.test(sub)) t = rp + "/log/" + id + ".html";
    else if (sub.startsWith("/tree/")) t = rp + "/tree/" + sub.slice(6) + "@id=" + id + ".html";
    else if (sub.startsWith("/blame/")) t = rp + "/blame/" + sub.slice(7) + "@id=" + id + ".html";
    else if (sub.startsWith("/plain/")) t = rp + "/plain/" + sub.slice(7) + "@id=" + id;
    else if (sub.startsWith("/diff/")) t = rp + "/diff/" + sub.slice(6) + "@id=" + id + ".html";
    if (!t) { showError(); return; }
    fetch(t).then(function(r) {
      if (!r.ok && t.includes("@id=")) return fetch(t.split("@id=")[0]+".html");
      return r;
    }).then(function(r) { return r.text(); })
    .then(function(html) { document.open(); document.write(html); document.close(); })
    .catch(showError);
  }
  function showError() {
    document.title = "404";
    document.getElementById("status-box").innerHTML =
      "<div class='error'>404 - Not found: <code>" + location.pathname + location.search + "</code></div>" +
      "<p><a href='/git-on-github/'>&larr; index</a></p>";
  }
  document.readyState === 'loading'
    ? document.addEventListener('DOMContentLoaded', handleRoute)
    : handleRoute();
})();
</script>
</body>
</html>
"#
}

fn setup_repo(cfg: &Config, repo: &Repo) {
    let repo_dir = cfg.git_base.join(format!("{}.git", repo.name));
    if repo_dir.exists() {
        println!("Updating cached repo '{}'...", repo.name);
        let depth = repo.depth.unwrap_or(50).to_string();
        Command::new("git").arg("-C").arg(&repo_dir)
            .args(["fetch", "--depth", &depth, "--no-tags", "--prune"]).status().ok();
    } else {
        println!("Cloning repo '{}' from {}...", repo.name, repo.url);
        fs::create_dir_all(&repo_dir).ok();
        let mut args = vec!["clone", "--bare", "--no-single-branch", "--no-tags", "--filter=blob:none"];
        let depth_s;
        if let Some(d) = repo.depth {
            depth_s = d.to_string();
            args.extend_from_slice(&["--depth", &depth_s]);
        }
        args.push(&repo.url);
        let rd = repo_dir.to_string_lossy().to_string();
        args.push(&rd);
        Command::new("git").args(&args).status().ok();
    }
    // Write metadata
    fs::write(repo_dir.join("description"), format!("{}\n", repo.desc)).ok();
    Command::new("git").arg("-C").arg(&repo_dir).args(["config", "gitweb.owner", &repo.owner]).status().ok();
    Command::new("git").arg("-C").arg(&repo_dir).args(["config", "gitweb.clone-url", &repo.url]).status().ok();
}

struct Task {
    rel_path: String,
    path_info: String,
    query_string: String,
}

fn render_repo(cfg: &Config, repo: &Repo) {
    let repo_dir = cfg.git_base.join(format!("{}.git", repo.name));
    let prefix = format!("/{}.git", repo.name);
    let rel = format!("{}.git", repo.name);
    
    println!("\n--- Rendering '{}' ---", repo.name);

    // Collect branch info
    let branches_raw = run_git(&repo_dir, &["for-each-ref", "--sort=-committerdate", "--count=25", "--format=%(refname:short)", "refs/heads"]);
    let branches: Vec<&str> = branches_raw.lines().collect();
    let tags_raw = run_git(&repo_dir, &["for-each-ref", "--sort=-creatordate", "--count=20", "--format=%(refname:short)", "refs/tags"]);
    let tags: Vec<&str> = tags_raw.lines().collect();
    
    let default_branch = branches.first().copied().unwrap_or("main");
    
    // Get commits + parents in one shot
    let log_raw = run_git(&repo_dir, &["log", "-n80", "--all", "--format=%H %P"]);
    let mut commits: Vec<String> = Vec::new();
    let mut parent_map: HashMap<String, String> = HashMap::new();
    for line in log_raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }
        let sha = parts[0].to_string();
        if let Some(p) = parts.get(1) {
            parent_map.insert(sha.clone(), p.to_string());
        }
        commits.push(sha);
    }
    let latest_commit = commits.first().cloned().unwrap_or_else(|| "HEAD".to_string());

    // Files (top-level only)
    let files_raw = run_git(&repo_dir, &["ls-tree", "--name-only", "HEAD"]);
    let files: Vec<&str> = files_raw.lines().filter(|f| !f.is_empty()).take(30).collect();

    // Build task list
    let mut tasks: Vec<Task> = Vec::new();

    macro_rules! t {
        ($rel:expr, $pi:expr, $qs:expr) => {
            tasks.push(Task { rel_path: $rel.to_string(), path_info: $pi.to_string(), query_string: $qs.to_string() });
        };
    }

    // Base views
    t!(format!("{rel}/index.html"),          format!("{prefix}/"),        "");
    t!(format!("{rel}/summary/index.html"),  format!("{prefix}/"),        "");
    t!(format!("{rel}/about/index.html"),    format!("{prefix}/about/"),  "");
    t!(format!("{rel}/refs/index.html"),     format!("{prefix}/refs/"),   "");
    t!(format!("{rel}/stats/index.html"),    format!("{prefix}/stats/"),  "");
    t!(format!("{rel}/atom/index.html"),     format!("{prefix}/atom/"),   "");
    t!(format!("{rel}/atom/index.xml"),      format!("{prefix}/atom/"),   "");
    t!(format!("{rel}/log/index.html"),      format!("{prefix}/log/"),    "");
    t!(format!("{rel}/tree/index.html"),     format!("{prefix}/tree/"),   "");

    // Branches
    for b in &branches {
        let b = b.trim();
        t!(format!("{rel}/log/{b}.html"),  format!("{prefix}/log/"),  format!("h={b}"));
        t!(format!("{rel}/tree/{b}.html"), format!("{prefix}/tree/"), format!("h={b}"));
    }

    // Tags
    for t_name in &tags {
        let t_name = t_name.trim();
        t!(format!("{rel}/tag/{t_name}.html"),    format!("{prefix}/tag/"),    format!("h={t_name}"));
        t!(format!("{rel}/commit/{t_name}.html"), format!("{prefix}/commit/"), format!("id={t_name}"));
    }

    // Commits
    for sha in &commits {
        let sha = sha.trim();
        let s = &sha[..7.min(sha.len())];
        t!(format!("{rel}/commit/{sha}.html"),  format!("{prefix}/commit/"), format!("id={sha}"));
        t!(format!("{rel}/commit/{s}.html"),    format!("{prefix}/commit/"), format!("id={sha}"));
        t!(format!("{rel}/diff/{sha}.html"),    format!("{prefix}/diff/"),   format!("id={sha}"));
        t!(format!("{rel}/diff/{s}.html"),      format!("{prefix}/diff/"),   format!("id={sha}"));
        t!(format!("{rel}/patch/{sha}.patch"),  format!("{prefix}/patch/"),  format!("id={sha}"));
        t!(format!("{rel}/patch/{s}.patch"),    format!("{prefix}/patch/"),  format!("id={sha}"));
        t!(format!("{rel}/tree/{sha}.html"),    format!("{prefix}/tree/"),   format!("id={sha}"));
        t!(format!("{rel}/tree/{s}.html"),      format!("{prefix}/tree/"),   format!("id={sha}"));
        
        if let Some(p_sha) = parent_map.get(sha.trim()) {
            let ps = &p_sha[..7.min(p_sha.len())];
            t!(format!("{rel}/diff/{sha}_{p_sha}.html"), format!("{prefix}/diff/"), format!("id={sha}&id2={p_sha}"));
            t!(format!("{rel}/diff/{s}_{ps}.html"),      format!("{prefix}/diff/"), format!("id={sha}&id2={p_sha}"));
        }
    }

    // HEAD aliases
    t!(format!("{rel}/commit/HEAD.html"), format!("{prefix}/commit/"), format!("id={latest_commit}"));
    t!(format!("{rel}/diff/HEAD.html"),   format!("{prefix}/diff/"),   format!("id={latest_commit}"));
    t!(format!("{rel}/patch/HEAD.patch"), format!("{prefix}/patch/"),  format!("id={latest_commit}"));
    t!(format!("{rel}/tree/HEAD.html"),   format!("{prefix}/tree/"),   format!("id={latest_commit}"));

    // Files
    for f in &files {
        let f = f.trim();
        t!(format!("{rel}/tree/{f}.html"),       format!("{prefix}/tree/{f}"),  "");
        t!(format!("{rel}/tree/{f}@id=HEAD.html"),format!("{prefix}/tree/{f}"), "id=HEAD");
        t!(format!("{rel}/plain/{f}"),            format!("{prefix}/plain/{f}"), "");
        t!(format!("{rel}/blame/{f}.html"),        format!("{prefix}/blame/{f}"), "");
        t!(format!("{rel}/blame/{f}@id=HEAD.html"),format!("{prefix}/blame/{f}"), "id=HEAD");
    }

    println!("Rendering {} tasks in parallel...", tasks.len());

    // Parallel render via rayon
    let out_dir = &cfg.out_dir;
    let errors = Mutex::new(0u32);
    tasks.par_iter().for_each(|task| {
        let content = run_cgit(cfg, &task.path_info, &task.query_string);
        if content.is_empty() {
            *errors.lock().unwrap() += 1;
        }
        save_page(out_dir, &task.rel_path, &content);
    });
    let e = *errors.lock().unwrap();
    if e > 0 { println!("  {} tasks produced empty output", e); }

    // Dispatchers (sequential, fast)
    save_str(out_dir, &format!("{rel}/commit/index.html"), &make_dispatcher(&repo.name, "commit", &latest_commit));
    save_str(out_dir, &format!("{rel}/diff/index.html"),   &make_dispatcher(&repo.name, "diff",   &latest_commit));
    save_str(out_dir, &format!("{rel}/patch/index.html"),  make_patch_dispatcher());
    save_str(out_dir, &format!("{rel}/tree/index.html"),   &make_dispatcher(&repo.name, "tree", default_branch));
    save_str(out_dir, &format!("{rel}/log/index.html"),    &make_dispatcher(&repo.name, "log",  default_branch));

    println!("Done '{}'", repo.name);
}

fn copy_assets(cfg: &Config) {
    for file in &["cgit.css", "cgit.png", "favicon.ico"] {
        let src = cfg.cgit_share.join(file);
        if src.exists() {
            let dst_name = if *file == "cgit.css" || *file == "cgit.png" || *file == "favicon.ico" {
                format!("cgit-css/{}", file)
            } else {
                file.to_string()
            };
            let content = fs::read(&src).unwrap_or_default();
            save_page(&cfg.out_dir, &dst_name, &content);
        }
    }
    let js = cfg.cgit_share.join("cgit.js");
    if js.exists() {
        let content = fs::read(&js).unwrap_or_default();
        save_page(&cfg.out_dir, "cgit.js", &content);
    }
}

fn main() {
    let cfg = Config::new();

    println!("cgit binary:  {}", cfg.cgit_bin.display());
    println!("cgit share:   {}", cfg.cgit_share.display());
    println!("workspace:    {}", cfg.workspace.display());
    println!("output dir:   {}", cfg.out_dir.display());
    println!("repos dir:    {}", cfg.git_base.display());

    let repos = parse_repos_conf(&cfg.repos_conf);
    println!("Repos: {:?}", repos.iter().map(|r| r.name.as_str()).collect::<Vec<_>>());

    // Setup/update all repos
    Command::new("git").args(["config", "--global", "--add", "safe.directory", "*"]).status().ok();
    fs::create_dir_all(&cfg.git_base).ok();
    for r in &repos {
        setup_repo(&cfg, r);
    }

    // Wipe output and recreate
    if cfg.out_dir.exists() {
        fs::remove_dir_all(&cfg.out_dir).ok();
    }
    fs::create_dir_all(&cfg.out_dir).ok();

    // Render each repo
    for r in &repos {
        render_repo(&cfg, r);
    }

    // Site index
    println!("Rendering site index...");
    let index = run_cgit(&cfg, "/", "");
    save_page(&cfg.out_dir, "index.html", &index);

    // 404 router
    save_str(&cfg.out_dir, "404.html", make_404());

    // Static assets
    println!("Copying assets...");
    copy_assets(&cfg);

    println!("\nAll done!");
}
