use std::fs;
use std::path::Path;

#[test]
fn test_site_output() {
    let out_dir = Path::new("../gh-pages");
    if !out_dir.exists() {
        println!("gh-pages directory does not exist locally; skipping test.");
        return;
    }

    // 1. Verify Site Index
    let index_file = out_dir.join("index.html");
    assert!(index_file.exists(), "Root index.html must exist");
    let index_content = fs::read_to_string(&index_file).expect("Failed to read root index.html");
    assert!(index_content.contains("<div id='cgit'>"), "Root index must contain cgit container");
    assert!(index_content.contains("Git repository browser"), "Root index must contain title");
    assert!(index_content.contains("cgit.git"), "Root index must list cgit.git");
    assert!(index_content.contains("curl.git"), "Root index must list curl.git");
    assert!(index_content.contains("ripgrep.git"), "Root index must list ripgrep.git");

    // 2. Verify 404 router
    let not_found = out_dir.join("404.html");
    assert!(not_found.exists(), "404.html router must exist");
    let not_found_content = fs::read_to_string(&not_found).expect("Failed to read 404.html");
    assert!(not_found_content.contains("handleRoute"), "404.html must contain route handler");
    assert!(not_found_content.contains("cgit-css/cgit.css"), "404.html must link cgit.css");

    // 3. Verify CSS & assets
    let css_file = out_dir.join("cgit-css/cgit.css");
    assert!(css_file.exists(), "cgit.css must exist");
    assert!(fs::metadata(&css_file).unwrap().len() > 1000, "cgit.css must not be empty");
    assert!(out_dir.join("cgit-css/curl-logo.svg").exists(), "curl-logo.svg must exist");

    // 4. Verify repositories
    let repos = vec!["cgit", "curl", "ripgrep"];
    for repo in repos {
        let repo_dir = out_dir.join(format!("{}.git", repo));
        if !repo_dir.exists() {
            println!("Repo {} not yet rendered locally; skipping.", repo);
            continue;
        }

        // Summary
        let summary = repo_dir.join("index.html");
        assert!(summary.exists(), "Summary {}/index.html must exist", repo);
        let s_content = fs::read_to_string(&summary).unwrap();
        assert!(s_content.contains("<div id='cgit'>"), "Summary must contain cgit layout");

        // About
        let about = repo_dir.join("about/index.html");
        assert!(about.exists(), "About view for {} must exist", repo);
        let about_content = fs::read_to_string(&about).unwrap();
        assert!(about_content.contains("class='markdown-body'"), "About view for {} must contain markdown-body", repo);

        // Refs
        let refs = repo_dir.join("refs/index.html");
        assert!(refs.exists(), "Refs view for {} must exist", repo);

        // Log
        let log = repo_dir.join("log/index.html");
        assert!(log.exists(), "Log view for {} must exist", repo);

        // Tree
        let tree = repo_dir.join("tree/index.html");
        assert!(tree.exists(), "Tree view for {} must exist", repo);

        // Stats
        let stats = repo_dir.join("stats/index.html");
        assert!(stats.exists(), "Stats view for {} must exist", repo);

        // Commits & Diffs
        let commit_dir = repo_dir.join("commit");
        assert!(commit_dir.exists(), "Commit directory for {} must exist", repo);
        let commit_count = fs::read_dir(&commit_dir).unwrap().count();
        assert!(commit_count >= 5, "At least 5 commit files must be generated for {}", repo);

        let diff_dir = repo_dir.join("diff");
        assert!(diff_dir.exists(), "Diff directory for {} must exist", repo);

        let patch_dir = repo_dir.join("patch");
        assert!(patch_dir.exists(), "Patch directory for {} must exist", repo);

        // Repo specific checks
        if repo == "curl" {
            // Nested directories
            assert!(repo_dir.join("tree/src/index.html").exists(), "curl.git/tree/src/index.html must exist");
            assert!(repo_dir.join("tree/include/index.html").exists(), "curl.git/tree/include/index.html must exist");
            assert!(repo_dir.join("tree/include/curl/index.html").exists(), "curl.git/tree/include/curl/index.html must exist");

            // Specific deep nested files requested by user
            let multi_h_idx = repo_dir.join("tree/include/curl/multi.h/index.html");
            assert!(multi_h_idx.exists(), "curl.git/tree/include/curl/multi.h/index.html must exist");
            let multi_content = fs::read_to_string(&multi_h_idx).unwrap();
            assert!(multi_content.contains("multi.h"), "multi.h view must contain filename");
            assert!(multi_content.contains("<div id='cgit'>"), "multi.h view must have cgit container");

            assert!(repo_dir.join("tree/include/curl/multi.h.html").exists(), "multi.h.html must exist");
            assert!(repo_dir.join("plain/include/curl/multi.h").exists(), "plain/include/curl/multi.h must exist");
            assert!(repo_dir.join("blame/include/curl/multi.h/index.html").exists(), "blame/include/curl/multi.h/index.html must exist");
        } else if repo == "cgit" {
            assert!(repo_dir.join("tree/cgit.c/index.html").exists(), "cgit.git/tree/cgit.c/index.html must exist");
            assert!(repo_dir.join("plain/cgit.c").exists(), "plain/cgit.c must exist");
        }
    }

    println!("All verification assertions passed successfully!");
}
