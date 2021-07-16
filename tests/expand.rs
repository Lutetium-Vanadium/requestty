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
