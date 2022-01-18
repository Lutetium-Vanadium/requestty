use requestty::{prompt::*, question::CustomPromptBuilder, Question};

#[derive(Debug)]
struct Validate<'a> {
    message: &'static str,
    prompted: &'a mut bool,
}

impl Prompt for Validate<'_> {
    fn ask(
        self,
        message: String,
        _: &Answers,
        _: &mut dyn Backend,
        _: &mut dyn EventIterator,
    ) -> requestty::Result<Option<Answer>> {
        assert_eq!(message, self.message);
        *self.prompted = true;

        Ok(Some(Answer::Int(0)))
    }
}

fn custom_prompt<'a>(
    name: &str,
    message: &'static str,
    prompted: &'a mut bool,
) -> CustomPromptBuilder<'a> {
    Question::custom(name, Validate { message, prompted })
}

fn prompt_all<'a>(questions: impl IntoIterator<Item = Question<'a>>) {
    requestty::prompt_with(
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
        custom_prompt("name", "message", &mut prompted_0)
            .message("message")
            .build(),
        custom_prompt("name", "message", &mut prompted_1)
            .message("message")
            .build(),
        custom_prompt("name", "message", &mut prompted_2)
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
        custom_prompt("name-0", "message", &mut prompted_0)
            .message("message")
            .when(false)
            .build(),
        custom_prompt("name-1", "message", &mut prompted_1)
            .message("message")
            .when(|ans: &requestty::Answers| !ans.is_empty())
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
        custom_prompt("name", "message", &mut prompted_0)
            .message("message")
            .build(),
        custom_prompt("message", "message:", &mut prompted_1).build(),
    ]);

    assert!(prompted_0);
    assert!(prompted_1);
}
