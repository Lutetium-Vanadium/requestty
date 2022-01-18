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
fn test_validate_on_key() {
    let prompt = Question::int("name")
        .message("message")
        .validate_on_key(|i, _| i > 3)
        .validate(|i, _| {
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

    let prompt = Question::int("name").message("message").default(32);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![KeyCode::Tab.into(), KeyCode::Enter.into()]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Int(32));

    let prompt = Question::int("name").message("message").default(32);

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![KeyCode::Right.into(), KeyCode::Enter.into()]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::Int(32));
}

#[test]
fn test_on_esc() {
    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_one_with(
        Question::int("name")
            .message("message")
            .on_esc(requestty::OnEsc::Terminate),
        &mut backend,
        &mut events,
    );

    assert!(matches!(res, Err(requestty::ErrorKind::Aborted)));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_with(
        Some(
            Question::int("name")
                .message("message")
                .on_esc(requestty::OnEsc::SkipQuestion)
                .build(),
        ),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert!(res.is_empty());
}
