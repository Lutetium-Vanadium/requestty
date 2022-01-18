use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use requestty::{question::Choice, Question};
use ui::{
    events::{KeyCode, TestEvents},
    style::Color,
};

mod helpers;

const SEED: u64 = 9828123;
const SEP_RATIO: f32 = 0.3;
const DEFAULT_SEP_RATIO: f32 = 0.10;

fn choices(len: usize) -> impl Iterator<Item = Choice<String>> {
    let mut rng = ChaCha12Rng::seed_from_u64(SEED);

    (0..len).map(move |i| {
        let rand: f32 = rng.gen();
        if rand < DEFAULT_SEP_RATIO {
            Choice::DefaultSeparator
        } else if rand < SEP_RATIO {
            Choice::Separator(format!("Separator {}", i))
        } else {
            Choice::Choice(format!("Choice {}", i))
        }
    })
}

#[test]
fn test_tranform() {
    let size = (50, 20).into();

    let raw_select = Question::raw_select("name")
        .message("message")
        .choices(choices(10))
        .transform(|ans, _, b| {
            b.set_fg(Color::Magenta)?;
            write!(b, "{}: {}", ans.index, ans.text)?;
            b.set_fg(Color::Reset)
        });

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::PageDown.into(),
        KeyCode::Delete.into(),
        KeyCode::Char('6').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(raw_select, &mut backend, &mut events)
        .unwrap()
        .try_into_list_item()
        .unwrap();

    assert_eq!(ans.index, 8);
}

#[test]
fn test_default() {
    let size = (50, 20).into();

    let raw_select = Question::raw_select("name")
        .message("message")
        .default(4)
        .choices(choices(10));

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Enter.into()));

    let ans = requestty::prompt_one_with(raw_select, &mut backend, &mut events)
        .unwrap()
        .try_into_list_item()
        .unwrap();

    assert_eq!(ans.index, 4);
}

#[test]
fn test_validate() {
    let size = (50, 20).into();

    let raw_select = Question::raw_select("name")
        .message("message")
        .default(4)
        .choices(choices(10));

    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::Char('6').into(),
        KeyCode::Backspace.into(),
        KeyCode::Enter.into(),
        KeyCode::Char('6').into(),
        KeyCode::Char('6').into(),
        KeyCode::Enter.into(),
        KeyCode::Backspace.into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(raw_select, &mut backend, &mut events)
        .unwrap()
        .try_into_list_item()
        .unwrap();

    assert_eq!(ans.index, 8);
}

#[test]
fn test_on_esc() {
    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_one_with(
        Question::raw_select("name")
            .message("message")
            .choices(choices(10))
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
            Question::raw_select("name")
                .message("message")
                .choices(choices(10))
                .on_esc(requestty::OnEsc::SkipQuestion)
                .build(),
        ),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert!(res.is_empty());
}
