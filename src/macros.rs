/// A macro to easily write an iterator of [`Question`]s.
///
/// # Usage
///
/// You can specify questions similar to a `vec![]` of struct instantiations. Each field corresponds
/// to calling the builder method of the same name for the respective question kind.
/// ```
/// # let some_variable = "message";
/// # let when = true;
/// # fn get_default() -> bool { true }
/// use discourse::questions;
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
///             // expression is given along with 'separator', it is taken as a
///             // `DefaultSeparator`.
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
/// use discourse::questions;
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
/// See also [`prompt_module`].
pub use macros::questions;

/// A macro to easily write a [`PromptModule`].
///
/// # Usage
///
/// You can specify questions similar to a `vec![]` of struct instantiations. Each field corresponds
/// to calling the builder method of the same name for the respective question kind.
/// ```
/// # let some_variable = "message";
/// # let when = true;
/// # fn get_default() -> bool { true }
/// use discourse::prompt_module;
///
/// let prompt_module = prompt_module![
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
///             // expression is given along with 'separator', it is taken as a
///             // `DefaultSeparator`.
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
/// use discourse::prompt_module;
///
/// let prompt_module = prompt_module![ inline
///     Input {
///         name: "input"
///     },
/// ];
/// ```
///
/// Note that inlining only works for rust version 1.51 onwards. Pre 1.51, a [`Vec`] is still used.
///
/// See also [`questions`].
#[macro_export]
macro_rules! prompt_module {
    ($($tt:tt)*) => {
        $crate::PromptModule::new($crate::questions! [ $($tt)* ])
    };
}
