use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

#[allow(dead_code)]
struct Config {
    workspace: PathBuf,
    out_dir: PathBuf,
    git_base: PathBuf,
    cgitrc: PathBuf,
    cgit_bin: PathBuf,
    cgit_share: PathBuf,
    repos_conf: PathBuf,
    theme_dir: PathBuf,
}

impl Config {
    fn from_args(args: &[String]) -> Self {
        let workspace = PathBuf::from(
            env::var("GITHUB_WORKSPACE")
                .unwrap_or_else(|_| {
                    if Path::new("/repo/.git").exists() {
                        "/repo".into()
                    } else if Path::new("../cgitrc").exists() {
                        Path::new("..").canonicalize().unwrap().to_string_lossy().into()
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
                for p in &[
                    "/usr/lib/cgit/cgit.cgi",
                    "/usr/libexec/cgit/cgit.cgi",
                    "/run/current-system/sw/cgit/cgit.cgi",
                ] {
                    if Path::new(p).exists() {
                        return p.to_string();
                    }
                }
                if let Ok(path_var) = env::var("PATH") {
                    for dir in env::split_paths(&path_var) {
                        for candidate in &[
                            dir.join("cgit.cgi"),
                            dir.join("cgit"),
                            dir.parent().map(|p| p.join("cgit/cgit.cgi")).unwrap_or_default(),
                        ] {
                            if candidate.is_file() {
                                return candidate.to_string_lossy().to_string();
                            }
                        }
                    }
                }
                "cgit".into()
            }),
        );
        let cgit_share = PathBuf::from(
            env::var("CGIT_SHARE").unwrap_or_else(|_| {
                if let Some(parent) = cgit_bin.parent() {
                    if parent.join("cgit.css").exists() {
                        return parent.to_string_lossy().to_string();
                    }
                }
                let local_assets = workspace.join("assets");
                if local_assets.join("cgit.css").exists() {
                    return local_assets.to_string_lossy().to_string();
                }
                for p in &[
                    "/usr/share/cgit",
                    "/usr/lib/cgit",
                    "/nix/var/nix/profiles/default/share/cgit",
                ] {
                    if Path::new(p).join("cgit.css").exists() {
                        return p.to_string();
                    }
                }
                "/usr/share/cgit".into()
            }),
        );

        let theme_arg = args.windows(2).find(|w| w[0] == "--theme").map(|w| w[1].clone());
        let theme_dir = theme_arg
            .or_else(|| env::var("CGIT_THEME").ok())
            .map(|t| {
                let p = PathBuf::from(&t);
                if p.is_absolute() && p.exists() {
                    p
                } else if workspace.join(&t).exists() {
                    workspace.join(&t)
                } else if workspace.join("themes").join(&t).exists() {
                    workspace.join("themes").join(&t)
                } else {
                    p
                }
            })
            .unwrap_or_else(|| {
                if workspace.join("themes/cgithub").exists() {
                    workspace.join("themes/cgithub")
                } else if workspace.join("theme").exists() {
                    workspace.join("theme")
                } else {
                    workspace.join("themes/cgithub")
                }
            });

        let cgitrc_arg = args.windows(2).find(|w| w[0] == "--cgitrc").map(|w| w[1].clone());
        let base_cgitrc_path = cgitrc_arg
            .or_else(|| env::var("CGITRC").ok())
            .map(|c| {
                let p = PathBuf::from(&c);
                if p.is_absolute() && p.exists() {
                    p
                } else if workspace.join(&c).exists() {
                    workspace.join(&c)
                } else {
                    p
                }
            })
            .unwrap_or_else(|| {
                if theme_dir.join("cgitrc").exists() {
                    theme_dir.join("cgitrc")
                } else {
                    workspace.join("cgitrc")
                }
            });

        let base_cgitrc = fs::read_to_string(&base_cgitrc_path).unwrap_or_default();
        let filter_bin = [
            workspace.join("bin/cgit-about-filter"),
            workspace.join("renderer/target/release/cgit-about-filter"),
            workspace.join("renderer/target/debug/cgit-about-filter"),
            workspace.join("target/release/cgit-about-filter"),
            workspace.join("target/debug/cgit-about-filter"),
            workspace.join("filters/about-formatting.sh"),
        ]
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| workspace.join("filters/about-formatting.sh"));

        let head_include_path = if theme_dir.join("head-include.html").exists() {
            Some(theme_dir.join("head-include.html"))
        } else if workspace.join("head-include.html").exists() {
            Some(workspace.join("head-include.html"))
        } else {
            None
        };

        let cgitrc_path = workspace.join(".cgitrc.runtime");
        let filtered_cgitrc = base_cgitrc
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.starts_with("about-filter=")
                    && !t.starts_with("head-include=")
                    && !t.starts_with("scan-path=")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut runtime_cgitrc = filtered_cgitrc;
        runtime_cgitrc.push_str(&format!("\nabout-filter={}\n", filter_bin.display()));
        if let Some(ref hi) = head_include_path {
            runtime_cgitrc.push_str(&format!("head-include={}\n", hi.display()));
        }
        runtime_cgitrc.push_str(&format!("scan-path={}\n", git_base.display()));

        fs::write(&cgitrc_path, runtime_cgitrc).ok();
        let cgitrc = cgitrc_path;
        let repos_conf = workspace.join("repos.conf");
        Config { workspace, out_dir, git_base, cgitrc, cgit_bin, cgit_share, repos_conf, theme_dir }
    }
}

