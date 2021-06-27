use discourse::{question::Completions, Answer, Question};
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

    let ans = discourse::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("str".into()));
    assert!(events.next().is_none());
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

    let ans = discourse::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("str--suffix".into()));
    assert!(events.next().is_none());
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

    let ans = discourse::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("str".into()));
    assert!(events.next().is_none());
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

    let ans = discourse::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("".into()));
    assert!(events.next().is_none());

    let prompt = Question::input("name")
        .message("message")
        .default("default");

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = discourse::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("default".into()));
    assert!(events.next().is_none());
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

    let ans = discourse::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
    assert_eq!(ans, Answer::String("string".into()));
    assert!(events.next().is_none());
}
