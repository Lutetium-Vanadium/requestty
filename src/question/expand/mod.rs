use std::{collections::HashSet, io};

use ui::{
    backend::{Backend, MoveDirection},
    events::{EventIterator, KeyEvent},
    style::{Color, Stylize},
    widgets::{self, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Options, Transform};
use crate::{Answer, Answers, ExpandItem};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct Expand<'a> {
    choices: super::ChoiceList<ExpandItem<Text<String>>>,
    selected: Option<char>,
    default: char,
    transform: Transform<'a, ExpandItem<String>>,
}

impl<'a> Default for Expand<'a> {
    fn default() -> Self {
        Expand {
            default: 'h',
            selected: None,
            choices: Default::default(),
            transform: Transform::None,
        }
    }
}

struct ExpandPrompt<'a, F> {
    prompt: widgets::Prompt<&'a str, &'a str>,
    select: widgets::Select<Expand<'a>>,
    input: widgets::CharInput<F>,
    expanded: bool,
}

impl<F: Fn(char) -> Option<char>> ExpandPrompt<'_, F> {
    fn selected(&mut self) -> Option<&mut ExpandItem<Text<String>>> {
        let key = self.input.value()?;

        self.select
            .list
            .choices
            .choices
            .iter_mut()
            .filter_map(|choice| match choice {
                Choice::Choice(choice) => Some(choice),
                _ => None,
            })
            .find(|item| item.key == key)
    }

    fn finish_with(self, c: char) -> ExpandItem<String> {
        let item = self
            .select
            .into_inner()
            .choices
            .choices
            .into_iter()
            .filter_map(|choice| match choice {
                Choice::Choice(choice) => Some(choice),
                _ => None,
            })
            .find(|item| item.key == c)
            .expect("Validation would fail unless an option was chosen");

        ExpandItem {
            name: item.name.text,
            key: item.key,
        }
    }
}

impl<F: Fn(char) -> Option<char>> Prompt for ExpandPrompt<'_, F> {
    type ValidateErr = &'static str;
    type Output = ExpandItem<String>;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        match self.input.value().unwrap_or(self.select.list.default) {
            'h' => {
                self.expanded = true;
                self.input.clear_value();
                self.select.list.selected = None;
                Ok(Validation::Continue)
            }
            _ => Ok(Validation::Finish),
        }
    }

    fn finish(self) -> Self::Output {
        let c = self.input.value().unwrap_or(self.select.list.default);
        self.finish_with(c)
    }
}

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl<F: Fn(char) -> Option<char>> ui::Widget for ExpandPrompt<'_, F> {
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
        self.prompt.render(layout, b)?;
        if self.expanded {
            self.select.render(layout, b)?;
            b.write_all(ANSWER_PROMPT)?;
            layout.line_offset = ANSWER_PROMPT.len() as u16;
            self.input.render(layout, b)
        } else {
            self.input.render(layout, b)?;

            if self.input.value().is_some() {
                b.move_cursor(MoveDirection::NextLine(1))?;
                b.write_styled(&ui::symbols::SMALL_ARROW.cyan())?;
                b.write_all(b" ")?;

                layout.offset_y += 1;
                layout.offset_x += 2;

                match self.selected() {
                    Some(item) => {
                        item.render(layout, b)?;
                        b.move_cursor(MoveDirection::Column(0))?;
                    }
                    None => {
                        layout.offset_y += 1;
                        b.write_all(b"Help, list all options")?;
                        b.move_cursor(MoveDirection::NextLine(1))?;
                    }
                }

                layout.offset_x = 0;
                layout.line_offset = 0;
            }

            Ok(())
        }
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        if self.expanded {
            // Don't need to add 1 for the answer prompt, since this will over count by 1 anyways
            let height = self.prompt.height(layout) + self.select.height(layout);
            layout.line_offset = ANSWER_PROMPT.len() as u16 + self.input.value().is_some() as u16;
            height
        } else if self.input.value().is_some() {
            let height = self.prompt.height(layout) - 1 + self.input.height(layout);

            layout.offset_y += 1;
            layout.line_offset = 0;

            // selected will return None if the help option is selected
            let selected_height = match self.selected().map(|c| c.height(layout)) {
                Some(height) => height,
                None => {
                    layout.offset_y += 1;
                    1
                }
            };

            height + selected_height
        } else {
            self.prompt.height(layout) + self.input.height(layout) - 1
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.input.handle_key(key) {
            self.select.list.selected = self.input.value();
            true
        } else if self.expanded {
            self.select.handle_key(key)
        } else {
            false
        }
    }

    fn cursor_pos(&mut self, mut layout: ui::layout::Layout) -> (u16, u16) {
        if self.expanded {
            let w = self
                .input
                .cursor_pos(layout.with_line_offset(ANSWER_PROMPT.len() as u16))
                .0;
            (w, self.height(&mut layout) - 1)
        } else {
            self.input
                .cursor_pos(layout.with_cursor_pos(self.prompt.cursor_pos(layout)))
        }
    }
}

