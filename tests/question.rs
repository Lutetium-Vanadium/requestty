use discourse::{plugin::*, question::PluginBuilder, Question};

#[derive(Debug)]
struct Validate<'a> {
    message: &'static str,
    prompted: &'a mut bool,
}

impl Plugin for Validate<'_> {
    fn ask(
        self,
        message: String,
        _: &Answers,
        _: &mut dyn Backend,
        _: &mut dyn EventIterator,
    ) -> discourse::Result<Answer> {
        assert_eq!(message, self.message);
        *self.prompted = true;

        Ok(Answer::Int(0))
    }
}

fn plugin<'a>(name: &str, message: &'static str, prompted: &'a mut bool) -> PluginBuilder<'a> {
    Question::plugin(name, Validate { message, prompted })
}

fn prompt_all<'a>(questions: impl IntoIterator<Item = Question<'a>>) {
    discourse::prompt_with(
        questions,
        &mut ui::backend::TestBackend::new((1, 1).into()),
        &mut ui::events::TestEvents::empty(),
    )
    .unwrap();
}

#[test]
fn test_ask_if_answered() {
    let mut prompted_0 = false;
    let mut prompted_1 = false;
    let mut prompted_2 = false;

    prompt_all(vec![
        plugin("name", "message", &mut prompted_0)
            .message("message")
            .build(),
        plugin("name", "message", &mut prompted_1)
            .message("message")
            .build(),
        plugin("name", "message", &mut prompted_2)
            .message("message")
            .ask_if_answered(true)
            .build(),
    ]);

    assert!(prompted_0);
    assert!(!prompted_1);
    assert!(prompted_2);
}

#[test]
fn test_when() {
    let mut prompted_0 = false;
    let mut prompted_1 = false;

    prompt_all(vec![
        plugin("name-0", "message", &mut prompted_0)
            .message("message")
            .when(false)
            .build(),
        plugin("name-1", "message", &mut prompted_1)
            .message("message")
            .when(|ans: &discourse::Answers| !ans.is_empty())
            .build(),
    ]);

    assert!(!prompted_0);
    assert!(!prompted_1);
}

#[test]
fn test_message() {
    let mut prompted_0 = false;
    let mut prompted_1 = false;

    prompt_all(vec![
        plugin("name", "message", &mut prompted_0)
            .message("message")
            .build(),
        plugin("message", "message:", &mut prompted_1).build(),
    ]);

    assert!(prompted_0);
    assert!(prompted_1);
}
