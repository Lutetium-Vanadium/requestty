# Changelog

## `0.6.1`
Fix doc issues

## `0.6.0`

The msrv has been bumped up to `1.78`

- `requestty`

  - Update `MultiSelect` to show spaces instead of gray tick marks for
    unselected items (#24).

- `requestty-ui`

  - Update crossterm and termion dependencies>

  - Split `Backend` trait into `DisplayBackend` `Backend` to support
    different trait requirements for the `termion` backend.

## `0.5.0`

- `requestty`

  - Update the way indices are shown in `RawSelect`

  - Added `OrderSelect` (#16)

  - [bug fix] Support multi-word editor commands (#14)

- `requestty-ui`

  - [bug fix] Add support for rendering wide characters. (#18, #19 and
    #20)

  - [bug fix] Show message when prompt's height exceeds the terminal
    height. Earlier, it would panic in debug builds due to overflow.
    (#17)

## `0.4.1`

- `requestty-ui`

  - Remove `dbg!` in `Input`. Fixes #12

## `0.4.0`

The msrv has been bumped up to `1.56`

- `requestty`

  - Allow programmatic customisation of `Question::editor`

  - Update `smallvec` version.

- `requestty-ui`

  - Allow customising the symbol set used during rendering.

  - Return error on 0 sized terminal instead of panicking.

  - Update `crossterm` version.

## `0.3.0`

- `requestty`

  - Allow using the Right Arrow key to auto-complete default

  - Implement #6 - Add support for handling abort with `Esc`

    Earlier, when the user pressed the `Esc` key, nothing would happen as
    `ui::Input` would just pass the event on to the prompt. Now 2 other
    actions can be specified.

    `OnEsc::Terminate`: returns an `Err` which will propagate upwards,
    essentially cancelling the `PromptModule`

    `OnEsc::SkipQuestion`: returns `None`, showing that the question has
    been skipped

  - Fix #7 - input returns empty string even if default was given

- `requestty-ui`

  - Added `OnEsc` to configure behaviour on `Esc` for `Input`s

  - Added the `skipped` parameter to `Prompt::write_finished_message`

  - Added `ErrorKind::Aborted`

  - Removed `ErrorKind::map_terminated` - this was a temporary code that
    came about while implementing a feature, but was never deleted.

  - Removed `StringInput::has_value` - this was used to get the capacity
    of the underlying string buffer due to a weird implementation of
    `Question::input`

  - Changed `StringInput::finish` to return `String` instead of
    `Option<String>` - it used `has_value` to choose `Some` or `None`.
    Again, due to a weird implementation of `Question::input`

## `0.2.1`

- `requestty`

  - Implement #4 - defaults are now shown in a different way for the
    `input`, `int` and `float` prompts.

    Earlier, the default value would just be shown on the side at all
    times. This is even if the default will not be selected which can be
    misleading. This change shows the default as greyed out text in the
    input itself. It also allows pressing 'Tab' to make the current
    input the default if the current input value is the start of the
    default.

  - Added the `validate_on_key` option for `input`, `int`, `float` and
    `password` prompts.

    `validate_on_key` if supplied will be called on every change of
    input. If validation fails, the input text is displayed in red.

    > `validate` still needs to be supplied as `validate_on_key` is
    > purely cosmetic, and does **not** prevent user submission

## `0.1.3`

- `requestty`

  - Fix #3

- `requestty-ui`
  - Update crossterm dependency

## `0.1.2`

- `requestty`

  - Fix #2

- `requestty-ui`
  - Change `Widget::cursor_pos` to return the position relative to the
    screen instead of the start of the root widget
  - Update dependencies
