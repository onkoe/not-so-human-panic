#[test]
#[cfg_attr(debug_assertions, ignore)]
fn release() {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("single-panic-test"))
    .assert()
    .stderr_matches(
      "\
Well, this is embarrassing.

single-panic-test had a problem and crashed. It seems that the problem has to do with the following:
OMG EVERYTHING IS ON FIRE!!!

If you'd like, you can help us diagnose the problem! Please feel free to send us a crash report using the instructions below.

We have generated a report file at \"[..].toml\". Submit an issue or email with the subject of \"single-panic-test Crash Report\" and include the report as an attachment.

- Authors: Human Panic Authors <human-panic-crate@example.com>

We take privacy very seriously - we don't perform any automated error collection. In order to improve the software, we rely on users like you to submit reports.

Thank you kindly!
",
    )
    .code(101);
}

#[test]
#[cfg_attr(not(debug_assertions), ignore)]
fn debug() {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("single-panic-test"))
        .assert()
        .stderr_matches(
            "\
thread 'main' panicked at 'OMG EVERYTHING IS ON FIRE!!!', tests/single-panic/src/main.rs:7:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
",
        )
        .code(101);
}
