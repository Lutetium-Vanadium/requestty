use requestty::Question;
use ui::events::{KeyCode, TestEvents};

mod helpers;

#[test]
fn test_validate() {
    requestty::symbols::set(requestty::symbols::ASCII);

    let prompt = Question::int("name").message("message").validate(|i, _| {
        if i > 3 {
            Ok(())
        } else {
            Err("The number must be more than 3".into())
        }
    });

    let mut backend = helpers::SnapshotOnFlushBackend::new((50, 20).into());
    let mut events = TestEvents::new(vec![
        KeyCode::Enter.into(),
        KeyCode::Char('2').into(),
        KeyCode::Enter.into(),
        KeyCode::Char('-').into(),
        KeyCode::Enter.into(),
        KeyCode::Backspace.into(),
        KeyCode::Home.into(),
        KeyCode::Char('3').into(),
        KeyCode::Enter.into(),
    ]);

    requestty::prompt_one_with(prompt, &mut backend, &mut events).unwrap();
}
