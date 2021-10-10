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

fn unwrap_select<'a>(question: impl Into<Question<'a>>) -> RawSelect<'a> {
    match question.into().kind {
        QuestionKind::RawSelect(s) => s,
        _ => unreachable!(),
    }
}

fn raw_select(message: &str) -> RawSelectPrompt<'_> {
    unwrap_select(RawSelectBuilder::new("name".into()).choices(choices(10))).into_prompt(message)
}

#[test]
fn test_render() {
    let size = (50, 20).into();
    let base_layout = Layout::new(0, size);
    let mut backend = TestBackend::new_with_layout(size, base_layout);

    let mut raw_select = raw_select("message");

    let keys = [
        (KeyEvent::from(KeyCode::Down), 11),
        (KeyEvent::from(KeyCode::Delete), 10),
        (KeyEvent::from(KeyCode::Char('6')), 11),
    ];

    let mut layout = base_layout;
    assert!(raw_select.render(&mut layout, &mut backend).is_ok());
    ui::assert_backend_snapshot!(backend);
    assert_eq!(layout, base_layout.with_offset(0, 11).with_line_offset(10));

    for &(key, line_offset) in keys.iter() {
        layout = base_layout;
        backend.reset_with_layout(layout);
        assert!(raw_select.handle_key(key));
        assert!(raw_select.render(&mut layout, &mut backend).is_ok());
        ui::assert_backend_snapshot!(backend);
        assert_eq!(
            layout,
            base_layout.with_offset(0, 11).with_line_offset(line_offset)
        );
    }
}

#[test]
fn test_height() {
    let size = (50, 20).into();
    let base_layout = Layout::new(0, size);

    let mut raw_select = raw_select("message");

    let keys = [
        (KeyEvent::from(KeyCode::Down), 11),
        (KeyEvent::from(KeyCode::Delete), 10),
        (KeyEvent::from(KeyCode::Char('6')), 11),
    ];

    let mut layout = base_layout;
    assert_eq!(raw_select.height(&mut layout), 12);
    assert_eq!(layout, base_layout.with_offset(0, 11).with_line_offset(10));

    for &(key, line_offset) in keys.iter() {
        layout = base_layout;
        assert!(raw_select.handle_key(key));
        assert_eq!(raw_select.height(&mut layout), 12);
        assert_eq!(
            layout,
            base_layout.with_offset(0, 11).with_line_offset(line_offset)
        );
    }
}

#[test]
fn test_cursor_pos() {
    let size = (50, 20).into();
    let layout = Layout::new(5, size);

    let mut select = raw_select("message");

    let keys = [
        (KeyEvent::from(KeyCode::Down), 10),
        (KeyEvent::from(KeyCode::Delete), 10),
        (KeyEvent::from(KeyCode::Char('6')), 11),
    ];

    assert_eq!(select.cursor_pos(layout), (10, 11));

    for &(key, line_offset) in keys.iter() {
        assert!(select.handle_key(key));
        assert_eq!(select.cursor_pos(layout), (line_offset, 11));
    }

    let message = "-".repeat(size.width as usize) + "message";
    let mut select = raw_select(&message);

    assert_eq!(select.cursor_pos(layout), (10, 12));

    for &(key, line_offset) in keys.iter() {
        assert!(select.handle_key(key));
        assert_eq!(select.cursor_pos(layout), (line_offset, 12));
    }
}
