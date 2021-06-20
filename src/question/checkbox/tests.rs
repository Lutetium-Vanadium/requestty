use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use ui::{backend::TestBackend, layout::Layout};

use crate::question::{Question, QuestionKind};

use super::*;

const SEED: u64 = 9828123;
const SEP_RATIO: f32 = 0.3;
const DEFAULT_SEP_RATIO: f32 = 0.10;

fn choices_with_default(len: usize) -> impl Iterator<Item = Choice<(String, bool)>> {
    let mut rng = ChaCha12Rng::seed_from_u64(SEED);

    (0..len).map(move |i| {
        let rand: f32 = rng.gen();
        if rand < DEFAULT_SEP_RATIO {
            Choice::DefaultSeparator
        } else if rand < SEP_RATIO {
            Choice::Separator(format!("Separator {}", i))
        } else {
            Choice::Choice((format!("Choice {}", i), rand > 0.7))
        }
    })
}

fn choices(len: usize) -> impl Iterator<Item = Choice<String>> {
    choices_with_default(len).map(|choice| choice.map(|(c, _)| c))
}

fn unwrap_checkbox<'a>(question: impl Into<Question<'a>>) -> Checkbox<'a> {
    match question.into().kind {
        QuestionKind::Checkbox(c) => c,
        _ => unreachable!(),
    }
}

macro_rules! test_checkbox {
    ($mod_name:ident { checkbox = $checkbox:expr; height = $height:expr $(;)? }) => {
        test_checkbox!($mod_name {
            checkbox = $checkbox;
            height = $height;
            events = [
                KeyEvent::from(KeyCode::Char('a')),
                KeyCode::Char('a').into(),
                KeyCode::Down.into(),
                KeyCode::Char(' ').into(),
                KeyCode::Char('i').into(),
            ];
            answers = Answers::default()
        });
    };

    ($mod_name:ident { checkbox = $checkbox:expr; height = $height:expr; events = $events:expr $(;)? }) => {
        test_checkbox!($mod_name {
            checkbox = $checkbox;
            height = $height;
            events = $events;
            answers = Answers::default()
        });
    };

    ($mod_name:ident { checkbox = $checkbox:expr; height = $height:expr; answers = $answers:expr $(;)? }) => {
        test_checkbox!($mod_name {
            checkbox = $checkbox;
            height = $height;
            events = [
                KeyEvent::from(KeyCode::Char('a')),
                KeyCode::Char('a').into(),
                KeyCode::Down.into(),
                KeyCode::Char(' ').into(),
                KeyCode::Char('i').into(),
            ];
            answers = $answers
        });
    };

    ($mod_name:ident { checkbox = $checkbox:expr; height = $height:expr; events = $events:expr; answers = $answers:expr $(;)? }) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn test_height() {
                let size = (50, 20).into();
                let base_layout = Layout::new(5, size);
                let answers = $answers;
                let mut checkbox = $checkbox.into_checkbox_prompt("message", &answers);

                let events = $events;

                for &key in events.iter() {
                    let mut layout = base_layout;

                    assert_eq!(checkbox.height(&mut layout), $height);
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );

                    assert!(checkbox.handle_key(key))
                }

                let mut layout = base_layout;

                assert_eq!(checkbox.height(&mut layout), $height);
                assert_eq!(
                    layout,
                    base_layout.with_offset(0, $height).with_line_offset(0)
                );
            }

            #[test]
            fn test_render() {
                let size = (50, 20).into();
                let base_layout = Layout::new(5, size);
                let answers = $answers;
                let mut checkbox = $checkbox.into_checkbox_prompt("message", &answers);

                let mut backend = TestBackend::new(size);

                let events = $events;

                for &key in events.iter() {
                    let mut layout = base_layout;
                    backend.reset_with_layout(layout);

                    assert!(checkbox.render(&mut layout, &mut backend).is_ok());
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );
                    ui::assert_backend_snapshot!(backend);

                    assert!(checkbox.handle_key(key))
                }

                let mut layout = base_layout;
                backend.reset_with_layout(layout);

                assert!(checkbox.render(&mut layout, &mut backend).is_ok());
                assert_eq!(
                    layout,
                    base_layout.with_offset(0, $height).with_line_offset(0)
                );
                ui::assert_backend_snapshot!(backend);
            }
        }
    };
}

test_checkbox!(basic {
    checkbox = unwrap_checkbox(
            CheckboxBuilder::new("name".into()).choices(choices(10)),
        );
    height = 12;
});

test_checkbox!(pagination {
    checkbox = unwrap_checkbox(
            CheckboxBuilder::new("name".into()).choices(choices(20)),
        );
    height = 17;
});

test_checkbox!(with_default {
    checkbox = unwrap_checkbox(
            CheckboxBuilder::new("name".into()).choices_with_default(choices_with_default(10)),
        );
    height = 12;
    events = [
        KeyEvent::from(KeyCode::Char('i')),
        KeyCode::Down.into(),
        KeyCode::Char(' ').into(),
        KeyCode::Char('a').into(),
        KeyCode::Char('a').into(),
    ]
});
