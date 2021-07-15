use super::{KeyCode, KeyEvent, KeyModifiers};

/// Movements that can be captured from a [`KeyEvent`]. See the individual variants for
/// what keys they capture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Movement {
    /// The Up arrow key, and `k` is captured
    Up,
    /// The Down arrow key, and `j` is captured
    Down,
    /// The Left arrow key, `h`, and `ctrl+b` is captured
    Left,
    /// The Right arrow key, `l`, and `ctrl+f` is captured
    Right,
    /// The PageUp key is captured
    PageUp,
    /// The PageDown key is captured
    PageDown,
    /// The Home key, `g`, `ctrl+a`  is captured
    Home,
    /// The End key, `G`, `ctrl+e`  is captured
    End,
    /// `ctrl+right`, `alt+right`, and `alt+f` are captured
    NextWord,
    /// `ctrl+left`, `alt+left`, and `alt+b` are captured
    PrevWord,
}

impl Movement {
    /// Gets the movement (if any) from the current event
    ///
    /// It also captures 'h', 'j', 'k', 'l', 'g', and 'G'. If these are required
    /// for some input, it must be checked before capturing a movement
    pub fn try_from_key(key: KeyEvent) -> Option<Movement> {
        let movement = match key.code {
            KeyCode::Left
                if key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Movement::PrevWord
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => Movement::PrevWord,

            KeyCode::Right
                if key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Movement::NextWord
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => Movement::NextWord,

            KeyCode::Up => Movement::Up,
            KeyCode::Char('k') => Movement::Up,

            KeyCode::Down => Movement::Down,
            KeyCode::Char('j') => Movement::Down,

            KeyCode::Left => Movement::Left,
            KeyCode::Char('h') => Movement::Left,
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => Movement::Left,

            KeyCode::Right => Movement::Right,
            KeyCode::Char('l') => Movement::Right,
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => Movement::Right,

            KeyCode::PageUp => Movement::PageUp,
            KeyCode::PageDown => Movement::PageDown,

            KeyCode::Home => Movement::Home,
            KeyCode::Char('g') => Movement::Home,
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => Movement::Home,

            KeyCode::End => Movement::End,
            KeyCode::Char('G') => Movement::End,
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => Movement::End,
            _ => return None,
        };

        Some(movement)
    }
}

#[test]
fn test_movement() {
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())),
        Some(Movement::Left)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())),
        Some(Movement::Left)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL)),
        Some(Movement::Left)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL,)),
        Some(Movement::PrevWord)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)),
        Some(Movement::PrevWord)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT)),
        Some(Movement::PrevWord)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())),
        Some(Movement::Right)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty())),
        Some(Movement::Right)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)),
        Some(Movement::Right)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
        Some(Movement::NextWord)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT)),
        Some(Movement::NextWord)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::ALT)),
        Some(Movement::NextWord)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Up, KeyModifiers::empty())),
        Some(Movement::Up)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty())),
        Some(Movement::Up)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty())),
        Some(Movement::Down)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty())),
        Some(Movement::Down)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty())),
        Some(Movement::PageDown)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty())),
        Some(Movement::PageUp)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Home, KeyModifiers::empty())),
        Some(Movement::Home)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())),
        Some(Movement::Home)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)),
        Some(Movement::Home)
    );

    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::End, KeyModifiers::empty())),
        Some(Movement::End)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::empty())),
        Some(Movement::End)
    );
    assert_eq!(
        Movement::try_from_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)),
        Some(Movement::End)
    );
}
