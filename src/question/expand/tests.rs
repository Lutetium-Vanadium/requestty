use ui::{backend::TestBackend, events::KeyCode, layout::Layout};

use super::*;
use crate::question::QuestionKind;

#[test]
#[should_panic(expected = "Reserved key 'h'")]
fn test_panic_reserved_key() {
    ExpandBuilder::new("name".into()).choice('h', "help");
}

#[test]
#[should_panic(expected = "Duplicate key 'k'")]
fn test_panic_duplicate() {
    ExpandBuilder::new("name".into())
        .choice('k', "key 1")
        .choice('k', "key 2");
}

#[test]
#[should_panic(expected = "Duplicate key 'k'")]
fn test_panic_duplicate_case_insensitive() {
    ExpandBuilder::new("name".into())
        .choice('k', "key 1")
        .choice('K', "key 2");
}

#[test]
#[should_panic(expected = "Invalid default 'd' does not occur in the given choices")]
fn test_panic_invalid_default() {
    ExpandBuilder::new("name".into())
        .choice('k', "key 1")
        .default('d')
        .build();
}

macro_rules! expand {
    (let mut $expand:ident; $message:expr) => {
        expand!(let mut $expand; $message, 'h');
    };

    (let mut $expand:ident; $message:expr, $default:expr) => {
        let hint: String = "abcdefgh"
            .chars()
            .map(|c| {
                if c == $default {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();

        let expand =
            match ExpandBuilder::new("name".into())
                .choices(('a'..='g').map(|key| {
                    (key, format!("Choice {}", key.to_ascii_uppercase()))
                }))
                .build()
                .kind
            {
                QuestionKind::Expand(e) => e,
                _ => unreachable!(),
            };

        let mut $expand = ExpandPrompt {
            prompt: widgets::Prompt::new($message).with_hint(&hint),
            input: widgets::CharInput::with_filter_map(|c| {
                let c = c.to_ascii_lowercase();
                hint.chars()
                    .find(|o| o.eq_ignore_ascii_case(&c))
                    .and(Some(c))
            }),
            select: widgets::Select::new(expand),
            expanded: false,
        };
    };
}

#[test]
fn test_render() {
    let size = (50, 20).into();
    let base_layout = Layout::new(0, size);
    let mut backend = TestBackend::new_with_layout(size, base_layout);

    expand!(let mut expand; "message");

    let keys = [
        (
            KeyEvent::from(KeyCode::Char('a')),
            base_layout.with_offset(0, 2),
        ),
        (
            KeyEvent::from(KeyCode::Char('h')),
            base_layout.with_offset(0, 2),
        ),
        (
            KeyEvent::from(KeyCode::Backspace),
            base_layout.with_line_offset(21),
        ),
    ];

    let mut layout = base_layout;
    assert!(expand.render(&mut layout, &mut backend).is_ok());
    ui::assert_backend_snapshot!(backend);
    assert_eq!(layout, base_layout.with_line_offset(21));

    for &(key, expected_layout) in keys.iter() {
        layout = base_layout;
        backend.reset_with_layout(layout);
        assert!(expand.handle_key(key));
        assert!(expand.render(&mut layout, &mut backend).is_ok());
        ui::assert_backend_snapshot!(backend);
        assert_eq!(layout, expected_layout);
    }

    layout = base_layout;
    backend.reset_with_layout(layout);
    assert_eq!(expand.validate(), Ok(Validation::Continue));
    assert!(expand.render(&mut layout, &mut backend).is_ok());
    ui::assert_backend_snapshot!(backend);
    assert_eq!(layout, base_layout.with_offset(0, 9).with_line_offset(10));

    layout = base_layout;
    backend.reset_with_layout(layout);
    assert!(expand.handle_key(KeyCode::Char('c').into()));
    assert!(expand.render(&mut layout, &mut backend).is_ok());
    ui::assert_backend_snapshot!(backend);
    assert_eq!(layout, base_layout.with_offset(0, 9).with_line_offset(11));
}

#[test]
fn test_height() {
    let size = (50, 20).into();
    let base_layout = Layout::new(0, size);

    expand!(let mut expand; "message");

    let keys = [
        (
            KeyEvent::from(KeyCode::Char('a')),
            2,
            base_layout.with_offset(0, 2),
        ),
        (
            KeyEvent::from(KeyCode::Char('h')),
            2,
            base_layout.with_offset(0, 2),
        ),
        (
            KeyEvent::from(KeyCode::Backspace),
            1,
            base_layout.with_line_offset(21),
        ),
    ];

    let mut layout = base_layout;
    assert_eq!(expand.height(&mut layout), 1);
    assert_eq!(layout, base_layout.with_line_offset(21));

    for &(key, height, expected_layout) in keys.iter() {
        layout = base_layout;
        assert!(expand.handle_key(key));
        assert_eq!(expand.height(&mut layout), height);
        assert_eq!(layout, expected_layout);
    }

    layout = base_layout;
    assert_eq!(expand.validate(), Ok(Validation::Continue));
    assert_eq!(expand.height(&mut layout), 10);
    assert_eq!(layout, base_layout.with_offset(0, 9).with_line_offset(10));

    layout = base_layout;
    assert!(expand.handle_key(KeyCode::Char('c').into()));
    assert_eq!(expand.height(&mut layout), 10);
    assert_eq!(layout, base_layout.with_offset(0, 9).with_line_offset(11));
}

#[test]
fn test_cursor_pos() {
    let size = (50, 20).into();
    let layout = Layout::new(5, size);

    expand!(let mut expand; "message");

    let keys = [
        (KeyEvent::from(KeyCode::Char('a')), (27, 0)),
        (KeyEvent::from(KeyCode::Char('h')), (27, 0)),
        (KeyEvent::from(KeyCode::Backspace), (26, 0)),
    ];

    assert_eq!(expand.cursor_pos(layout), (26, 0));

    for &(key, cursor_pos) in keys.iter() {
        assert!(expand.handle_key(key));
        assert_eq!(expand.cursor_pos(layout), cursor_pos);
    }

    assert_eq!(expand.validate(), Ok(Validation::Continue));
    assert_eq!(expand.cursor_pos(layout), (10, 9));

    assert!(expand.handle_key(KeyCode::Char('c').into()));
    assert_eq!(expand.cursor_pos(layout), (11, 9));

    let message = "-".repeat(size.width as usize) + "message";
    expand!(let mut expand; &*message);

    let keys = [
        (KeyEvent::from(KeyCode::Char('a')), (27, 1)),
        (KeyEvent::from(KeyCode::Char('h')), (27, 1)),
        (KeyEvent::from(KeyCode::Backspace), (26, 1)),
    ];

    assert_eq!(expand.cursor_pos(layout), (26, 1));

    for &(key, cursor_pos) in keys.iter() {
        assert!(expand.handle_key(key));
        assert_eq!(expand.cursor_pos(layout), cursor_pos);
    }

    assert_eq!(expand.validate(), Ok(Validation::Continue));
    assert_eq!(expand.cursor_pos(layout), (10, 10));

    assert!(expand.handle_key(KeyCode::Char('c').into()));
    assert_eq!(expand.cursor_pos(layout), (11, 10));
}
