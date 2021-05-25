use inquisition::plugin::*;

#[derive(Debug)]
struct TestPlugin;

impl inquisition::question::Plugin for TestPlugin {
    fn ask(
        &mut self,
        _message: String,
        _answers: &Answers,
        _stdout: &mut dyn Backend,
        _events: &mut Events,
    ) -> inquisition::Result<Answer> {
        Ok(Answer::Int(0))
    }
}

fn main() {
    inquisition::questions![Plugin {
        name: "name",
        plugin: TestPlugin,
    }];
}
