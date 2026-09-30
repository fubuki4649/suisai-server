use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let commit_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            std::env::var("GIT_HASH").unwrap_or_else(|_| "unknown".to_string())
        });

    println!("cargo:rustc-env=SUISAI_COMMIT_HASH={}", commit_hash);

    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(head_contents) = fs::read_to_string(".git/HEAD")
        && let Some(ref_path) = head_contents.strip_prefix("ref: ")
    {
        let ref_path = ref_path.trim();
        let full_ref_path = Path::new(".git").join(ref_path);
        println!("cargo:rerun-if-changed={}", full_ref_path.display());
    }
    println!("cargo:rerun-if-changed=.git/packed-refs");
}
