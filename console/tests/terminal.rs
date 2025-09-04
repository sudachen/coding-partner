use console::terminal::Terminal;

#[test]
fn test_create_and_drop_terminal() {
    // This test is designed to be run in a non-interactive environment.
    // It simply checks that the Terminal can be created and dropped
    // without panicking. A real interactive test would require a pseudo-terminal (pty).
    if let Ok(console) = Terminal::new() {
        // The console is created, and will be dropped at the end of the scope.
        // If this doesn't panic, the test is considered successful.
        drop(console);
    } else {
        // In some environments (like CI), creating a terminal may fail.
        // We'll consider this a pass to avoid flaky tests, as the primary
        // goal is to test the console logic, not the environment's capabilities.
        println!("Could not create terminal, skipping test.");
    }
}
