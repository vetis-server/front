use assert_cmd::Command;

#[test]
fn test_no_args() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("front").unwrap();
    cmd.assert()
        .success();
}