impl widgets::List for Expand<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        _: bool,
        layout: ui::layout::Layout,
        b: &mut B,
    ) -> io::Result<()> {
        if index == self.choices.len() {
            return self.render_choice(None, layout, b);
        }

        match &mut self.choices[index] {
            Choice::Choice(_) => self.render_choice(Some(index), layout, b),
            separator => {
                b.set_fg(Color::DarkGrey)?;
                b.write_all(b"   ")?;
                super::get_sep_str(separator).render(&mut layout.with_line_offset(3), b)?;
                b.set_fg(Color::Reset)
            }
        }
    }

    fn is_selectable(&self, _: usize) -> bool {
        true
    }

    fn height_at(&mut self, index: usize, mut layout: ui::layout::Layout) -> u16 {
        if index >= self.choices.len() {
            // Help option
            1
        } else {
            layout.offset_x += 5;
            self.choices[index].height(&mut layout)
        }
    }

    fn len(&self) -> usize {
        self.choices.len() + 1
    }

    fn page_size(&self) -> usize {
        self.choices.page_size()
    }

    fn should_loop(&self) -> bool {
        self.choices.should_loop()
    }
}

impl Expand<'_> {
    fn has_valid_default(&self) -> bool {
        self.default == 'h'
            || self.choices.choices.iter().any(
                |c| matches!(c, Choice::Choice(ExpandItem { key, .. }) if *key == self.default),
            )
    }

    fn render_choice<B: Backend>(
        &mut self,
        index: Option<usize>,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> io::Result<()> {
        let key = match index {
            Some(index) => self.choices[index].as_ref().unwrap_choice().key,
            None => 'h',
        };

        let hovered = self.selected.map(|c| c == key).unwrap_or(false);

        if hovered {
            b.set_fg(Color::Cyan)?;
        }

        write!(b, "  {}) ", key)?;

        layout.offset_x += 5;

        match index {
            Some(index) => self.choices[index]
                .as_mut()
                .unwrap_choice()
                .render(&mut layout, b)?,
            None => "Help, list all options".render(&mut layout.with_line_offset(5), b)?,
        }

        if hovered {
            b.set_fg(Color::Reset)?;
        }

        Ok(())
    }

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Answer> {
        let help_key = if self.default == 'h' { 'H' } else { 'h' };

        let hint: String = self
            .choices
            .choices
            .iter()
            .filter_map(|choice| match choice {
                Choice::Choice(choice) if self.default == choice.key => {
                    Some(choice.key.to_ascii_uppercase())
                }
                Choice::Choice(choice) => Some(choice.key.to_ascii_lowercase()),
                _ => None,
            })
            .chain(std::iter::once(help_key))
            .collect();

        let transform = self.transform.take();

        let ans = ui::Input::new(
            ExpandPrompt {
                prompt: widgets::Prompt::new(&*message).with_hint(&hint),
                input: widgets::CharInput::with_filter_map(|c| {
                    let c = c.to_ascii_lowercase();
                    hint.chars()
                        .find(|o| o.eq_ignore_ascii_case(&c))
                        .and(Some(c))
                }),
                select: widgets::Select::new(self),
                expanded: false,
            },
            b,
        )
        .run(events)?;

        crate::write_final!(
            transform,
            message,
            &ans,
            answers,
            b,
            b.write_styled(
                &ans.name
                    .lines()
                    .next()
                    .expect("There must be at least one line in a `str`")
                    .cyan()
            )?
        );

        Ok(Answer::ExpandItem(ans))
    }
}

/// The builder for a [`expand`] prompt.
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use discourse::Question;
///
/// let expand = Question::expand("overwrite")
///     .message("Conflict on `file.rs`")
///     .choices(vec![
///         ('y', "Overwrite"),
///         ('a', "Overwrite this one and all next"),
///         ('d', "Show diff"),
///     ])
///     .default_separator()
///     .choice('x', "Abort")
///     .build();
/// ```
///
/// [`expand`]: crate::question::Question::expand
#[derive(Debug)]
pub struct ExpandBuilder<'a> {
    opts: Options<'a>,
    expand: Expand<'a>,
    keys: HashSet<char>,
}

