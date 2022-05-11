//! `requestty` (request-tty) is an easy-to-use collection of interactive cli prompts inspired by
//! [Inquirer.js].
//!
//! [Inquirer.js]: https://github.com/SBoudrias/Inquirer.js/
//!
//! # Questions
//!
//! This crate is based on creating [`Question`]s, and then prompting them to the user. There are 10
//! in-built [`Question`]s, but if none of them fit your need, you can [create your own!](#custom-prompts)
//!
//! There are 2 ways of creating [`Question`]s.
//!
//! ### Using builders
//!
//! ```
//! use requestty::{Question, Answers};
//!
//! let question = Question::expand("toppings")
//!     .message("What toppings do you want?")
//!     .when(|answers: &Answers| !answers["custom_toppings"].as_bool().unwrap())
//!     .choice('p', "Pepperoni and cheese")
//!     .choice('a', "All dressed")
//!     .choice('w', "Hawaiian")
//!     .build();
//! ```
//!
//! See [`Question`] for more information on the builders.
//!
//! ### Using macros (only with `macros` feature)
//!
//! Unlike the builder api, the macros can only be used to create a list of questions.
//!
#![cfg_attr(feature = "macros", doc = "```")]
#![cfg_attr(not(feature = "macros"), doc = "```ignore")]
//! use requestty::{questions, Answers};
//!
//! let questions = questions! [
//!     Expand {
//!         name: "toppings",
//!         message: "What toppings do you want?",
//!         when: |answers: &Answers| !answers["custom_toppings"].as_bool().unwrap(),
//!         choices: [
//!             ('p', "Pepperoni and cheese"),
//!             ('a', "All dressed"),
//!             ('w', "Hawaiian"),
//!         ]
//!     }
//! ];
//! ```
//!
//! See [`questions`] and [`prompt_module`](prompt_module!) for more information on the macros.
//!
//! ### Prompting
//!
//! [`Question`]s can be asked in 2 main ways.
//!
//! - Using direct [functions](#functions) provided by the crate.
//!   ```no_run
//!   let questions = vec![
//!       // Declare the questions you want to ask
//!   ];
//!
//!   let answers = requestty::prompt(questions)?;
//!   # Result::<_, requestty::ErrorKind>::Ok(())
//!   ```
//!
//! - Using [`PromptModule`]
//!   ```no_run
//!   use requestty::PromptModule;
//!
//!   let questions = PromptModule::new(vec![
//!       // Declare the questions you want to ask
//!   ]);
//!
//!   let answers = questions.prompt_all()?;
//!   # Result::<_, requestty::ErrorKind>::Ok(())
//!   ```
//!   This is mainly useful if you need more control over prompting the questions, and using
//!   previous [`Answers`].
//!
//! See the documentation of [`Question`] for more information on the different in-built questions.
//!
//! # Terminal Interaction
//!
//! Terminal interaction is handled by 2 traits: [`Backend`] and [`EventIterator`].
//!
//! The traits are already implemented and can be enabled with features for the following terminal
//! libraries:
//! - [`crossterm`](https://crates.io/crates/crossterm) (default)
//! - [`termion`](https://crates.io/crates/termion)
//!
//! The default is `crossterm` for the following reasons:
//! - Wider terminal support
//! - Better event processing (in my experience)
//!
//! [`Backend`]: prompt::Backend
//! [`EventIterator`]: prompt::EventIterator
//!
//! # Custom Prompts
//!
//! If the crate's in-built prompts does not satisfy your needs, you can build your own custom
//! prompts using the [`Prompt`](question::Prompt) trait.
//!
//! # Optional features
//!
//! - `macros`: Enabling this feature will allow you to use the [`questions`] and
//!   [`prompt_module`](prompt_module!) macros.
//!
//! - `smallvec` (default): Enabling this feature will use [`SmallVec`] instead of [`Vec`] for [auto
//!   completions]. This allows inlining single completions.
//!
//! - `crossterm` (default): Enabling this feature will use the [`crossterm`](https://crates.io/crates/crossterm)
//!   library for terminal interactions such as drawing and receiving events.
//!
//! - `termion`: Enabling this feature will use the [`termion`](https://crates.io/crates/termion)
//!   library for terminal interactions such as drawing and receiving events.
//!
//! [`SmallVec`]: https://docs.rs/smallvec/latest/smallvec/struct.SmallVec.html
//! [auto completions]: crate::question::InputBuilder::auto_complete
//!
//! # Examples
//!
//! ```no_run
//! use requestty::Question;
//!
//! let password = Question::password("password")
//!     .message("What is your password?")
//!     .mask('*')
//!     .build();
//!
//! let answer = requestty::prompt_one(password)?;
//!
//! println!("Your password was: {}", answer.as_string().expect("password returns a string"));
//! # Result::<_, requestty::ErrorKind>::Ok(())
//! ```
//!
//! For more examples, see the documentation for the various in-built questions, and the
//! [`examples`] directory.
//!
//! [`examples`]: https://github.com/lutetium-vanadium/requestty/tree/master/examples
#![deny(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    rustdoc::broken_intra_doc_links
)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod answer;
mod prompt_module;
pub mod question;