#[derive(Clone, Debug)]
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
            name: "cgit".into(),
            url: "https://git.zx2c4.com/cgit".into(),
            desc: "A hyperfast web frontend for git repositories written in C".into(),
            owner: "Jason A. Donenfeld".into(),
            depth: Some(50),
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
        .args(["-c", "safe.directory=*"])
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("git failed"));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn run_cgit(cfg: &Config, path_info: &str, query_string: &str) -> Vec<u8> {
    let out = Command::new(&cfg.cgit_bin)
        .current_dir(&cfg.workspace)
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
        if parent.is_file() {
            fs::remove_file(parent).ok();
        }
        let mut cur = out_dir.to_path_buf();
        if let Ok(rel) = parent.strip_prefix(out_dir) {
            for comp in rel.components() {
                cur.push(comp);
                if cur.is_file() {
                    fs::remove_file(&cur).ok();
                }
            }
        }
        fs::create_dir_all(parent).ok();
    }
    if dest.is_dir() {
        fs::remove_dir_all(&dest).ok();
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
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>cgit {kind}</title>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/cgit.css">
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/github.min.css"/>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/github-dark.min.css"/>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/theme.css">
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
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>404</title>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/cgit.css">
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/github.min.css"/>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/github-dark.min.css"/>
<link rel="stylesheet" type="text/css" href="/git-on-github/cgit-css/theme.css">
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
    if (!m) return;
    var rp = "/git-on-github/" + m[1] + ".git";
    var sub = m[2] || "";
    var cands = [];
    if (/\/commit$/.test(sub)) {
      cands.push(rp + "/commit/" + id + ".html");
    } else if (/\/diff$/.test(sub)) {
      if (id2) cands.push(rp + "/diff/" + id + "_" + id2 + ".html");
      cands.push(rp + "/diff/" + id + ".html");
    } else if (/\/patch$/.test(sub)) {
      cands.push(rp + "/patch/" + id + ".patch");
    } else if (/\/tree$/.test(sub)) {
      cands.push(rp + "/tree/" + id + ".html");
      cands.push(rp + "/tree/index.html");
    } else if (/\/log$/.test(sub)) {
      cands.push(rp + "/log/" + id + ".html");
      cands.push(rp + "/log/index.html");
    } else if (sub.startsWith("/tree/")) {
      var item = sub.slice(6);
      cands.push(rp + "/tree/" + item + "/index.html");
      cands.push(rp + "/tree/" + item + ".html");
      cands.push(rp + "/tree/" + item + "@id=" + id + ".html");
    } else if (sub.startsWith("/blame/")) {
      var item = sub.slice(7);
      cands.push(rp + "/blame/" + item + ".html");
      cands.push(rp + "/blame/" + item + "@id=" + id + ".html");
    } else if (sub.startsWith("/plain/")) {
      var item = sub.slice(7);
      cands.push(rp + "/plain/" + item);
      cands.push(rp + "/plain/" + item + "@id=" + id);
    } else if (sub.startsWith("/diff/")) {
      var item = sub.slice(6);
      cands.push(rp + "/diff/" + item + "@id=" + id + ".html");
      cands.push(rp + "/diff/" + item + ".html");
    }
    
    function tryFetch(i) {
      if (i >= cands.length) return;
      fetch(cands[i]).then(function(r) {
        if (!r.ok) { tryFetch(i + 1); return; }
        return r.text().then(function(html) {
          document.open();
          document.write(html);
          document.close();
        });
      }).catch(function() {
        tryFetch(i + 1);
      });
    }
    if (cands.length > 0) tryFetch(0);
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
    let has_commits = Command::new("git")
        .args(["-c", "safe.directory=*"])
        .arg("-C")
        .arg(&repo_dir)
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_commits {
        println!("Updating cached repo '{}'...", repo.name);
        let depth = repo.depth.unwrap_or(30).to_string();
        Command::new("git").args(["-c", "safe.directory=*"]).arg("-C").arg(&repo_dir)
            .args(["fetch", "--depth", &depth, "--no-tags", "--prune"]).status().ok();
    } else {
        println!("Cloning repo '{}' from {}...", repo.name, repo.url);
        fs::remove_dir_all(&repo_dir).ok();
        fs::create_dir_all(&repo_dir).ok();
        let depth_s = repo.depth.unwrap_or(30).to_string();
        let repo_dir_str = repo_dir.to_string_lossy().to_string();
        let args = vec![
            "-c",
            "safe.directory=*",
            "clone",
            "--bare",
            "--no-single-branch",
            "--no-tags",
            "--depth",
            &depth_s,
            &repo.url,
            &repo_dir_str,
        ];
        Command::new("git").args(&args).status().ok();
    }
    fs::write(repo_dir.join("description"), format!("{}\n", repo.desc)).ok();
    Command::new("git").args(["-c", "safe.directory=*"]).arg("-C").arg(&repo_dir).args(["config", "gitweb.owner", &repo.owner]).status().ok();
    Command::new("git").args(["-c", "safe.directory=*"]).arg("-C").arg(&repo_dir).args(["config", "gitweb.clone-url", &repo.url]).status().ok();
}

