use duct::cmd;

#[test]
fn test_all() {
    let output = cmd![
        "cargo",
        "run",
        "--",
        "-i",
        "examples/version.json5",
        "incrPatch"
    ]
    .stdout_capture()
    .stderr_capture()
    .unchecked()
    .run()
    .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(output.status.success(), "stderr: {}", stderr,);

    assert!(stderr.contains("Operation"));
    assert!(stderr.contains("Would update"));
    assert!(stderr.contains("Would write"));
    assert!(stderr.contains("Would copy"));
}