impl<'a> ExpandBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        ExpandBuilder {
            opts: Options::new(name),
            expand: Default::default(),
            keys: HashSet::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .message("Conflict on `file.rs`")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use discourse::{Question, Answers};
    ///
    /// let expand = Question::expand("overwrite")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("ignore-conflicts") {
    ///         Some(ans) => ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Set a default key for the expand
    ///
    /// If no key is entered by the user and they press `Enter`, the default key is used.
    ///
    /// If `default` is unspecified, it defaults to the 'h' key.
    ///
    /// # Panics
    ///
    /// If the default given is not a key, it will cause a panic on [`build`]
    ///
    /// [`build`]: Self::build
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .choice('d', "Show diff")
    ///     .default('d')
    ///     .build();
    /// ```
    pub fn default(mut self, default: char) -> Self {
        self.expand.default = default;
        self
    }

    /// The maximum height that can be taken by the expanded list
    ///
    /// If the total height exceeds the page size, the list will be scrollable.
    ///
    /// The `page_size` must be a minimum of 5. If `page_size` is not set, it will default to 15. It
    /// will only be used if the user expands the prompt.
    ///
    /// # Panics
    ///
    /// It will panic if the `page_size` is less than 5.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .page_size(10)
    ///     .build();
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        assert!(page_size >= 5, "page size can be a minimum of 5");

        self.expand.choices.set_page_size(page_size);
        self
    }

    /// Whether to wrap around when user gets to the last element.
    ///
    /// This only applies when the list is scrollable, i.e. page size > total height.
    ///
    /// If `should_loop` is not set, it will default to `true`. It will only be used if the user
    /// expands the prompt.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .should_loop(false)
    ///     .build();
    /// ```
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.expand.choices.set_should_loop(should_loop);
        self
    }

    /// Inserts a [`Choice`] with the given key
    ///
    /// See [`expand`] for more information.
    ///
    /// [`Choice`]: super::Choice::Choice
    /// [`expand`]: super::Question::expand
    ///
    /// # Panics
    ///
    /// It will panic if the key is 'h' or a duplicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .choice('x', "Abort")
    ///     .build();
    /// ```
    pub fn choice<I: Into<String>>(mut self, mut key: char, name: I) -> Self {
        key = key.to_ascii_lowercase();

        if key == 'h' {
            panic!("Reserved key 'h'");
        }
        if self.keys.contains(&key) {
            panic!("Duplicate key '{}'", key);
        }

        self.keys.insert(key);

        self.expand.choices.choices.push(Choice::Choice(ExpandItem {
            key,
            name: Text::new(name.into()),
        }));

        self
    }

    /// Inserts a [`Separator`] with the given text
    ///
    /// See [`expand`] for more information.
    ///
    /// [`Separator`]: super::Choice::Separator
    /// [`expand`]: super::Question::expand
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .separator("-- custom separator text --")
    ///     .build();
    /// ```
    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.expand
            .choices
            .choices
            .push(Choice::Separator(text.into()));
        self
    }

    /// Inserts a [`DefaultSeparator`]
    ///
    /// See [`expand`] for more information.
    ///
    /// [`DefaultSeparator`]: super::Choice::DefaultSeparator
    /// [`expand`]: super::Question::expand
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .default_separator()
    ///     .build();
    /// ```
    pub fn default_separator(mut self) -> Self {
        self.expand.choices.choices.push(Choice::DefaultSeparator);
        self
    }

    /// Extends the given iterator of [`Choice`]s
    ///
    /// See [`expand`] for more information.
    ///
    /// [`Choice`]: super::Choice
    /// [`expand`]: super::Question::expand
    ///
    /// # Panics
    ///
    /// It will panic if the key of any choice is 'h' or a duplicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .choices(vec![
    ///         ('y', "Overwrite"),
    ///         ('a', "Overwrite this one and all next"),
    ///         ('d', "Show diff"),
    ///     ])
    ///     .build();
    /// ```
    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<ExpandItem<String>>>,
        I: IntoIterator<Item = T>,
    {
        let Self {
            ref mut keys,
            ref mut expand,
            ..
        } = self;

        expand.choices.choices.extend(choices.into_iter().map(|c| {
            c.into().map(|ExpandItem { name, mut key }| {
                key = key.to_ascii_lowercase();
                if key == 'h' {
                    panic!("Reserved key 'h'");
                }
                if keys.contains(&key) {
                    panic!("Duplicate key '{}'", key);
                }
                keys.insert(key);

                ExpandItem {
                    name: Text::new(name),
                    key,
                }
            })
        }));

        self
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .transform(|choice, previous_answers, backend| {
    ///         write!(backend, "({}) {}", choice.key, choice.name)
    ///     })
    ///     .build();
    /// ```
    ExpandItem<String>; expand
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> super::Question<'a> {
        if !self.expand.has_valid_default() {
            panic!(
                "Invalid default '{}' does not occur in the given choices",
                self.expand.default
            );
        }

        super::Question::new(self.opts, super::QuestionKind::Expand(self.expand))
    }
}

impl<'a> From<ExpandBuilder<'a>> for super::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: ExpandBuilder<'a>) -> Self {
        builder.build()
    }
}
