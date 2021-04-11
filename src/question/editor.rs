use std::{
    env,
    ffi::OsString,
    io::{self, Read},
    process::Command,
};

use crossterm::style::Colorize;
use ui::Widget;

use crate::{error, Answer};

use super::Options;

#[derive(Debug, Default)]
pub struct Editor {
    // TODO: What is this??
    postfix: (),
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
    message: String,
    editor: Editor,
}

impl Widget for EditorPrompt {
    fn render<W: io::Write>(&mut self, _: usize, _: &mut W) -> crossterm::Result<()> {
        Ok(())
    }

    fn height(&self) -> usize {
        0
    }
}

impl ui::Prompt for EditorPrompt {
    type ValidateErr = &'static str;
    type Output = Editor;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some("Press <enter> to launch your preferred editor.")
    }

    fn finish(self) -> Self::Output {
        self.editor
    }

    fn has_default(&self) -> bool {
        false
    }
}

impl Editor {
    pub fn ask<W: io::Write>(self, message: String, w: &mut W) -> error::Result<Answer> {
        ui::Input::new(EditorPrompt {
            message,
            editor: self,
        })
        .run(w)?;

        let mut file = tempfile::NamedTempFile::new()?;

        // FIXME: handle error
        assert!(Command::new(get_editor())
            .arg(file.path())
            .status()?
            .success());

        let mut ans = String::new();
        file.read_to_string(&mut ans)?;

        writeln!(w, "{}", "Received".dark_grey())?;

        Ok(Answer::String(ans))
    }
}

pub struct EditorBuilder<'m, 'w> {
    opts: Options<'m, 'w>,
    editor: Editor,
}

impl<'m, 'w> EditorBuilder<'m, 'w> {
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
