use std::{
    env,
    ffi::OsString,
    io::{self, Read, Seek, SeekFrom, Write},
    process::Command,
};

use crossterm::style::Colorize;
use tempfile::NamedTempFile;
use ui::{Validation, Widget};

use crate::{error, Answer};

use super::Options;

#[derive(Debug)]
pub struct Editor {
    postfix: Option<String>,
    default: Option<String>,
    editor: OsString,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            editor: get_editor(),
            postfix: None,
            default: None,
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

struct EditorPrompt {
    file: NamedTempFile,
    ans: String,
    message: String,
    editor: Editor,
}

impl Widget for EditorPrompt {
    fn render<W: Write>(&mut self, _: usize, _: &mut W) -> crossterm::Result<()> {
        Ok(())
    }

    fn height(&self) -> usize {
        0
    }
}

impl ui::Prompt for EditorPrompt {
    type ValidateErr = io::Error;
    type Output = String;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some("Press <enter> to launch your preferred editor.")
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        // FIXME: handle error
        assert!(Command::new(&self.editor.editor)
            .arg(self.file.path())
            .status()?
            .success());

        self.file.read_to_string(&mut self.ans)?;
        self.file.seek(SeekFrom::Start(0))?;

        // TODO: accept validation from library caller

        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        self.ans
    }

    fn has_default(&self) -> bool {
        false
    }
}

impl Editor {
    pub fn ask<W: Write>(self, message: String, w: &mut W) -> error::Result<Answer> {
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

        let ans = ui::Input::new(EditorPrompt {
            message,
            editor: self,
            file,
            ans: String::new(),
        })
        .run(w)?;

        writeln!(w, "{}", "Received".dark_grey())?;

        Ok(Answer::String(ans))
    }
}

pub struct EditorBuilder<'m, 'w> {
    opts: Options<'m, 'w>,
    editor: Editor,
}

impl<'m, 'w> EditorBuilder<'m, 'w> {
    pub fn default<I: Into<String>>(mut self, default: I) -> Self {
        self.editor.default = Some(default.into());
        self
    }

    pub fn postfix<I: Into<String>>(mut self, postfix: I) -> Self {
        self.editor.postfix = Some(postfix.into());
        self
    }

    pub fn build(self) -> super::Question<'m, 'w> {
        super::Question::new(self.opts, super::QuestionKind::Editor(self.editor))
    }
}

impl<'m, 'w> From<EditorBuilder<'m, 'w>> for super::Question<'m, 'w> {
    fn from(builder: EditorBuilder<'m, 'w>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(EditorBuilder; (this, opts) => {
    EditorBuilder {
        opts,
        editor: this.editor,
    }
});

impl super::Question<'static, 'static> {
    pub fn editor<N: Into<String>>(name: N) -> EditorBuilder<'static, 'static> {
        EditorBuilder {
            opts: Options::new(name.into()),
            editor: Default::default(),
        }
    }
}
