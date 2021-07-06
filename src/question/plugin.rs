use ui::{backend::Backend, events::EventIterator};

use super::{Options, Question, QuestionKind};
use crate::{Answer, Answers};

pub trait Plugin: std::fmt::Debug {
    fn ask(
        self,
        message: String,
        answers: &Answers,
        stdout: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer>;
}

/// The same trait as `Plugin`, except it take `&mut self` instead of `self`.
///
/// This is required since traits with functions that take `self` are not object safe, and so
/// implementors of the trait would have to use &mut self even though it will only be called once.
///
/// Now instead of QuestionKind::Plugin having a `dyn Plugin`, it has a `dyn PluginInteral`, which
/// is an `Option<T: Plugin>`.
pub(super) trait PluginInteral: std::fmt::Debug {
    fn ask(
        &mut self,
        message: String,
        answers: &Answers,
        stdout: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer>;
}

impl<T: Plugin> PluginInteral for Option<T> {
    fn ask(
        &mut self,
        message: String,
        answers: &Answers,
        stdout: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer> {
        self.take()
            .expect("Plugin::ask called twice")
            .ask(message, answers, stdout, events)
    }
}

pub struct PluginBuilder<'a> {
    opts: Options<'a>,
    plugin: Box<dyn PluginInteral + 'a>,
}

impl<'a, P: PluginInteral + 'a> From<P> for Box<dyn PluginInteral + 'a> {
    fn from(plugin: P) -> Self {
        Box::new(plugin)
    }
}

impl<'a> PluginBuilder<'a> {
    pub(super) fn new(name: String, plugin: Box<dyn PluginInteral + 'a>) -> Self {
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
