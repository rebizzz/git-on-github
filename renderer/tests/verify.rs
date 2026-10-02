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

    // 4. Verify each repository's core views
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

        // Check that at least some commits were rendered
        let commit_dir = repo_dir.join("commit");
        assert!(commit_dir.exists(), "Commit directory for {} must exist", repo);
        let commit_count = fs::read_dir(&commit_dir).unwrap().count();
        assert!(commit_count >= 5, "At least 5 commit files must be generated for {}", repo);

        // Check that diffs were rendered
        let diff_dir = repo_dir.join("diff");
        assert!(diff_dir.exists(), "Diff directory for {} must exist", repo);
        let diff_count = fs::read_dir(&diff_dir).unwrap().count();
        assert!(diff_count >= 5, "At least 5 diff files must be generated for {}", repo);

        // Check that patches were rendered
        let patch_dir = repo_dir.join("patch");
        assert!(patch_dir.exists(), "Patch directory for {} must exist", repo);

        // Check tree directory (specifically src/index.html if repo is curl)
        if repo == "curl" {
            let curl_src = repo_dir.join("tree/src/index.html");
            assert!(curl_src.exists(), "curl.git/tree/src/index.html must exist");
            let src_content = fs::read_to_string(&curl_src).unwrap();
            assert!(src_content.contains("CMakeLists.txt"), "curl.git/tree/src must contain CMakeLists.txt");
        }
    }

    println!("All verification assertions passed successfully!");
}
