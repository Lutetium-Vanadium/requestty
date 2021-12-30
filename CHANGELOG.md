# Changelog

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