use ui::{backend::Backend, events::EventIterator};

/// A macro to easily write an iterator of [`Question`]s.
///
/// [`Question`]: crate::Question
///
/// # Usage
///
/// You can specify questions similar to a `vec![]` of struct instantiations. Each field corresponds
/// to calling the builder method of the same name for the respective question kind.
/// ```
/// # let some_variable = "message";
/// # let when = true;
/// # fn get_default() -> bool { true }
/// use requestty::questions;
///
/// let questions = questions![
///     MultiSelect {
///         // Each field takes a value, which can result in anything that implements `Into`
///         // for the required type
///         name: "name",
///         // The value can be any expression, for example a local variable.
///         message: some_variable,
///         // If the field name and the variable name are the same, this shorthand can be
///         // used.
///         when,
///         // While most values are generic expressions, if a array literal is passed to
///         // choices, some special syntax applies. Also, unlike other fields, 'choices'
///         // will call `choices_with_default` for `MultiSelect` questions only.
///         choices: [
///             // By default array entries are taken as `Choice(_)`s.
///             "Choice 1",
///             // For `MultiSelect` if the word 'default' follows the initial expression, a
///             // default can be set for that choice.
///             "Choice 2" default get_default(),
///             // The word 'separator' or 'sep' can be used to create separators. If no
///             // expression is given along with 'separator', it is taken as a `DefaultSeparator`.
///             separator,
///             // Otherwise if there is an expression, it is taken as a `Separator(_)`,
///             sep "Separator text!",
///         ],
///     },
/// ];
/// ```
///
/// # Inline
///
/// By default, the questions are stored in a [`Vec`]. However, if you wish to store the questions
/// on the stack, prefix the questions with `inline`:
/// ```
/// use requestty::questions;
///
/// let questions = questions! [ inline
///     Input {
///         name: "input"
///     },
/// ];
/// ```
///
/// Note that inlining only works for rust version 1.51 onwards. Pre 1.51, a [`Vec`] is still used.
///
/// See also [`prompt_module`](prompt_module!).
#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use r#macro::questions;

pub use answer::{Answer, Answers, ExpandItem, ListItem};
pub use prompt_module::PromptModule;
pub use question::{Choice::Choice, Choice::DefaultSeparator, Choice::Separator, Question};
pub use ui::{ErrorKind, OnEsc, Result};

/// A module that re-exports all the things required for writing custom [`Prompt`]s.
///
/// [`Prompt`]: prompt::Prompt
pub mod prompt {
    pub use crate::{question::Prompt, Answer, Answers};
    pub use ui::{
        backend::{self, Backend},
        events::{self, EventIterator},
        style,
    };
}

/// Prompt all the questions in the given iterator, with the default [`Backend`] and [`EventIterator`].
#[cfg(any(feature = "crossterm", feature = "termion"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
pub fn prompt<'a, Q>(questions: Q) -> Result<Answers>
where
    Q: IntoIterator<Item = Question<'a>>,
{
    PromptModule::new(questions.into_iter()).prompt_all()
}

/// Prompt the given question, with the default [`Backend`] and [`EventIterator`].
///
/// # Panics
///
/// This will panic if `when` on the [`Question`] prevents the question from being asked.
#[cfg(any(feature = "crossterm", feature = "termion"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
pub fn prompt_one<'a, I: Into<Question<'a>>>(question: I) -> Result<Answer> {
    let stdout = std::io::stdout();
    let mut stdout = ui::backend::get_backend(stdout.lock());
    let mut events = ui::events::get_events();

    prompt_one_with(question.into(), &mut stdout, &mut events)
}

/// Prompt all the questions in the given iterator, with the given [`Backend`] and [`EventIterator`].
pub fn prompt_with<'a, Q, B, E>(questions: Q, backend: &mut B, events: &mut E) -> Result<Answers>
where
    Q: IntoIterator<Item = Question<'a>>,
    B: Backend,
    E: EventIterator,
{
    PromptModule::new(questions.into_iter()).prompt_all_with(backend, events)
}

/// Prompt the given question, with the given [`Backend`] and [`EventIterator`].
///
/// # Panics
///
/// This will panic if `when` on the [`Question`] prevents the question from being asked.
pub fn prompt_one_with<'a, Q, B, E>(question: Q, backend: &mut B, events: &mut E) -> Result<Answer>
where
    Q: Into<Question<'a>>,
    B: Backend,
    E: EventIterator,
{
    let ans = question.into().ask(&Answers::default(), backend, events)?;

    Ok(ans.expect("The question wasn't asked").1)
}
