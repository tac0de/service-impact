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
    use std::io::Write;

    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("service-impact")
        .arg("--")
        .arg("validate")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("validate should spawn");

    child
        .stdin
        .as_mut()
        .expect("stdin should be available")
        .write_all(br#"{"registry_path":"fixtures/sample/registry.json"}"#)
        .expect("stdin write should succeed");

    let output = child.wait_with_output().expect("process should complete");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
