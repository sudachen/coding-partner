use console::common::{Console, ConsoleError, ConsoleInput, Observability};
use console::stdio::StdIo;
use std::io::Cursor;

#[tokio::test]
async fn test_create_console() {
    let input = b"";
    let output = Vec::new();
    let _console = StdIo::new_with_buffers(&input[..], output);
}

#[tokio::test]
async fn test_prompt_input() {
    let input = b"hello world\n";
    let output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], output);
    let result = console.prompt_input().await;
    assert!(result.is_ok());
    let input = result.unwrap();
    match input {
        ConsoleInput::Prompt { prompt } => {
            assert_eq!(prompt, "hello world");
        }
        _ => panic!("Expected a prompt"),
    }
}

#[tokio::test]
async fn test_prompt_input_closed_pipe() {
    let input = b"";
    let output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], output);
    let result = console.prompt_input().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ConsoleError::Terminated));
}

#[tokio::test]
async fn test_state_transitions() {
    let input = b"";
    let output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], output);

    // Initial state is Prompting, so start_responding should succeed
    assert!(console.start_responding().await.is_ok());

    // State is now Responding, so stop_responding should succeed
    assert!(console.stop_responding().await.is_ok());

    // State is now Prompting again
    assert!(console.start_responding().await.is_ok());
}

#[tokio::test]
async fn test_invalid_state_calls() {
    let input = b"";
    let output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], output);

    // Calling responding methods in Prompting state should fail
    assert!(matches!(
        console.add_response_text("test".to_string()).await,
        Err(ConsoleError::InvalidState)
    ));

    // Transition to Responding state
    console.start_responding().await.unwrap();

    // Calling prompting methods in the Responding state should fail
    assert!(matches!(
        console.prompt_input().await,
        Err(ConsoleError::InvalidState)
    ));
}

#[tokio::test]
async fn test_observability() {
    let input = b"";
    let output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], output);

    // Initially, observability is off
    assert_eq!(console.observability(None), Observability::default());

    // Turn observability on
    let new_observability = Observability { statistics: true, thinking: true };
    assert_eq!(console.observability(Some(new_observability)), Observability::default());
    assert_eq!(console.observability(None), new_observability);

    // Turn observability off
    assert_eq!(console.observability(Some(Observability::default())), new_observability);
    assert_eq!(console.observability(None), Observability::default());
}

#[tokio::test]
async fn test_statistics_printing() {
    let input = b"";
    let mut output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], &mut output);

    // Enable observability
    console.observability(Some(Observability { statistics: true, thinking: false }));

    // Go to the responding state and back
    console.start_responding().await.unwrap();
    console.stop_responding().await.unwrap();

    // Check that statistics were printed
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Response time:"));
}

#[tokio::test]
async fn test_statistics_no_printing() {
    let input = b"";
    let mut output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], &mut output);

    // Observability is off by default

    // Go to the responding state and back
    console.start_responding().await.unwrap();
    console.stop_responding().await.unwrap();

    // Check that statistics were not printed
    let output_str = String::from_utf8(output).unwrap();
    assert!(!output_str.contains("Response time:"));
}

#[tokio::test]
async fn test_thinking_printing() {
    let mut output = Vec::new();
    let mut console = StdIo::new_with_buffers(Cursor::new(b""), &mut output);
    console.observability(Some(Observability { statistics: false, thinking: true }));

    console.start_responding().await.unwrap();
    console.add_thinking_text("one".to_string()).await.unwrap();
    console.add_thinking_text("two".to_string()).await.unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("one"));
    assert!(output_str.contains("two"));
}

#[tokio::test]
async fn test_thinking_no_printing() {
    let mut output = Vec::new();
    let mut console = StdIo::new_with_buffers(Cursor::new(b""), &mut output);
    console.observability(Some(Observability { statistics: false, thinking: false }));

    console.start_responding().await.unwrap();
    console.add_thinking_text("one".to_string()).await.unwrap();
    console.add_thinking_text("two".to_string()).await.unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(!output_str.contains("one"));
    assert!(!output_str.contains("two"));
}

