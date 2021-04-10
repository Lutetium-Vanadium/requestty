use std::{
    env,
    ffi::OsString,
    io::{self, Read},
    process::Command,
};

use crossterm::style::Colorize;
use ui::Widget;

use crate::{error, Answer};

use super::{Options, Question, QuestionKind};

pub struct Editor {
    // FIXME: What is correct type here?
    default: String,
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
    opts: Options,
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
    type Output = ();

    fn prompt(&self) -> &str {
        &self.opts.message
    }

    fn hint(&self) -> Option<&str> {
        Some("Press <enter> to launch your preferred editor.")
    }

    fn finish(self) -> Self::Output {}

    fn finish_default(self) -> Self::Output {}
}

impl Editor {
    pub fn ask<W: io::Write>(self, opts: Options, w: &mut W) -> error::Result<Answer> {
        ui::Input::new(EditorPrompt { opts }).run(w)?;

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

impl Question {
    pub fn editor(name: String, message: String) -> Self {
        Self::new(
            name,
            message,
            QuestionKind::Editor(Editor {
                default: String::new(),
                postfix: (),
            }),
        )
    }
}
