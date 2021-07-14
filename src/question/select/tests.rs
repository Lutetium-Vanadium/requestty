use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use ui::{backend::TestBackend, events::KeyCode, layout::Layout};

use crate::question::{Choice, Question, QuestionKind};

use super::*;

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

fn unwrap_select<'a>(question: impl Into<Question<'a>>) -> Select<'a> {
    match question.into().kind {
        QuestionKind::Select(s) => s,
        _ => unreachable!(),
    }
}

macro_rules! test_select {
    ($mod_name:ident { select = $select:expr; height = $height:expr $(;)? }) => {
        test_select!($mod_name {
            select = $select;
            height = $height;
            events = [
                KeyEvent::from(KeyCode::PageDown),
                KeyCode::Up.into(),
            ];
        });
    };

    ($mod_name:ident { select = $select:expr; height = $height:expr; events = $events:expr $(;)? }) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn test_height() {
                let size = (50, 20).into();
                let base_layout = Layout::new(5, size);
                let mut select = $select.into_prompt("message");

                let events = $events;

                for &key in events.iter() {
                    let mut layout = base_layout;

                    assert_eq!(select.height(&mut layout), $height);
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );

                    assert!(select.handle_key(key))
                }

                let mut layout = base_layout;

                assert_eq!(select.height(&mut layout), $height);
                assert_eq!(
                    layout,
                    base_layout.with_offset(0, $height).with_line_offset(0)
                );
            }

            #[test]
            fn test_render() {
                let size = (50, 20).into();
                let base_layout = Layout::new(5, size);
                let mut select = $select.into_prompt("message");

                let mut backend = TestBackend::new(size);

                let events = $events;

                for &key in events.iter() {
                    let mut layout = base_layout;
                    backend.reset_with_layout(layout);

                    assert!(select.render(&mut layout, &mut backend).is_ok());
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );
                    ui::assert_backend_snapshot!(backend);

                    assert!(select.handle_key(key))
                }

                let mut layout = base_layout;
                backend.reset_with_layout(layout);

                assert!(select.render(&mut layout, &mut backend).is_ok());
                assert_eq!(
                    layout,
                    base_layout.with_offset(0, $height).with_line_offset(0)
                );
                ui::assert_backend_snapshot!(backend);
            }
        }
    };
}

test_select!(basic {
    select = unwrap_select(
            SelectBuilder::new("name".into()).choices(choices(10)),
        );
    height = 11;
});

test_select!(pagination {
    select = unwrap_select(
            SelectBuilder::new("name".into()).choices(choices(20)).default(6),
        );
    height = 16;
});
