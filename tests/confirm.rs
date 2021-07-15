use requestty::{Answer, Question};
use ui::{
    events::{KeyCode, TestEvents},
    style::Stylize,
};

mod helpers;

#[test]
fn test_validate() {
    let size = (50, 20).into();

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::Enter.into(),
        KeyCode::Char('y').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(
        Question::confirm("name").message("message").build(),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert_eq!(ans, Answer::Bool(true));

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![KeyCode::Char('n').into(), KeyCode::Enter.into()]);

    let ans = requestty::prompt_one_with(
        Question::confirm("name")
            .message("message")
            .default(true)
            .build(),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert_eq!(ans, Answer::Bool(false));

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(
        Question::confirm("name")
            .message("message")
            .default(true)
            .build(),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert_eq!(ans, Answer::Bool(true));

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(
        Question::confirm("name")
            .message("message")
            .default(false)
            .build(),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert_eq!(ans, Answer::Bool(false));
}

#[test]
fn test_transform() {
    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(
        Question::confirm("name")
            .message("message")
            .default(true)
            .transform(|ans, _, b| b.write_styled(&ans.magenta())),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert_eq!(ans, Answer::Bool(true));
}