#[tokio::test]
async fn test_command_parsing() {
    let scenarios = vec![
        ("/exit", Ok(ConsoleInput::Exit)),
        ("/thinking on", Ok(ConsoleInput::Thinking(true))),
        ("/thinking off", Ok(ConsoleInput::Thinking(false))),
        ("/statistics on", Ok(ConsoleInput::Statistics(true))),
        ("/statistics off", Ok(ConsoleInput::Statistics(false))),
        ("  /exit  ", Ok(ConsoleInput::Exit)),
        ("/thinking", Err(ConsoleError::UnknownCommand { command: "/thinking".to_string() })),
    ];

    for (input_str, expected) in scenarios {
        let input = format!("{}\n", input_str);
        let mut console = StdIo::new_with_buffers(&input.as_bytes()[..], Vec::new());
        let result = console.prompt_input().await;

        match (result, expected) {
            (Ok(res), Ok(exp)) => assert_eq!(res, exp),
            (Err(res), Err(exp)) => assert_eq!(res.to_string(), exp.to_string()),
            (res, exp) => panic!("Mismatched result and expectation: {:?} vs {:?}", res, exp),
        }
    }
}

#[tokio::test]
async fn test_unknown_command() {
    let input = b"/foo bar";
    let mut console = StdIo::new_with_buffers(&input[..], Vec::new());

    match console.prompt_input().await {
        Err(ConsoleError::UnknownCommand { command }) => {
            assert_eq!(command, "/foo");
        }
        other => panic!("Expected UnknownCommand error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_ask_user() {
    let input = b"user response\n";
    let mut output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], &mut output);

    // Switch to responding state
    console.start_responding().await.unwrap();

    // Ask the user a question
    let response = console.ask_user("Question for user".to_string()).await.unwrap();

    // Check that the question was printed
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Question for user"));

    // Check that the user's response was returned
    assert_eq!(response, "user response");
}

#[tokio::test]
async fn test_if_yes() {
    let input = b"y\nY\nyes\nYES\nn\nN\nno\nNO\n";
    let mut output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], &mut output);
    console.start_responding().await.unwrap();
    assert!(console.if_yes("Question?".to_string()).await.unwrap());
    assert!(console.if_yes("Question?".to_string()).await.unwrap());
    assert!(console.if_yes("Question?".to_string()).await.unwrap());
    assert!(console.if_yes("Question?".to_string()).await.unwrap());
    assert!(!console.if_yes("Question?".to_string()).await.unwrap());
    assert!(!console.if_yes("Question?".to_string()).await.unwrap());
    assert!(!console.if_yes("Question?".to_string()).await.unwrap());
    assert!(!console.if_yes("Question?".to_string()).await.unwrap());
    // Check that the prompts were printed
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(output_str.matches("[y/N]").count(), 8);
}

#[tokio::test]
async fn test_if_accept() {
    let input = b"y\nY\nyes\nYES\nn\nN\nno\nNO\n";
    let mut output = Vec::new();
    let mut console = StdIo::new_with_buffers(&input[..], &mut output);
    console.start_responding().await.unwrap();
    assert!(console.if_accept("Question?".to_string()).await.unwrap());
    assert!(console.if_accept("Question?".to_string()).await.unwrap());
    assert!(console.if_accept("Question?".to_string()).await.unwrap());
    assert!(console.if_accept("Question?".to_string()).await.unwrap());
    assert!(!console.if_accept("Question?".to_string()).await.unwrap());
    assert!(!console.if_accept("Question?".to_string()).await.unwrap());
    assert!(!console.if_accept("Question?".to_string()).await.unwrap());
    assert!(!console.if_accept("Question?".to_string()).await.unwrap());
    // Check that the prompts were printed
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(output_str.matches("[Y/n]").count(), 8);
}
