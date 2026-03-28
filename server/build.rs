fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    let hash = {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .expect("failed to run git");
        assert!(
            output.status.success(),
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let hash = String::from_utf8(output.stdout)
            .expect("git output was not valid utf-8")
            .trim()
            .to_string();
        assert!(!hash.is_empty(), "git rev-parse returned empty output");
        hash
    };

    let dirty = {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .expect("failed to run git status");
        assert!(
            output.status.success(),
            "git status failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        !output.stdout.is_empty()
    };

    let suffix = if dirty { "-dirty" } else { "" };
    println!("cargo:rustc-env=GIT_HASH={hash}{suffix}");
}
