use requestty::Question;
use ui::{
    events::{KeyCode, TestEvents},
    style::Color,
};

mod helpers;

#[test]
fn test_tranform() {
    let size = (50, 20).into();

    let expand = Question::expand("name")
        .message("message")
        .choices(('a'..='g').map(|key| (key, format!("Choice {}", key.to_ascii_uppercase()))))
        .transform(|ans, _, b| {
            b.set_fg(Color::Magenta)?;

            write!(b, "{}: {}", ans.key, ans.text)?;

            b.set_fg(Color::Reset)
        });

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::Char('a').into(),
        KeyCode::Char('h').into(),
        KeyCode::Backspace.into(),
        KeyCode::Enter.into(),
        KeyCode::Char('b').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(expand, &mut backend, &mut events)
        .unwrap()
        .try_into_expand_item()
        .unwrap();

    assert_eq!(ans.key, 'b');
}

#[test]
fn test_default() {
    let size = (50, 20).into();

    let expand = Question::expand("name")
        .message("message")
        .default('d')
        .choices(('a'..='g').map(|key| (key, format!("Choice {}", key.to_ascii_uppercase()))))
        .transform(|ans, _, b| {
            b.set_fg(Color::Magenta)?;

            write!(b, "{}: {}", ans.key, ans.text)?;

            b.set_fg(Color::Reset)
        });

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(expand, &mut backend, &mut events)
        .unwrap()
        .try_into_expand_item()
        .unwrap();

    assert_eq!(ans.key, 'd');

    let expand = Question::expand("name")
        .message("message")
        .default('d')
        .choices(('a'..='g').map(|key| (key, format!("Choice {}", key.to_ascii_uppercase()))));

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::Char('h').into(),
        KeyCode::Enter.into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(expand, &mut backend, &mut events)
        .unwrap()
        .try_into_expand_item()
        .unwrap();

    assert_eq!(ans.key, 'd');
}

#[test]
fn test_on_esc() {
    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_one_with(
        Question::expand("name")
            .message("message")
            .default('d')
            .choices(('a'..='g').map(|key| (key, format!("Choice {}", key.to_ascii_uppercase()))))
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
            Question::expand("name")
                .message("message")
                .default('d')
                .choices(
                    ('a'..='g').map(|key| (key, format!("Choice {}", key.to_ascii_uppercase()))),
                )
                .on_esc(requestty::OnEsc::SkipQuestion)
                .build(),
        ),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert!(res.is_empty());
}
