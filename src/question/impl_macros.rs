#[doc(hidden)]
#[macro_export]
macro_rules! impl_filter_builder {
    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        /// Function to change the final submitted value before it is displayed to the user and
        /// added to the [`Answers`].
        ///
        /// It is a [`FnOnce`] that is given the answer and the previous [`Answers`], and should
        /// return the new answer.
        ///
        /// This will be called after the answer has been validated.
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$meta])+
        pub fn filter<F>(mut self, filter: F) -> Self
        where
            F: FnOnce($t, &$crate::Answers) -> $t + 'a,
        {
            self.$inner.filter = $crate::question::Filter::Sync(Box::new(filter));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_auto_complete_builder {
    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        /// Function to suggest completions to the answer when the user presses `Tab`.
        ///
        /// It is a [`FnMut`] that is given the current state of the answer and the previous
        /// [`Answers`], and should return a list of completions.
        ///
        /// There must be at least 1 completion. Returning 0 completions will cause a panic. If
        /// there are no completions to give, the `auto_complete` function can simply return the
        /// state of the answer passed to it.
        ///
        /// If 1 completion is returned, then the state of the answer becomes that completion.
        ///
        /// If 2 or more completions are returned, a list of completions is displayed from which the
        /// user can pick one completion.
        ///
        /// [`Answers`]: crate::Answers
        ///
        /// # Panics
        ///
        /// If 0 completions are returned when the function is called, it will panic
        ///
        ///
        $(#[$meta])+
        pub fn auto_complete<F>(mut self, auto_complete: F) -> Self
        where
            F: FnMut($t, &$crate::Answers) -> Completions<$t> + 'a,
        {
            self.$inner.auto_complete =
                $crate::question::AutoComplete::Sync(Box::new(auto_complete));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_validate_builder {
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        $crate::impl_validate_builder!($(#[$meta])* impl &$t; $inner Validate);
    };

    ($(#[$meta:meta])+ by val $t:ty; $inner:ident) => {
        $crate::impl_validate_builder!($(#[$meta])* impl $t; $inner ValidateByVal);
    };

    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ impl $t:ty; $inner:ident $handler:ident) => {
        /// Function to validate the submitted value before it's returned.
        ///
        /// It is a [`FnMut`] that is given the answer and the previous [`Answers`], and should
        /// return `Ok(())` if the given answer is valid. If it is invalid, it should return an
        /// [`Err`] with the error message to display to the user.
        ///
        /// This will be called when the user presses the `Enter` key.
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$meta])*
        pub fn validate<F>(mut self, filter: F) -> Self
        where
            F: FnMut($t, &$crate::Answers) -> Result<(), String> + 'a,
        {
            self.$inner.validate = $crate::question::$handler::Sync(Box::new(filter));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_validate_on_key_builder {
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        $crate::impl_validate_on_key_builder!($(#[$meta])* impl &$t; $inner ValidateOnKey);
    };

    ($(#[$meta:meta])+ by val $t:ty; $inner:ident) => {
        $crate::impl_validate_on_key_builder!($(#[$meta])* impl $t; $inner ValidateOnKeyByVal);
    };

    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ impl $t:ty; $inner:ident $handler:ident) => {
        /// Function to validate the value on every key press. If the validation fails, the text is
        /// displayed in red.
        ///
        /// It is a [`FnMut`] that is given the answer and the previous [`Answers`], and should
        /// return `true` if it is valid.
        ///
        /// This will be called after every change in state. Note that this validation is purely
        /// cosmetic. If the user presses `Enter`, this function is **not** called. Instead, the one
        /// supplied to [`validate`](Self::validate) (if any) is the only validation that can
        /// prevent a user submission. This is required since final validation needs to return an
        /// error message to show the user.
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$meta])*
        pub fn validate_on_key<F>(mut self, filter: F) -> Self
        where
            F: FnMut($t, &$crate::Answers) -> bool + 'a,
        {
            self.$inner.validate_on_key = $crate::question::$handler::Sync(Box::new(filter));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_transform_builder {
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        $crate::impl_transform_builder!($(#[$meta])* impl &$t; $inner Transform);
    };

    ($(#[$meta:meta])+ by val $t:ty; $inner:ident) => {
        $crate::impl_transform_builder!($(#[$meta])* impl $t; $inner TransformByVal);
    };

    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ impl $t:ty; $inner:ident $handler:ident) => {
        /// Change the way the answer looks when displayed to the user.
        ///
        /// It is a [`FnOnce`] that is given the answer, previous [`Answers`] and the [`Backend`] to
        /// display the answer on. After the `transform` is called, a new line is also added.
        ///
        /// It will only be called once the user finishes answering the question.
        ///
        /// [`Answers`]: crate::Answers
        /// [`Backend`]: crate::prompt::Backend
        ///
        ///
        $(#[$meta])*
        pub fn transform<F>(mut self, transform: F) -> Self
        where
            F: FnOnce($t, &$crate::Answers, &mut dyn Backend) -> std::io::Result<()> + 'a,
        {
            self.$inner.transform = $crate::question::$handler::Sync(Box::new(transform));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! write_final {
    ($transform:expr, $message:expr, $ans:ident $([$tt:tt])?, $answers:expr, $backend:expr, |$ident:ident| $custom:expr) => {{
        ui::widgets::Prompt::write_finished_message(&$message, $ans.is_none(), $backend)?;

        // Weird reborrowing trick to make sure ans is not moved when $tt is ref, but is copied when
        // $tt is not there
        match (&$ans, $transform) {
            (&Some($($tt)? ans), Transform::Sync(transform)) => transform(ans, $answers, $backend)?,
            (&Some($($tt)? $ident), _) => $custom,
            (None, _) => {
                $backend.write_styled(&ui::style::Stylize::dark_grey("Skipped"))?;
            }
        }

        $backend.write_all(b"\n")?;
        $backend.flush()?;

        Ok($ans.map($crate::answer::Answer::from))
    }};
}
