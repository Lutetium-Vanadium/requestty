use ui::{backend::Backend, error, events};

use super::{Options, Question, QuestionKind};
use crate::{Answer, Answers};

pub trait Plugin: std::fmt::Debug {
    fn ask(
        &mut self,
        message: String,
        answers: &Answers,
        stdout: &mut dyn Backend,
        events: &mut events::Events,
    ) -> error::Result<Answer>;

    crate::cfg_async! {
    fn ask_async<'future>(
        &mut self,
        message: String,
        answers: &Answers,
        stdout: &mut dyn Backend,
        events: &mut events::AsyncEvents,
    ) -> crate::question::BoxFuture<'future, error::Result<Answer>>;
    }
}

pub struct PluginBuilder<'m, 'w, 'p> {
    opts: Options<'m, 'w>,
    plugin: Box<dyn Plugin + 'p>,
}

impl<'p, P: Plugin + 'p> From<P> for Box<dyn Plugin + 'p> {
    fn from(plugin: P) -> Self {
        Box::new(plugin)
    }
}

impl Question<'static, 'static, 'static, 'static, 'static> {
    pub fn plugin<'a, N, P>(
        name: N,
        plugin: P,
    ) -> PluginBuilder<'static, 'static, 'a>
    where
        N: Into<String>,
        P: Into<Box<dyn Plugin + 'a>>,
    {
        PluginBuilder {
            opts: Options::new(name.into()),
            plugin: plugin.into(),
        }
    }
}

crate::impl_options_builder!(PluginBuilder<'q>; (this, opts) => {
    PluginBuilder {
        opts,
        plugin: this.plugin,
    }
});

impl<'m, 'w, 'q> PluginBuilder<'m, 'w, 'q> {
    pub fn build(self) -> Question<'m, 'w, 'q, 'static, 'static> {
        Question::new(self.opts, QuestionKind::Plugin(self.plugin))
    }
}

impl<'m, 'w, 'q> From<PluginBuilder<'m, 'w, 'q>>
    for Question<'m, 'w, 'q, 'static, 'static>
{
    fn from(builder: PluginBuilder<'m, 'w, 'q>) -> Self {
        builder.build()
    }
}
