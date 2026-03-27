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

#[test]
fn validate_command_runs() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("service-impact")
        .arg("--")
        .arg("validate")
        .arg("--registry")
        .arg("fixtures/sample/registry.json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("validate should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn validate_can_fail_on_warnings() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("service-impact")
        .arg("--")
        .arg("validate")
        .arg("--registry")
        .arg("fixtures/sample/registry.json")
        .arg("--fail-on-warnings")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("validate should run");
    assert!(!output.status.success(), "process should fail on warnings");
}
