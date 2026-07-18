use std::process::Command;

fn main() {
    // TODO: Capture git commit SHA for version info
    /*
     * println!("cargo:rerun-if-changed=../../.git/logs/HEAD");
     * if let Some(output) = Command::new("git")
     *     .args(["rev-parse", "--short", "HEAD"])
     *     .output()
     *     .ok()
     *     filter(|o| o.status.success())
     * {
     *     let git_sha = String::from_utf8_lossy(&output.stdout);
     *     println!("cargo:rustc-env=GIT_SHA={}", git_sha.trim());
     * } else {
     *     println!("cargo:rustc-env=GIT_SHA=unknown");
     * }
     */

    // Capture build timestamp
    println!(
        "cargo:rustc-env=BUILD_DATE={}",
        chrono::Utc::now().format("%Y-%m-%d")
    );

    // Platform-specific configuration
    #[cfg(target_os = "linux")]
    {
        //TODO: Set RPATH for custom library locations
    }

    #[cfg(target_os = "macos")]
    {
        // TODO: Set minimum macOS version
    }
}
