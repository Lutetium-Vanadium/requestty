use std::{
    env,
    ffi::OsString,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    process::Command,
};

use tempfile::TempPath;

use ui::{backend::Backend, events::EventIterator, style::Stylize, widgets, Validation, Widget};

use super::{Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug)]
pub(super) struct Editor<'a> {
    extension: Option<String>,
    default: Option<String>,
    editor: OsString,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    transform: Transform<'a, str>,
}

impl<'a> Default for Editor<'a> {
    fn default() -> Self {
        Self {
            editor: get_editor(),
            extension: None,
            default: None,
            filter: Filter::None,
            validate: Validate::None,
            transform: Transform::None,
        }
    }
}

fn get_editor() -> OsString {
    env::var_os("VISUAL")
        .or_else(|| env::var_os("EDITOR"))
        .unwrap_or_else(|| {
            if cfg!(windows) {
                "notepad".into()
            } else {
                "vim".into()
            }
        })
}

struct EditorPrompt<'a, 'e> {
    prompt: widgets::Prompt<&'a str>,
    file: File,
    path: TempPath,
    ans: String,
    editor: Editor<'e>,
    answers: &'a Answers,
}

impl Widget for EditorPrompt<'_, '_> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        backend: &mut B,
    ) -> io::Result<()> {
        self.prompt.render(layout, backend)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout)
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        // Cannot return this directly since we want to remove the extra space at the end
        let (x, y) = self.prompt.cursor_pos(layout);
        if x == 0 {
            (layout.width - 1, y - 1)
        } else {
            (x - 1, y)
        }
    }

    fn handle_key(&mut self, _: ui::events::KeyEvent) -> bool {
        false
    }
}

fn map_err(err: io::Error) -> widgets::Text<String> {
    widgets::Text::new(err.to_string())
}

impl ui::Prompt for EditorPrompt<'_, '_> {
    type ValidateErr = widgets::Text<String>;
    type Output = String;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if !Command::new(&self.editor.editor)
            .arg(&self.path)
            .status()
            .map_err(map_err)?
            .success()
        {
            return Err(map_err(io::Error::new(
                io::ErrorKind::Other,
                "Could not open editor",
            )));
        }

        self.ans.clear();
        self.file.read_to_string(&mut self.ans).map_err(map_err)?;
        self.file.seek(SeekFrom::Start(0)).map_err(map_err)?;

        if let Validate::Sync(ref mut validate) = self.editor.validate {
            validate(&self.ans, self.answers)
                .map_err(|err| map_err(io::Error::new(io::ErrorKind::InvalidInput, err)))?;
        }

        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        match self.editor.filter {
            Filter::Sync(filter) => filter(self.ans, self.answers),
            _ => self.ans,
        }
    }
}

impl Editor<'_> {
    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Answer> {
        let mut builder = tempfile::Builder::new();

        if let Some(ref extension) = self.extension {
            builder.suffix(extension);
        }

        let mut file = builder.tempfile()?;

        if let Some(ref default) = self.default {
            file.write_all(default.as_bytes())?;
            file.seek(SeekFrom::Start(0))?;
            file.flush()?;
        }

        let transform = self.transform.take();

        let (file, path) = file.into_parts();

        let ans = ui::Input::new(
            EditorPrompt {
                prompt: widgets::Prompt::new(&*message)
                    .with_hint("Press <enter> to launch your preferred editor.")
                    .with_delim(widgets::Delimiter::None),
                editor: self,
                file,
                path,
                ans: String::new(),
                answers,
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
            b.write_styled(&"Received".dark_grey())?
        );

        Ok(Answer::String(ans))
    }
}

/// The builder for the [`editor`] prompt.
///
/// Once the user exits their editor, the contents of the temporary file are read in as the
/// result. The editor to use is determined by the `$VISUAL` or `$EDITOR` environment variables.
/// If neither of those are present, `vim` (for unix) or `notepad` (for windows) is used.
///
/// <img
///   src="https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/editor.gif"
///   style="max-height: 30rem"
/// />
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use requestty::Question;
///
/// let editor = Question::editor("description")
///     .message("Please enter a short description about yourself")
///     .extension(".md")
///     .build();
/// ```
///
/// [`editor`]: crate::question::Question::editor
#[derive(Debug)]
pub struct EditorBuilder<'a> {
    opts: Options<'a>,
    editor: Editor<'a>,
}

impl<'a> EditorBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        EditorBuilder {
            opts: Options::new(name),
            editor: Default::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let editor = Question::editor("description")
    ///     .message("Please enter a short description about yourself")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, Answers};
    ///
    /// let editor = Question::editor("description")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("anonymous") {
    ///         Some(ans) => !ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, Answers};
    ///
    /// let editor = Question::editor("description")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Set a default value for the file
    ///
    /// If set, when the user first opens the file, it will contain the `default` value. Subsequent
    /// times will contain what was last written.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let editor = Question::editor("description")
    ///     .default("My name is ")
    ///     .build();
    /// ```
    pub fn default<I: Into<String>>(mut self, default: I) -> Self {
        self.editor.default = Some(default.into());
        self
    }

    /// Set an extension on the temporary file
    ///
    /// If set, the extension will be concatenated with the randomly generated filename. This is a
    /// useful way to signify accepted styles of input, and provide syntax highlighting on supported
    /// editors.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let editor = Question::editor("description")
    ///     .extension(".md")
    ///     .build();
    /// ```
    pub fn extension<I: Into<String>>(mut self, extension: I) -> Self {
        self.editor.extension = Some(extension.into());
        self
    }

    crate::impl_filter_builder! {
    /// # Examples
    ///
    /// ```
    /// # fn parse_markdown(s: String) -> String { s }
    /// use requestty::Question;
    ///
    /// let editor = Question::editor("description")
    ///     .filter(|description, previous_answers| parse_markdown(description))
    ///     .build();
    /// ```
    String; editor
    }

    crate::impl_validate_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let editor = Question::editor("description")
    ///     .validate(|description, previous_answers| if description.lines().count() >= 2 {
    ///         Ok(())
    ///     } else {
    ///         Err("Please enter a few lines".to_owned())
    ///     })
    ///     .build();
    /// ```
    str; editor
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let editor = Question::editor("description")
    ///     .transform(|description, previous_answers, backend| {
    ///         write!(backend, "\n{}", description)
    ///     })
    ///     .build();
    /// ```
    str; editor
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Editor(self.editor))
    }
}

impl<'a> From<EditorBuilder<'a>> for super::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: EditorBuilder<'a>) -> Self {
        builder.build()
    }
}

// TODO: figure out a way to write tests for this
