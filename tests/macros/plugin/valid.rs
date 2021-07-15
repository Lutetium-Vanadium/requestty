use requestty::plugin::*;

#[derive(Debug)]
struct TestPlugin;

impl Plugin for TestPlugin {
    fn ask(
        self,
        _message: String,
        _answers: &Answers,
        _backend: &mut dyn Backend,
        _events: &mut dyn EventIterator,
    ) -> requestty::Result<Answer> {
        Ok(Answer::Int(0))
    }
}

fn main() {
    requestty::questions![Plugin {
        name: "name",
        plugin: TestPlugin,
    }];
}
