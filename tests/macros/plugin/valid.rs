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

    #[cfg(any(feature = "tokio", feature = "async-std", feature = "smol"))]
    fn ask_async<'future>(
        &mut self,
        _message: String,
        _answers: &Answers,
        _stdout: &mut dyn Backend,
        _events: &mut inquisition::plugin::AsyncEvents,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = inquisition::Result<Answer>>
                + Send
                + Sync
                + 'future,
        >,
    > {
        Box::pin(async { Ok(Answer::Int(0)) })
    }
}

fn main() {
    inquisition::questions![plugin {
        name: "name",
        plugin: TestPlugin,
    }];
}
