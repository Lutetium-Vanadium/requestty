use requestty::{Answer, Question};
use ui::{
    events::{KeyCode, TestEvents},
    style::Color,
};

mod helpers;

#[test]
fn test_validate() {
    let prompt = Question::password("name")
        .message("message")
        .mask('*')
        .validate(|s, _| {
            if s.len() > 2 {
                Ok(())
            } else {
                Err("The password must be more than 2 characters long".into())
            }
        });

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('t').into(),
        KeyCode::Char('r').into(),
        KeyCode::Enter.into(),
        KeyCode::Home.into(),
        KeyCode::Char('s').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("str".into()));
}

#[test]
fn test_validate_on_key() {
    let prompt = Question::password("name")
        .message("message")
        .mask('*')
        .validate_on_key(|s, _| s.len() > 2)
        .validate(|s, _| {
            if s.len() > 2 {
                Ok(())
            } else {
                Err("The password must be more than 2 characters long".into())
            }
        });

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('t').into(),
        KeyCode::Char('r').into(),
        KeyCode::Enter.into(),
        KeyCode::Home.into(),
        KeyCode::Char('s').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("str".into()));
}

#[test]
fn test_filter() {
    let prompt = Question::password("name")
        .message("message")
        .mask('*')
        .filter(|s, _| s + "--suffix");

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('s').into(),
        KeyCode::Char('t').into(),
        KeyCode::Char('r').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("str--suffix".into()));
}

#[test]
fn test_transform() {
    let prompt = Question::password("name")
        .message("message")
        .mask('*')
        .transform(|s, _, b| {
            b.set_fg(Color::Magenta)?;
            write!(b, "{:?}", s)?;
            b.set_fg(Color::Reset)
        });

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('s').into(),
        KeyCode::Char('t').into(),
        KeyCode::Char('r').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("str".into()));
}

#[test]
fn test_hidden() {
    let prompt = Question::password("name").message("message");

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('p').into(),
        KeyCode::Char('a').into(),
        KeyCode::Char('s').into(),
        KeyCode::Char('s').into(),
        KeyCode::Char('w').into(),
        KeyCode::Char('o').into(),
        KeyCode::Char('r').into(),
        KeyCode::Char('d').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("password".into()));
}

#[test]
fn test_on_esc() {
    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_one_with(
        Question::password("name")
            .message("message")
            .mask('*')
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
            Question::password("name")
                .message("message")
                .mask('*')
                .on_esc(requestty::OnEsc::SkipQuestion)
                .build(),
        ),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert!(res.is_empty());
}
