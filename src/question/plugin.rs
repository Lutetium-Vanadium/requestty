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
}

pub struct PluginBuilder<'a> {
    opts: Options<'a>,
    plugin: Box<dyn Plugin + 'a>,
}

impl<'a, P: Plugin + 'a> From<P> for Box<dyn Plugin + 'a> {
    fn from(plugin: P) -> Self {
        Box::new(plugin)
    }
}

impl<'a> PluginBuilder<'a> {
    pub(crate) fn new(name: String, plugin: Box<dyn Plugin + 'a>) -> Self {
        Self {
            opts: Options::new(name),
            plugin,
        }
    }

    crate::impl_options_builder!();

    pub fn build(self) -> Question<'a> {
        Question::new(self.opts, QuestionKind::Plugin(self.plugin))
    }
}

impl<'a> From<PluginBuilder<'a>> for Question<'a> {
    fn from(builder: PluginBuilder<'a>) -> Self {
        builder.build()
    }
}
