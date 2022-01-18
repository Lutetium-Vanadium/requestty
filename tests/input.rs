use requestty::{question::Completions, Answer, Question};
use ui::{
    events::{KeyCode, TestEvents},
    style::Color,
};

mod helpers;

#[test]
fn test_validate() {
    let prompt = Question::input("name").message("message").validate(|s, _| {
        if s.len() > 2 {
            Ok(())
        } else {
            Err("The string must be more than 2 characters long".into())
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
    let prompt = Question::input("name")
        .message("message")
        .validate_on_key(|s, _| s.len() > 2)
        .validate(|s, _| {
            if s.len() > 2 {
                Ok(())
            } else {
                Err("The string must be more than 2 characters long".into())
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
    let prompt = Question::input("name")
        .message("message")
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
    let prompt = Question::input("name")
        .message("message")
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
fn test_default() {
    let prompt = Question::input("name")
        .message("message")
        .default("default");

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('s').into(),
        KeyCode::Backspace.into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("".into()));

    let prompt = Question::input("name")
        .message("message")
        .default("default");

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("".into()));

    let prompt = Question::input("name")
        .message("message")
        .default("default");

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![KeyCode::Tab.into(), KeyCode::Enter.into()]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("default".into()));

    let prompt = Question::input("name")
        .message("message")
        .default("default");

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![KeyCode::Right.into(), KeyCode::Enter.into()]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("default".into()));
}

#[test]
fn test_auto_complete() {
    let prompt = Question::input("name")
        .message("message")
        .auto_complete(|s, _| {
            let mut completions: Completions<_> = ('g'..='j')
                .map(|c| {
                    let mut s = s.clone();
                    s.push(c);
                    s
                })
                .collect();
            completions.push(s + "k");
            completions
        });

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Char('s').into(),
        KeyCode::Char('t').into(),
        KeyCode::Char('r').into(),
        KeyCode::Tab.into(),
        KeyCode::Tab.into(),
        KeyCode::Tab.into(),
        KeyCode::Char('n').into(),
        KeyCode::Char('g').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("string".into()));
}

#[test]
fn test_on_esc() {
    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_one_with(
        Question::input("name")
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
            Question::input("name")
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
