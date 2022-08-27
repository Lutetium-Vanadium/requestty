use ui::{backend::TestBackend, layout::Layout, events::{KeyCode, KeyEvent}};

use crate::question::{Question, QuestionKind};

use super::*;

fn choices(len: usize) -> impl Iterator<Item = String> {
    (0..len).map(|choice| choice.to_string())
}

fn unwrap_order_select<'a>(question: impl Into<Question<'a>>) -> OrderSelect<'a> {
    match question.into().kind {
        QuestionKind::OrderSelect(c) => c,
        _ => unreachable!(),
    }
}

macro_rules! test_order_select {
    ($mod_name:ident { order_select = $order_select:expr; height = $height:expr $(;)? }) => {
        test_order_select!($mod_name {
            order_select = $order_select;
            height = $height;
            events = [
                KeyEvent::from(KeyCode::Char(' ')),
                KeyCode::Up.into(),
                KeyCode::Down.into(),
                KeyCode::Char(' ').into(),
            ];
            answers = Answers::default()
        });
    };

    ($mod_name:ident { order_select = $order_select:expr; height = $height:expr; events = $events:expr $(;)? }) => {
        test_order_select!($mod_name {
            order_select = $order_select;
            height = $height;
            events = $events;
            answers = Answers::default()
        });
    };

    ($mod_name:ident { order_select = $order_select:expr; height = $height:expr; events = $events:expr; answers = $answers:expr $(;)? }) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn test_height() {
                let size = (50, 20).into();
                let base_layout = Layout::new(5, size);
                let answers = $answers;
                let mut order_select = $order_select.into_order_select_prompt("message", &answers);

                let events = $events;

                for &key in events.iter() {
                    let mut layout = base_layout;

                    assert_eq!(order_select.height(&mut layout), $height);
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );

                    assert!(order_select.handle_key(key))
                }

                let mut layout = base_layout;

                assert_eq!(order_select.height(&mut layout), $height);
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
                let mut order_select = $order_select.into_order_select_prompt("message", &answers);

                let mut backend = TestBackend::new(size);

                let events = $events;

                for &key in events.iter() {
                    let mut layout = base_layout;
                    backend.reset_with_layout(layout);

                    assert!(order_select.render(&mut layout, &mut backend).is_ok());
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );
                    // key events (Up and down) will change the view, triggering an error
                    // ui::assert_backend_snapshot!(backend); 

                    assert!(order_select.handle_key(key))
                }

                let mut layout = base_layout;
                backend.reset_with_layout(layout);

                assert!(order_select.render(&mut layout, &mut backend).is_ok());
                assert_eq!(
                    layout,
                    base_layout.with_offset(0, $height).with_line_offset(0)
                );
                // key events (Up and down) will change the view, triggering an error
                // ui::assert_backend_snapshot!(backend);
            }
        }
    };
}

test_order_select!(basic {
    order_select = unwrap_order_select(
            OrderSelectBuilder::new("name".into()).choices(choices(10)),
        );
    height = 12;
});

test_order_select!(pagination {
    order_select = unwrap_order_select(
            OrderSelectBuilder::new("name".into()).choices(choices(20)),
        );
    height = 17;
});
