use ui::{backend::Backend, events::EventIterator};

use super::{Options, Question, QuestionKind};
use crate::{Answer, Answers};

/// Plugins are a way to write custom [`Question`]s.
///
/// The plugin is given a `message`, the previous [`Answers`] and a [`Backend`] and
/// [`EventIterator`]. Using these, it is responsible for doing everything from rendering to user
/// interaction. While no particular look is enforced, it is recommended to keep a similar look to
/// the rest of the in-built questions.
///
/// You can use the `requestty-ui` crate to build the prompts. You can see the implementations of
/// the in-built questions for examples on how to use it.
///
/// See also [`Question::plugin`]
pub trait Plugin: std::fmt::Debug {
    /// Prompt the user with the given message, [`Answers`], [`Backend`] and [`EventIterator`]
    fn ask(
        self,
        message: String,
        answers: &Answers,
        backend: &mut dyn Backend,
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
        backend: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer>;
}

impl<T: Plugin> PluginInteral for Option<T> {
    fn ask(
        &mut self,
        message: String,
        answers: &Answers,
        backend: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer> {
        self.take()
            .expect("Plugin::ask called twice")
            .ask(message, answers, backend, events)
    }
}

/// The builder for custom questions.
///
/// See [`Plugin`] for more information on writing custom prompts.
///
/// # Examples
///
/// ```
/// use requestty::{plugin, Question};
///
/// #[derive(Debug)]
/// struct MyPlugin { /* ... */ }
///
/// # impl MyPlugin {
/// #     fn new() -> MyPlugin {
/// #         MyPlugin {}
/// #     }
/// # }
///
/// impl plugin::Plugin for MyPlugin {
///     fn ask(
///         self,
///         message: String,
///         answers: &plugin::Answers,
///         backend: &mut dyn plugin::Backend,
///         events: &mut dyn plugin::EventIterator,
///     ) -> requestty::Result<plugin::Answer> {
///         // ...
/// #         todo!()
///     }
/// }
///
/// let plugin = Question::plugin("my-plugin", MyPlugin::new())
///     .message("Hello from MyPlugin!")
///     .build();
/// ```
#[derive(Debug)]
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

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::{plugin, Question};
    ///
    /// #[derive(Debug)]
    /// struct MyPlugin { /* ... */ }
    ///
    /// # impl MyPlugin {
    /// #     fn new() -> MyPlugin {
    /// #         MyPlugin {}
    /// #     }
    /// # }
    ///
    /// impl plugin::Plugin for MyPlugin {
    ///     fn ask(
    ///         self,
    ///         message: String,
    ///         answers: &plugin::Answers,
    ///         backend: &mut dyn plugin::Backend,
    ///         events: &mut dyn plugin::EventIterator,
    ///     ) -> requestty::Result<plugin::Answer> {
    ///         // ...
    /// #         todo!()
    ///     }
    /// }
    ///
    /// let plugin = Question::plugin("my-plugin", MyPlugin::new())
    ///     .message("Hello from MyPlugin!")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{plugin, Question, Answers};
    ///
    /// #[derive(Debug)]
    /// struct MyPlugin { /* ... */ }
    ///
    /// # impl MyPlugin {
    /// #     fn new() -> MyPlugin {
    /// #         MyPlugin {}
    /// #     }
    /// # }
    ///
    /// impl plugin::Plugin for MyPlugin {
    ///     fn ask(
    ///         self,
    ///         message: String,
    ///         answers: &plugin::Answers,
    ///         backend: &mut dyn plugin::Backend,
    ///         events: &mut dyn plugin::EventIterator,
    ///     ) -> requestty::Result<plugin::Answer> {
    ///         // ...
    /// #         todo!()
    ///     }
    /// }
    ///
    /// let plugin = Question::plugin("my-plugin", MyPlugin::new())
    ///     .when(|previous_answers: &Answers| match previous_answers.get("use-custom-prompt") {
    ///         Some(ans) => !ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use requestty::{plugin, Question};
    ///
    /// #[derive(Debug)]
    /// struct MyPlugin { /* ... */ }
    ///
    /// # impl MyPlugin {
    /// #     fn new() -> MyPlugin {
    /// #         MyPlugin {}
    /// #     }
    /// # }
    ///
    /// impl plugin::Plugin for MyPlugin {
    ///     fn ask(
    ///         self,
    ///         message: String,
    ///         answers: &plugin::Answers,
    ///         backend: &mut dyn plugin::Backend,
    ///         events: &mut dyn plugin::EventIterator,
    ///     ) -> requestty::Result<plugin::Answer> {
    ///         // ...
    /// #         todo!()
    ///     }
    /// }
    ///
    /// let plugin = Question::plugin("my-plugin", MyPlugin::new())
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Consumes the builder returning a [`Question`]
    pub fn build(self) -> Question<'a> {
        Question::new(self.opts, QuestionKind::Plugin(self.plugin))
    }
}

impl<'a> From<PluginBuilder<'a>> for Question<'a> {
    /// Consumes the builder returning a [`Question`]
    fn from(builder: PluginBuilder<'a>) -> Self {
        builder.build()
    }
}
