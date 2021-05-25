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

pub struct PluginBuilder<'m, 'w, 'p> {
    opts: Options<'m, 'w>,
    plugin: Box<dyn Plugin + 'p>,
}

impl<'p, P: Plugin + 'p> From<P> for Box<dyn Plugin + 'p> {
    fn from(plugin: P) -> Self {
        Box::new(plugin)
    }
}

crate::impl_options_builder!(PluginBuilder<'q>; (this, opts) => {
    PluginBuilder {
        opts,
        plugin: this.plugin,
    }
});

impl<'m, 'w, 'p> PluginBuilder<'m, 'w, 'p> {
    pub(crate) fn new(name: String, plugin: Box<dyn Plugin + 'p>) -> Self {
        Self {
            opts: Options::new(name),
            plugin,
        }
    }

    pub fn build(self) -> Question<'m, 'w, 'p, 'static, 'static> {
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