struct Task {
    rel_paths: Vec<String>,
    path_info: String,
    query_string: String,
}

fn render_repo(cfg: &Config, repo: &Repo) {
    let repo_dir = cfg.git_base.join(format!("{}.git", repo.name));
    let prefix = format!("/{}.git", repo.name);
    let rel = format!("{}.git", repo.name);
    
    println!("\n--- Rendering '{}' ---", repo.name);

    let branches_raw = run_git(&repo_dir, &["for-each-ref", "--sort=-committerdate", "--count=25", "--format=%(refname:short)", "refs/heads"]);
    let branches: Vec<&str> = branches_raw.lines().collect();
    let tags_raw = run_git(&repo_dir, &["for-each-ref", "--sort=-creatordate", "--count=20", "--format=%(refname:short)", "refs/tags"]);
    let tags: Vec<&str> = tags_raw.lines().collect();
    
    let default_branch = branches.first().copied().unwrap_or("master");
    
    let log_raw = run_git(&repo_dir, &["log", "-n30", "--all", "--format=%H %P"]);
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

    let stamp_file = cfg.out_dir.join(&rel).join(".rendered-commit");
    if stamp_file.exists() && cfg.out_dir.join(&rel).join("tree/index.html").exists() {
        if let Ok(stamped) = fs::read_to_string(&stamp_file) {
            if stamped.trim() == latest_commit.trim() {
                println!("Repo '{}' already rendered at commit {}, skipping rendering!", repo.name, latest_commit);
                return;
            }
        }
    }

    let mut tasks: Vec<Task> = Vec::new();

    macro_rules! t {
        ($paths:expr, $pi:expr, $qs:expr) => {
            tasks.push(Task {
                rel_paths: $paths,
                path_info: $pi.to_string(),
                query_string: $qs.to_string(),
            });
        };
    }

    t!(vec![format!("{rel}/index.html"), format!("{rel}/summary/index.html")], format!("{prefix}/"), "");
    t!(vec![format!("{rel}/about/index.html")],   format!("{prefix}/about/"), "");
    t!(vec![format!("{rel}/refs/index.html")],    format!("{prefix}/refs/"), "");
    t!(vec![format!("{rel}/stats/index.html")],   format!("{prefix}/stats/"), "");
    t!(vec![format!("{rel}/atom/index.html"), format!("{rel}/atom/index.xml")], format!("{prefix}/atom/"), "");
    t!(vec![format!("{rel}/log/index.html")],     format!("{prefix}/log/"), "");
    t!(vec![format!("{rel}/tree/index.html")],    format!("{prefix}/tree/"), "");

    for b in &branches {
        let b = b.trim();
        t!(vec![format!("{rel}/log/{b}.html")],  format!("{prefix}/log/"),  format!("h={b}"));
        t!(vec![format!("{rel}/tree/{b}.html")], format!("{prefix}/tree/"), format!("h={b}"));
    }

    for t_name in &tags {
        let t_name = t_name.trim();
        t!(vec![format!("{rel}/tag/{t_name}.html")],    format!("{prefix}/tag/"),    format!("h={t_name}"));
        t!(vec![format!("{rel}/commit/{t_name}.html")], format!("{prefix}/commit/"), format!("id={t_name}"));
    }

    for sha in &commits {
        let sha = sha.trim();
        let s = &sha[..7.min(sha.len())];
        t!(vec![format!("{rel}/commit/{sha}.html"), format!("{rel}/commit/{s}.html")], format!("{prefix}/commit/"), format!("id={sha}"));
        t!(vec![format!("{rel}/diff/{sha}.html"), format!("{rel}/diff/{s}.html")],     format!("{prefix}/diff/"),   format!("id={sha}"));
        t!(vec![format!("{rel}/patch/{sha}.patch"), format!("{rel}/patch/{s}.patch")], format!("{prefix}/patch/"),  format!("id={sha}"));
        t!(vec![format!("{rel}/tree/{sha}.html"), format!("{rel}/tree/{s}.html")],     format!("{prefix}/tree/"),   format!("id={sha}"));
        
        if let Some(p_sha) = parent_map.get(sha.trim()) {
            let ps = &p_sha[..7.min(p_sha.len())];
            t!(vec![format!("{rel}/diff/{sha}_{p_sha}.html"), format!("{rel}/diff/{s}_{ps}.html")], format!("{prefix}/diff/"), format!("id={sha}&id2={p_sha}"));
        }
    }

    t!(vec![format!("{rel}/commit/HEAD.html")], format!("{prefix}/commit/"), format!("id={latest_commit}"));
    t!(vec![format!("{rel}/diff/HEAD.html")],   format!("{prefix}/diff/"),   format!("id={latest_commit}"));
    t!(vec![format!("{rel}/patch/HEAD.patch")], format!("{prefix}/patch/"),  format!("id={latest_commit}"));
    t!(vec![format!("{rel}/tree/HEAD.html")],   format!("{prefix}/tree/"),   format!("id={latest_commit}"));

    // Full recursive tree exploration
    let tree_raw = run_git(&repo_dir, &["ls-tree", "-r", "-t", "HEAD"]);
    for line in tree_raw.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() < 2 { continue; }
        let meta: Vec<&str> = parts[0].split_whitespace().collect();
        if meta.len() < 3 { continue; }
        let obj_type = meta[1];
        let path = parts[1].trim();
        if path.is_empty() { continue; }

        if obj_type == "tree" {
            // Directory tree view
            t!(
                vec![
                    format!("{rel}/tree/{path}/index.html"),
                    format!("{rel}/tree/{path}.html"),
                    format!("{rel}/tree/{path}@id=HEAD.html"),
                ],
                format!("{prefix}/tree/{path}"),
                ""
            );
        } else if obj_type == "blob" {
            // File blob view
            t!(
                vec![
                    format!("{rel}/tree/{path}/index.html"),
                    format!("{rel}/tree/{path}.html"),
                    format!("{rel}/tree/{path}@id=HEAD.html"),
                ],
                format!("{prefix}/tree/{path}"),
                ""
            );
            // Plain text view
            t!(
                vec![format!("{rel}/plain/{path}")],
                format!("{prefix}/plain/{path}"),
                ""
            );
            // Blame view
            t!(
                vec![
                    format!("{rel}/blame/{path}/index.html"),
                    format!("{rel}/blame/{path}.html"),
                    format!("{rel}/blame/{path}@id=HEAD.html"),
                ],
                format!("{prefix}/blame/{path}"),
                ""
            );
        }
    }

    println!("Rendering {} tasks in parallel for '{}'...", tasks.len(), repo.name);

    let out_dir = &cfg.out_dir;
    let errors = Mutex::new(0u32);
    tasks.par_iter().for_each(|task| {
        let content = run_cgit(cfg, &task.path_info, &task.query_string);
        if content.is_empty() {
            *errors.lock().unwrap() += 1;
        }
        for rel_path in &task.rel_paths {
            save_page(out_dir, rel_path, &content);
        }
    });
    let e = *errors.lock().unwrap();
    if e > 0 { println!("  {} tasks produced empty output", e); }

    save_str(out_dir, &format!("{rel}/commit/index.html"), &make_dispatcher(&repo.name, "commit", &latest_commit));
    save_str(out_dir, &format!("{rel}/diff/index.html"),   &make_dispatcher(&repo.name, "diff",   &latest_commit));
    save_str(out_dir, &format!("{rel}/patch/index.html"),  make_patch_dispatcher());
    save_str(out_dir, &format!("{rel}/tree/index.html"),   &make_dispatcher(&repo.name, "tree", default_branch));
    save_str(out_dir, &format!("{rel}/log/index.html"),    &make_dispatcher(&repo.name, "log",  default_branch));
    save_str(out_dir, &format!("{rel}/.rendered-commit"),  &latest_commit);

    println!("Done '{}'", repo.name);
}

