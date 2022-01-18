use requestty::prompt::*;

#[derive(Debug)]
struct TestPrompt;

impl Prompt for TestPrompt {
    fn ask(
        self,
        _message: String,
        _answers: &Answers,
        _backend: &mut dyn Backend,
        _events: &mut dyn EventIterator,
    ) -> requestty::Result<Option<Answer>> {
        Ok(Some(Answer::Int(0)))
    }
}

fn main() {
    requestty::questions![Custom {
        name: "name",
        prompt: TestPrompt,
    }];
}
