use std::process::{Command, Stdio};

#[test]
fn example_runs() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--example")
        .arg("basic")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("example should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
