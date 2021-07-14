//! `discourse` is an easy-to-use collection of interactive cli prompts inspired by [Inquirer.js].
//!
//! [Inquirer.js]: https://github.com/SBoudrias/Inquirer.js/
//!
//! # Questions
//!
//! This crate is based on creating [`Question`]s, and then prompting them to the user. There are 10
//! in-built [`Question`]s, but if none of them fit your need, you can [create your own!](#plugins)
//!
//! There are 2 ways of creating [`Question`]s.
//!
//! ### Using builders
//!
//! ```
//! use discourse::{Question, Answers};
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
//! ### Using macros
//!
//! Unlike the builder api, the macros can only be used to create a list of questions.
//!
//! ```
//! use discourse::{questions, Answers};
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
//! See [`questions`] and [`prompt_module`] for more information on the macros.
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
//!   let answers = discourse::prompt(questions)?;
//!   # Result::<_, discourse::ErrorKind>::Ok(())
//!   ```
//!
//! - Using [`PromptModule`]
//!   ```no_run
//!   use discourse::PromptModule;
//!
//!   let questions = PromptModule::new(vec![
//!       // Declare the questions you want to ask
//!   ]);
//!
//!   let answers = questions.prompt_all()?;
//!   # Result::<_, discourse::ErrorKind>::Ok(())
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
//! The traits are already implemented for terminal libraries:
//! - [`crossterm`](https://crates.io/crates/crossterm) (default)
//! - [`termion`](https://crates.io/crates/termion)
//!
//! The default backend is `crossterm` for the following reasons:
//! - Wider terminal support
//! - Better event processing (in my experience)
//!
//! [`Backend`]: plugin::Backend
//! [`EventIterator`]: plugin::EventIterator
//!
//! # Plugins
//!
//! If the crate's in-built prompts does not satisfy your needs, you can build your own custom
//! prompts using the [`Plugin`](question::Plugin) trait.
//!
//! # Optional features
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
//! [`SmallVec`]: https://docs.rs/smallvec/1.6.1/smallvec/struct.SmallVec.html
//! [auto completions]: crate::question::InputBuilder::auto_complete
//!
//! # Examples
//!
//! ```no_run
//! use discourse::Question;
//!
//! let password = Question::password("password")
//!     .message("What is your password?")
//!     .mask('*')
//!     .build();
//!
//! let answer = discourse::prompt_one(password)?;
//!
//! println!("Your password was: {}", answer.as_string().expect("password returns a string"));
//! # Result::<_, discourse::ErrorKind>::Ok(())
//! ```
//!
//! For more examples, see the documentation for the various in-built questions, and the
//! [`examples`] directory.
//!
//! [`examples`]: https://github.com/lutetium-vanadium/discourse/tree/master/examples
#![deny(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    broken_intra_doc_links
)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod answer;
mod macros;
mod prompt_module;
pub mod question;

use ui::{
    backend::{get_backend, Backend},
    events::{get_events, EventIterator},
};

pub use self::macros::questions;
pub use answer::{Answer, Answers, ExpandItem, ListItem};
pub use prompt_module::PromptModule;
pub use question::{Choice::Choice, Choice::DefaultSeparator, Choice::Separator, Question};
pub use ui::{ErrorKind, Result};

/// A module that re-exports all the things required for writing [`Plugin`]s.
///
/// [`Plugin`]: plugin::Plugin
pub mod plugin {
    pub use crate::{question::Plugin, Answer, Answers};
    pub use ui::{
        backend::{self, Backend},
        events::{self, EventIterator},
        style,
    };
}

/// Prompt all the questions in the given iterator, with the default [`Backend`] and [`EventIterator`].
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
pub fn prompt_one<'a, I: Into<Question<'a>>>(question: I) -> Result<Answer> {
    let stdout = std::io::stdout();
    let mut stdout = get_backend(stdout.lock())?;
    let mut events = get_events();

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
