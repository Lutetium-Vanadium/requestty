use discourse::question::Choice;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
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
    let checkbox = discourse::Question::select("name")
        .transform(|item, _, b| {
            b.set_fg(ui::style::Color::Magenta)?;
            write!(b, "{}: {}", item.index, item.name)?;
            b.set_fg(ui::style::Color::Reset)
        })
        .message("checkbox")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyEvent::from(KeyCode::PageDown),
        KeyEvent::from(KeyCode::Up),
        KeyCode::Enter.into(),
    ]);

    let ans = discourse::prompt_one_with(checkbox, &mut backend, &mut events)
        .unwrap()
        .try_into_list_item()
        .unwrap();

    assert_eq!(ans.index, 8);
}
