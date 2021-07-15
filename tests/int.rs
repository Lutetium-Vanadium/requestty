use requestty::{Answer, Question};
use ui::{
    events::{KeyCode, TestEvents},
    style::Stylize,
};

mod helpers;

#[test]
fn test_validate() {
    let prompt = Question::int("name").message("message").validate(|i, _| {
        if i > 3 {
            Ok(())
        } else {
            Err("The number must be more than 3".into())
        }
    });

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Enter.into(),
        KeyCode::Char('2').into(),
        KeyCode::Enter.into(),
        KeyCode::Char('-').into(),
        KeyCode::Enter.into(),
        KeyCode::Backspace.into(),
        KeyCode::Home.into(),
        KeyCode::Char('3').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Int(32));
}

#[test]
fn test_filter() {
    let prompt = Question::int("name")
        .message("message")
        .filter(|i, _| i + 10);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('2').into(),
        KeyCode::Char('2').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Int(32));
}

#[test]
fn test_transform() {
    let prompt = Question::int("name")
        .message("message")
        .transform(|i, _, b| b.write_styled(&i.magenta()));

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('3').into(),
        KeyCode::Char('2').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Int(32));
}

#[test]
fn test_default() {
    let prompt = Question::int("name").message("message").default(32);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('3').into(),
        KeyCode::Backspace.into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Int(32));

    let prompt = Question::int("name").message("message").default(32);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Int(32));
}
