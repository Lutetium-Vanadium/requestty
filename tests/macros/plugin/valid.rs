use discourse::plugin::*;

#[derive(Debug)]
struct TestPlugin;

impl Plugin for TestPlugin {
    fn ask(
        self,
        _message: String,
        _answers: &Answers,
        _backend: &mut dyn Backend,
        _events: &mut dyn EventIterator,
    ) -> discourse::Result<Answer> {
        Ok(Answer::Int(0))
    }
}

fn main() {
    discourse::questions![Plugin {
        name: "name",
        plugin: TestPlugin,
    }];
}
