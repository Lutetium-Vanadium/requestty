use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use requestty::question::Choice;
use ui::events::{KeyCode, KeyEvent, TestEvents};

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
fn test_transform() {
    let select = requestty::Question::select("name")
        .transform(|item, _, b| {
            b.set_fg(ui::style::Color::Magenta)?;
            write!(b, "{}: {}", item.index, item.text)?;
            b.set_fg(ui::style::Color::Reset)
        })
        .message("select")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyEvent::from(KeyCode::PageDown),
        KeyEvent::from(KeyCode::Up),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(select, &mut backend, &mut events)
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
        requestty::Question::select("name")
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
            requestty::Question::select("name")
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
