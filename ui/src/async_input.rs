use std::io;

use async_trait::async_trait;
use futures::StreamExt;

use super::{Input, Prompt, Validation};
use crate::{
    backend::Backend,
    error,
    events::{AsyncEvents, KeyCode, KeyModifiers},
};

/// This trait should be implemented by all 'root' widgets.
///
/// It provides the functionality specifically required only by the main controlling widget. For the
/// trait required for general rendering to terminal, see [`Widget`].
#[async_trait]
pub trait AsyncPrompt: Prompt {
    /// Try to validate the prompt state is valid synchronously without blocking. If it can't be
    /// done without blocking, return `None` and [`validate_async`](AsyncPrompt::validate_async)
    /// will be called instead. It is called whenever the use presses the enter key.
    fn try_validate_sync(&mut self) -> Option<Result<Validation, Self::ValidateErr>> {
        None
    }

    /// Determine whether the prompt state is ready to be submitted. It is called whenever the use
    /// presses the enter key.
    #[allow(unused_variables)]
    async fn validate_async(&mut self) -> Result<Validation, Self::ValidateErr> {
        Ok(Validation::Finish)
    }

    /// The value to return from [`Input::run_async`]. This will only be called once validation returns
    /// [`Validation::Finish`];
    async fn finish_async(self) -> Self::Output;
}

impl<P: AsyncPrompt + Send, B: Backend + Unpin> Input<P, B> {
    #[inline]
    async fn finish_async(
        mut self,
        pressed_enter: bool,
        prompt_len: u16,
    ) -> error::Result<P::Output> {
        self.clear(prompt_len)?;
        self.reset_terminal()?;

        if pressed_enter {
            Ok(self.prompt.finish_async().await)
        } else {
            Ok(self.prompt.finish_default())
        }
    }

    /// Run the ui on the given writer. It will return when the user presses `Enter` or `Escape`
    /// based on the [`AsyncPrompt`] implementation.
    pub async fn run_async(mut self, events: &mut AsyncEvents) -> error::Result<P::Output> {
        let prompt_len = self.init()?;

        loop {
            let e = events.next().await.unwrap()?;
            let key_handled = match e.code {
                KeyCode::Char('c') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit()?;
                    return Err(io::Error::new(io::ErrorKind::Other, "CTRL+C").into());
                }
                KeyCode::Null => {
                    self.exit()?;
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF").into());
                }
                KeyCode::Esc if self.prompt.has_default() => {
                    return self.finish_async(false, prompt_len).await;
                }

                KeyCode::Enter => {
                    let result = match self.prompt.try_validate_sync() {
                        Some(res) => res,
                        None => self.prompt.validate_async().await,
                    };

                    match result {
                        Ok(Validation::Finish) => {
                            return self.finish_async(true, prompt_len).await;
                        }
                        Ok(Validation::Continue) => true,
                        Err(e) => {
                            self.print_error(e)?;
                            continue;
                        }
                    }
                }
                _ => self.prompt.handle_key(e),
            };

            if key_handled {
                self.size = self.backend.size()?;

                self.render()?;
            }
        }
    }
}
