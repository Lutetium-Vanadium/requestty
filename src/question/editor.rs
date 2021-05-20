use std::{
    env,
    ffi::OsString,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    process::Command,
};

use tempfile::TempPath;

#[cfg(feature = "async-std")]
use async_std_dep::{
    fs::File as AsyncFile,
    io::prelude::{ReadExt, SeekExt, WriteExt},
    process::Command as AsyncCommand,
};
#[cfg(feature = "smol")]
use smol_dep::{
    fs::File as AsyncFile,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    process::Command as AsyncCommand,
};
#[cfg(feature = "tokio")]
use tokio_dep::{
    fs::File as AsyncFile,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    process::Command as AsyncCommand,
};

use ui::{
    backend::{Backend, Stylize},
    error, Validation, Widget,
};

use super::{Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug)]
pub struct Editor<'f, 'v, 't> {
    extension: Option<String>,
    default: Option<String>,
    editor: OsString,
    filter: Filter<'f, String>,
    validate: Validate<'v, str>,
    transform: Transform<'t, str>,
}

impl Default for Editor<'static, 'static, 'static> {
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

struct EditorPrompt<'f, 'v, 't, 'a> {
    file: File,
    path: TempPath,
    ans: String,
    message: String,
    editor: Editor<'f, 'v, 't>,
    answers: &'a Answers,
}

impl Widget for EditorPrompt<'_, '_, '_, '_> {
    fn render<B: Backend>(&mut self, _: ui::Layout, _: &mut B) -> error::Result<()> {
        Ok(())
    }

    fn height(&mut self, _: ui::Layout) -> u16 {
        0
    }
}

impl ui::Prompt for EditorPrompt<'_, '_, '_, '_> {
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
            .arg(&self.path)
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

        if let Validate::Sync(ref validate) = self.editor.validate {
            validate(&self.ans, self.answers)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
        }

        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        match self.editor.filter {
            Filter::Sync(filter) => filter(self.ans, self.answers),
            _ => self.ans,
        }
    }

    fn has_default(&self) -> bool {
        false
    }
}

crate::cfg_async! {
struct EditorPromptAsync<'f, 'v, 't, 'a> {
    file: AsyncFile,
    path: TempPath,
    ans: String,
    message: String,
    editor: Editor<'f, 'v, 't>,
    answers: &'a Answers,
}

impl Widget for EditorPromptAsync<'_, '_, '_, '_> {
    fn render<B: Backend>(&mut self, _: ui::Layout, _: &mut B) -> error::Result<()> {
        Ok(())
    }

    fn height(&mut self, _: ui::Layout) -> u16 {
        0
    }
}

impl ui::Prompt for EditorPromptAsync<'_, '_, '_, '_> {
    type ValidateErr = io::Error;
    type Output = String;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some("Press <enter> to launch your preferred editor.")
    }

    fn finish(self) -> Self::Output {
        unimplemented!("EditorPromptAsync should only be called through async api");
    }

    fn has_default(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl ui::AsyncPrompt for EditorPromptAsync<'_, '_, '_, '_> {
    async fn finish_async(self) -> Self::Output {
        match self.editor.filter {
            Filter::Async(filter) => filter(self.ans, self.answers).await,
            Filter::Sync(filter) => filter(self.ans, self.answers),
            Filter::None => self.ans,
        }
    }

    async fn validate_async(&mut self) -> Result<Validation, Self::ValidateErr> {
        if !AsyncCommand::new(&self.editor.editor)
            .arg(&self.path)
            .status()
            .await?
            .success()
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not open editor",
            ));
        }

        self.ans.clear();
        self.file.read_to_string(&mut self.ans).await?;
        self.file.seek(SeekFrom::Start(0)).await?;

        match self.editor.validate {
            Validate::Async(ref validate) => validate(&self.ans, self.answers).await
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?,
            Validate::Sync(ref validate) => validate(&self.ans, self.answers)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?,
            Validate::None => {}

        }

        Ok(Validation::Finish)
    }
}
}

impl Editor<'_, '_, '_> {
    pub(crate) fn ask<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
    ) -> error::Result<Answer> {
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
                message,
                editor: self,
                file,
                path,
                ans: String::new(),
                answers,
            },
            b,
        )
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled("Received".dark_grey())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::String(ans))
    }

    crate::cfg_async! {
    pub(crate) async fn ask_async<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::AsyncEvents,
    ) -> error::Result<Answer> {
        let mut builder = tempfile::Builder::new();

        if let Some(ref extension) = self.extension {
            builder.suffix(extension);
        }

        let (file, path) = builder.tempfile()?.into_parts();
        let mut file = AsyncFile::from(file);

        if let Some(ref default) = self.default {
            file.write_all(default.as_bytes()).await?;
            file.seek(SeekFrom::Start(0)).await?;
            file.flush().await?;
        }

        let transform = self.transform.take();

        let ans = ui::Input::new(EditorPromptAsync {
            message,
            editor: self,
            file,
            path,
            ans: String::new(),
            answers,
        }, b)
        .run_async(events)
        .await?;

        match transform {
            Transform::Async(transform) => transform(&ans, answers, b).await?,
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            Transform::None => {
                b.write_styled("Received".dark_grey())?;
                b.write_all(b"\n")?;
                b.flush()?;
            },
        }

        Ok(Answer::String(ans))
    }
    }
}

pub struct EditorBuilder<'m, 'w, 'f, 'v, 't> {
    opts: Options<'m, 'w>,
    editor: Editor<'f, 'v, 't>,
}

impl EditorBuilder<'static, 'static, 'static, 'static, 'static> {
    pub(crate) fn new(name: String) -> Self {
        EditorBuilder {
            opts: Options::new(name),
            editor: Default::default(),
        }
    }
}

impl<'m, 'w, 'f, 'v, 't> EditorBuilder<'m, 'w, 'f, 'v, 't> {
    pub fn default<I: Into<String>>(mut self, default: I) -> Self {
        self.editor.default = Some(default.into());
        self
    }

    pub fn extension<I: Into<String>>(mut self, extension: I) -> Self {
        self.editor.extension = Some(extension.into());
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
            extension: this.editor.extension,
            default: this.editor.default,
            validate: this.editor.validate,
            transform: this.editor.transform,
        }
    }
});
crate::impl_validate_builder!(EditorBuilder<'m, 'w, 'f, v, 't> str; (this, validate) => {
    EditorBuilder {
        opts: this.opts,
        editor: Editor {
            validate,
            editor: this.editor.editor,
            extension: this.editor.extension,
            default: this.editor.default,
            filter: this.editor.filter,
            transform: this.editor.transform,
        }
    }
});
crate::impl_transform_builder!(EditorBuilder<'m, 'w, 'f, 'v, t> str; (this, transform) => {
    EditorBuilder {
        opts: this.opts,
        editor: Editor {
            transform,
            editor: this.editor.editor,
            extension: this.editor.extension,
            validate: this.editor.validate,
            default: this.editor.default,
            filter: this.editor.filter,
        }
    }
});
