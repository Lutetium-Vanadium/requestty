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
fn test_validate() {
    let checkbox = discourse::Question::checkbox("name")
        .validate(|checked, _| {
            let count = checked.iter().filter(|&&b| b).count();
            if count > 1 {
                Ok(())
            } else {
                Err(format!(
                    "At least 2 items must be checked. {} items were checked",
                    count
                ))
            }
        })
        .message("checkbox")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyEvent::from(KeyCode::Down),
        KeyCode::Char(' ').into(),
        KeyCode::Enter.into(),
        KeyCode::End.into(),
        KeyCode::Char(' ').into(),
        KeyCode::Enter.into(),
    ]);

    let ans: Vec<_> = discourse::prompt_one_with(checkbox, &mut backend, &mut events)
        .unwrap()
        .try_into_list_items()
        .unwrap()
        .into_iter()
        .map(|item| item.index)
        .collect();

    assert!(events.next().is_none());
    assert_eq!(ans, [3, 9]);
}

#[test]
fn test_filter() {
    let checkbox = discourse::Question::checkbox("name")
        .filter(|mut checked, _| {
            checked.iter_mut().for_each(|b| *b = !*b);
            checked
        })
        .message("checkbox")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyEvent::from(KeyCode::Down),
        KeyCode::Char(' ').into(),
        KeyCode::End.into(),
        KeyCode::Char(' ').into(),
        KeyCode::Enter.into(),
    ]);

    let ans: Vec<_> = discourse::prompt_one_with(checkbox, &mut backend, &mut events)
        .unwrap()
        .try_into_list_items()
        .unwrap()
        .into_iter()
        .map(|item| item.index)
        .collect();

    assert!(events.next().is_none());
    assert_eq!(ans, [0, 4, 6, 7, 8]);
}

#[test]
fn test_transform() {
    let checkbox = discourse::Question::checkbox("name")
        .transform(|items, _, b| {
            b.set_fg(ui::style::Color::Magenta)?;
            for (i, item) in items.iter().enumerate() {
                write!(b, "{}: {}", item.index, item.name)?;
                if i + 1 != items.len() {
                    write!(b, ", ")?;
                }
            }
            b.set_fg(ui::style::Color::Reset)
        })
        .message("checkbox")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyEvent::from(KeyCode::Down),
        KeyCode::Char(' ').into(),
        KeyCode::End.into(),
        KeyCode::Char(' ').into(),
        KeyCode::Enter.into(),
    ]);

    let ans: Vec<_> = discourse::prompt_one_with(checkbox, &mut backend, &mut events)
        .unwrap()
        .try_into_list_items()
        .unwrap()
        .into_iter()
        .map(|item| item.index)
        .collect();

    assert!(events.next().is_none());
    assert_eq!(ans, [3, 9]);
}
