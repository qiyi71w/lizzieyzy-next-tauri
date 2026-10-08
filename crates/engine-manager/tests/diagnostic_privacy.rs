use engine_manager::diagnostics::DiagnosticSanitizer;

#[test]
fn private_values_have_stable_aliases_without_merging_record_boundaries() {
    let mut sanitizer = DiagnosticSanitizer::default();
    let first = sanitizer.sanitize("WARN roomId=424242\nthreadId=42 running\npassword=\"two words\nsecond secret line\"\nINFO recovered");
    assert_eq!(first.lines().count(), 5);
    for private in ["424242", "two words", "second secret line"] { assert!(!first.contains(private)); }
    assert!(first.contains("threadId=42 running"));
    assert!(first.ends_with("INFO recovered"));
    let second = sanitizer.sanitize("INFO roomId=424242");
    assert_eq!(first.lines().next().unwrap().split('=').next_back(), second.split('=').next_back());
    let paths = sanitizer.sanitize("WARN https://host.invalid/a?token=FAKE /home/alice/model.bin\nC:\\Users\\alice\\config.cfg\nINFO \u{1b}[31mready\u{0}");
    for private in ["host.invalid", "FAKE", "alice", "\u{1b}", "\u{0}"] { assert!(!paths.contains(private)); }
    assert_eq!(paths.lines().count(), 3);
    assert_eq!(sanitizer.sanitize(&"x".repeat(1024 * 1024)), "[overlong record omitted]");
}

#[test]
fn escaped_quote_does_not_release_a_multiline_secret() {
    let mut sanitizer = DiagnosticSanitizer::default();
    let text = sanitizer.sanitize("password=\"first \\\" quoted\nFAKE-next-line\"\nINFO finished");
    assert!(!text.contains("FAKE-next-line"));
    assert!(text.ends_with("INFO finished"));
}

#[test]
fn quoted_paths_and_known_argv_values_never_leak_space_delimited_fragments() {
    let mut sanitizer = DiagnosticSanitizer::default();
    let alias = sanitizer.alias("FAKE argument value");
    assert_eq!(sanitizer.sanitize("echo FAKE argument value done"), format!("echo {alias} done"));
    let result = sanitizer.sanitize("WARN config \"C:\\Users\\Fake User\\file.cfg\" absent\nWARN \"/home/Fake User/config\" absent");
    assert_eq!(result.lines().count(), 2);
    for secret in ["Fake", "User", "file.cfg", "config"] { assert!(!result.contains(secret)); }
}
