use std::{
    env,
    ffi::OsString,
    fmt,
    io::{self, Read, Seek, SeekFrom, Write},
    process::Command,
};

use crossterm::style::Colorize;
use tempfile::NamedTempFile;
use ui::{Validation, Widget};

use crate::{error, Answer};

use super::{none, some, Filter, Options, Transformer, Validate};

pub struct Editor<'f, 'v, 't> {
    postfix: Option<String>,
    default: Option<String>,
    editor: OsString,
    filter: Option<Box<Filter<'f, String>>>,
    validate: Option<Box<Validate<'v, str>>>,
    transformer: Option<Box<Transformer<'t, str>>>,
}

impl fmt::Debug for Editor<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Editor")
            .field("default", &self.default)
            .field("postfix", &self.postfix)
            .field("editor", &self.editor)
            .field("filter", &self.filter.as_ref().map_or_else(none, some))
            .field("validate", &self.validate.as_ref().map_or_else(none, some))
            .field(
                "transformer",
                &self.transformer.as_ref().map_or_else(none, some),
            )
            .finish()
    }
}

impl Default for Editor<'static, 'static, 'static> {
    fn default() -> Self {
        Self {
            editor: get_editor(),
            postfix: None,
            default: None,
            filter: None,
            validate: None,
            transformer: None,
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

struct EditorPrompt<'f, 'v, 't> {
    file: NamedTempFile,
    ans: String,
    message: String,
    editor: Editor<'f, 'v, 't>,
}

impl Widget for EditorPrompt<'_, '_, '_> {
    fn render<W: Write>(&mut self, _: usize, _: &mut W) -> crossterm::Result<()> {
        Ok(())
    }

    fn height(&self) -> usize {
        0
    }
}

impl ui::Prompt for EditorPrompt<'_, '_, '_> {
    type ValidateErr = io::Error;
    type Output = String;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some("Press <enter> to launch your preferred editor.")
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if !Command::new(&self.editor.editor)
            .arg(self.file.path())
            .status()?
            .success()
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not open editor",
            ));
        }

        self.ans.clear();
        self.file.read_to_string(&mut self.ans)?;
        self.file.seek(SeekFrom::Start(0))?;

        if let Some(ref validate) = self.editor.validate {
            validate(&self.ans).map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
        }

        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        match self.editor.filter {
            Some(filter) => filter(self.ans),
            None => self.ans,
        }
    }

    fn has_default(&self) -> bool {
        false
    }
}

impl Editor<'_, '_, '_> {
    pub fn ask<W: Write>(mut self, message: String, w: &mut W) -> error::Result<Answer> {
        let mut builder = tempfile::Builder::new();

        if let Some(ref postfix) = self.postfix {
            builder.suffix(postfix);
        }

        let mut file = builder.tempfile()?;

        if let Some(ref default) = self.default {
            file.write_all(default.as_bytes())?;
            file.seek(SeekFrom::Start(0))?;
            file.flush()?;
        }

        let transformer = self.transformer.take();

        let ans = ui::Input::new(EditorPrompt {
            message,
            editor: self,
            file,
            ans: String::new(),
        })
        .run(w)?;

        match transformer {
            Some(transformer) => transformer(&ans, w)?,
            None => writeln!(w, "{}", "Received".dark_grey())?,
        }

        Ok(Answer::String(ans))
    }
}

pub struct EditorBuilder<'m, 'w, 'f, 'v, 't> {
    opts: Options<'m, 'w>,
    editor: Editor<'f, 'v, 't>,
}

impl<'m, 'w, 'f, 'v, 't> EditorBuilder<'m, 'w, 'f, 'v, 't> {
    pub fn default<I: Into<String>>(mut self, default: I) -> Self {
        self.editor.default = Some(default.into());
        self
    }

    pub fn postfix<I: Into<String>>(mut self, postfix: I) -> Self {
        self.editor.postfix = Some(postfix.into());
        self
    }

    pub fn build(self) -> super::Question<'m, 'w, 'f, 'v, 't> {
        super::Question::new(self.opts, super::QuestionKind::Editor(self.editor))
    }
}

impl<'m, 'w, 'f, 'v, 't> From<EditorBuilder<'m, 'w, 'f, 'v, 't>>
    for super::Question<'m, 'w, 'f, 'v, 't>
{
    fn from(builder: EditorBuilder<'m, 'w, 'f, 'v, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(EditorBuilder<'f, 'v, 't>; (this, opts) => {
    EditorBuilder {
        opts,
        editor: this.editor,
    }
});

crate::impl_filter_builder!(EditorBuilder<'m, 'w, f, 'v, 't> String; (this, filter) => {
    EditorBuilder {
        opts: this.opts,
        editor: Editor {
            filter,
            editor: this.editor.editor,
            postfix: this.editor.postfix,
            default: this.editor.default,
            validate: this.editor.validate,
            transformer: this.editor.transformer,
        }
    }
});
crate::impl_validate_builder!(EditorBuilder<'m, 'w, 'f, v, 't> str; (this, validate) => {
    EditorBuilder {
        opts: this.opts,
        editor: Editor {
            validate,
            editor: this.editor.editor,
            postfix: this.editor.postfix,
            default: this.editor.default,
            filter: this.editor.filter,
            transformer: this.editor.transformer,
        }
    }
});
crate::impl_transformer_builder!(EditorBuilder<'m, 'w, 'f, 'v, t> str; (this, transformer) => {
    EditorBuilder {
        opts: this.opts,
        editor: Editor {
            transformer,
            editor: this.editor.editor,
            postfix: this.editor.postfix,
            validate: this.editor.validate,
            default: this.editor.default,
            filter: this.editor.filter,
        }
    }
});

impl super::Question<'static, 'static, 'static, 'static, 'static> {
    pub fn editor<N: Into<String>>(
        name: N,
    ) -> EditorBuilder<'static, 'static, 'static, 'static, 'static> {
        EditorBuilder {
            opts: Options::new(name.into()),
            editor: Default::default(),
        }
    }
}
