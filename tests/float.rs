use requestty::{Answer, Question};
use ui::{
    events::{KeyCode, TestEvents},
    style::Stylize,
};

mod helpers;

#[test]
fn test_validate() {
    let prompt = Question::float("name").message("message").validate(|i, _| {
        if i > 3.0 {
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
        KeyCode::Char('.').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));
}

#[test]
fn test_validate_on_key() {
    let prompt = Question::float("name")
        .message("message")
        .validate_on_key(|i, _| i > 3.0)
        .validate(|i, _| {
            if i > 3.0 {
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
        KeyCode::Char('.').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));
}

#[test]
fn test_filter() {
    let prompt = Question::float("name")
        .message("message")
        .filter(|i, _| i + 1.0);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('2').into(),
        KeyCode::Char('.').into(),
        KeyCode::Char('2').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));
}

#[test]
fn test_transform() {
    let prompt = Question::float("name")
        .message("message")
        .transform(|i, _, b| b.write_styled(&i.magenta()));

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('3').into(),
        KeyCode::Char('.').into(),
        KeyCode::Char('2').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));
}

#[test]
fn test_default() {
    let prompt = Question::float("name").message("message").default(3.2);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('3').into(),
        KeyCode::Backspace.into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));

    let prompt = Question::float("name").message("message").default(3.2);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));

    let prompt = Question::float("name").message("message").default(3.2);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![KeyCode::Tab.into(), KeyCode::Enter.into()]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));

    let prompt = Question::float("name").message("message").default(3.2);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![KeyCode::Right.into(), KeyCode::Enter.into()]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Float(3.2));
}