fn copy_assets(cfg: &Config) {
    for file in &["cgit.css", "cgit.png", "favicon.ico", "curl-logo.svg"] {
        let src = cfg.cgit_share.join(file);
        let local_src = cfg.workspace.join("assets").join(file);
        let actual_src = if src.exists() {
            src
        } else if local_src.exists() {
            local_src
        } else {
            continue;
        };
        let dst_name = if *file == "cgit.css" || *file == "cgit.png" || *file == "favicon.ico" || *file == "curl-logo.svg" {
            format!("cgit-css/{}", file)
        } else {
            file.to_string()
        };
        let content = fs::read(&actual_src).unwrap_or_default();
        save_page(&cfg.out_dir, &dst_name, &content);
    }

    if cfg.theme_dir.exists() {
        if let Ok(entries) = fs::read_dir(&cfg.theme_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let fname = path.file_name().unwrap().to_string_lossy();
                    if fname.ends_with(".css") || fname.ends_with(".js") || fname.ends_with(".png") || fname.ends_with(".svg") || fname.ends_with(".ico") || fname.ends_with(".html") {
                        let content = fs::read(&path).unwrap_or_default();
                        save_page(&cfg.out_dir, &format!("cgit-css/{}", fname), &content);
                    }
                }
            }
        }
    }

    let js = cfg.cgit_share.join("cgit.js");
    let local_js = cfg.workspace.join("assets").join("cgit.js");
    let actual_js = if js.exists() { js } else { local_js };
    if actual_js.exists() {
        let content = fs::read(&actual_js).unwrap_or_default();
        save_page(&cfg.out_dir, "cgit.js", &content);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cfg = Config::from_args(&args);
    
    let repo_filter = args.windows(2).find(|w| w[0] == "--repo").map(|w| w[1].clone());
    let base_only = args.iter().any(|a| a == "--base");

    println!("cgit binary:  {}", cfg.cgit_bin.display());
    println!("cgit share:   {}", cfg.cgit_share.display());
    println!("output dir:   {}", cfg.out_dir.display());
    println!("repos dir:    {}", cfg.git_base.display());

    let all_repos = parse_repos_conf(&cfg.repos_conf);
    fs::create_dir_all(&cfg.git_base).ok();

    if base_only {
        println!("Rendering site base (index, 404, assets)...");
        for r in &all_repos {
            let r_dir = cfg.git_base.join(format!("{}.git", r.name));
            if !r_dir.exists() {
                fs::create_dir_all(&r_dir).ok();
                Command::new("git").args(["-c", "safe.directory=*"]).arg("-C").arg(&r_dir).args(["init", "--bare"]).status().ok();
            }
            fs::write(r_dir.join("description"), format!("{}\n", r.desc)).ok();
            Command::new("git").args(["-c", "safe.directory=*"]).arg("-C").arg(&r_dir).args(["config", "gitweb.owner", &r.owner]).status().ok();
        }
        
        let index = run_cgit(&cfg, "/", "");
        save_page(&cfg.out_dir, "index.html", &index);
        save_str(&cfg.out_dir, "404.html", make_404());
        copy_assets(&cfg);
        println!("Site base complete!");
        return;
    }

    let repos_to_render: Vec<Repo> = if let Some(ref target) = repo_filter {
        all_repos.into_iter().filter(|r| r.name == *target).collect()
    } else {
        all_repos
    };

    println!("Target repos: {:?}", repos_to_render.iter().map(|r| r.name.as_str()).collect::<Vec<_>>());

    for r in &repos_to_render {
        setup_repo(&cfg, r);
        render_repo(&cfg, r);
    }

    if repo_filter.is_none() {
        println!("Rendering site index...");
        let index = run_cgit(&cfg, "/", "");
        save_page(&cfg.out_dir, "index.html", &index);
        save_str(&cfg.out_dir, "404.html", make_404());
        copy_assets(&cfg);
    }

    println!("\nExecution completed successfully!");
}
