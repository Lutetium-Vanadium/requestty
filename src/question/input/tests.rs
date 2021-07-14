use ui::{backend::TestBackend, layout::Layout};

use super::*;

const NINPUTS: usize = 3;
static INPUT_IDS: [&str; NINPUTS] = ["no_default", "default", "auto_complete"];
const AUTO_COMPLETE_IDX: usize = 2;

fn inputs(answers: &Answers) -> [(InputPrompt<'static, '_>, u16); NINPUTS] {
    [
        (Input::default().into_input_prompt("message", &answers), 17),
        (
            Input {
                default: Some("default".into()),
                ..Input::default()
            }
            .into_input_prompt("message", &answers),
            25,
        ),
        (
            Input {
                auto_complete: AutoComplete::Sync(Box::new(|s, _| {
                    let mut completions: Completions<_> = ('a'..='d')
                        .map(|c| {
                            let mut s = s.clone();
                            s.push(c);
                            s
                        })
                        .collect();
                    completions.push(s + "e");
                    completions
                })),
                ..Input::default()
            }
            .into_input_prompt("message", &answers),
            17,
        ),
    ]
}

#[test]
fn test_render() {
    let size = (50, 20).into();
    let base_layout = Layout::new(5, size);
    let answers = Answers::default();

    let mut inputs = inputs(&answers);
    let mut backend = TestBackend::new_with_layout(size, base_layout);

    for (i, (prompt, line_offset)) in inputs.iter_mut().enumerate() {
        let line_offset = *line_offset;

        let mut layout = base_layout;
        backend.reset_with_layout(layout);
        assert!(prompt.render(&mut layout, &mut backend).is_ok());
        assert_eq!(layout, base_layout.with_line_offset(line_offset));
        ui::assert_backend_snapshot!(format!("{}-1", INPUT_IDS[i]), backend);

        prompt.input.set_value("input".repeat(10));

        layout = base_layout;
        backend.reset_with_layout(layout);
        assert!(prompt.render(&mut layout, &mut backend).is_ok());
        assert_eq!(
            layout,
            base_layout.with_offset(0, 1).with_line_offset(line_offset)
        );
        ui::assert_backend_snapshot!(format!("{}-2", INPUT_IDS[i]), backend);
    }

    let prompt = &mut inputs[AUTO_COMPLETE_IDX].0;

    prompt.input.replace_with(|mut s| {
        s.truncate(5);
        s
    });
    assert!(prompt.handle_key(KeyCode::Tab.into()));

    let mut layout = base_layout;
    backend.reset_with_layout(layout);
    assert!(prompt.render(&mut layout, &mut backend).is_ok());
    assert_eq!(layout, base_layout.with_offset(0, 6).with_line_offset(0));
    ui::assert_backend_snapshot!(format!("{}-3", INPUT_IDS[AUTO_COMPLETE_IDX]), backend);

    assert!(prompt.handle_key(KeyCode::Tab.into()));
    assert_eq!(prompt.validate(), Ok(Validation::Continue));

    layout = base_layout;
    backend.reset_with_layout(layout);
    assert!(prompt.render(&mut layout, &mut backend).is_ok());
    assert_eq!(
        layout,
        base_layout
            .with_offset(0, 0)
            .with_line_offset(inputs[AUTO_COMPLETE_IDX].1 + 6)
    );
    ui::assert_backend_snapshot!(format!("{}-4", INPUT_IDS[AUTO_COMPLETE_IDX]), backend);
}

#[test]
fn test_height() {
    let size = (50, 20).into();
    let base_layout = Layout::new(5, size);
    let answers = Answers::default();

    let mut inputs = inputs(&answers);

    for (prompt, line_offset) in inputs.iter_mut() {
        let line_offset = *line_offset;

        let mut layout = base_layout;
        assert_eq!(prompt.height(&mut layout), 1);
        assert_eq!(layout, base_layout.with_line_offset(line_offset));

        prompt.input.set_value("input".repeat(10));

        layout = base_layout;
        assert_eq!(prompt.height(&mut layout), 2);
        assert_eq!(
            layout,
            base_layout.with_offset(0, 1).with_line_offset(line_offset)
        );
    }

    let prompt = &mut inputs[AUTO_COMPLETE_IDX].0;

    prompt.input.replace_with(|mut s| {
        s.truncate(5);
        s
    });
    assert!(prompt.handle_key(KeyCode::Tab.into()));

    let mut layout = base_layout;
    assert_eq!(prompt.height(&mut layout), 6);
    assert_eq!(layout, base_layout.with_offset(0, 6).with_line_offset(0));

    assert!(prompt.handle_key(KeyCode::Tab.into()));
    assert_eq!(prompt.validate(), Ok(Validation::Continue));

    layout = base_layout;
    assert_eq!(prompt.height(&mut layout), 1);
    assert_eq!(
        layout,
        base_layout
            .with_offset(0, 0)
            .with_line_offset(inputs[AUTO_COMPLETE_IDX].1 + 6)
    );
}

#[test]
fn test_cursor_pos() {
    let size = (50, 20).into();
    let layout = Layout::new(5, size);
    let answers = Answers::default();

    let mut inputs = inputs(&answers);

    for (prompt, line_offset) in inputs.iter_mut() {
        let line_offset = *line_offset;

        assert_eq!(prompt.cursor_pos(layout), (line_offset, 0));
        prompt.input.set_value("input".repeat(10));
        prompt.input.set_at(50);
        assert_eq!(prompt.cursor_pos(layout), (line_offset, 1));
    }

    let prompt = &mut inputs[AUTO_COMPLETE_IDX].0;
    let line_offset = inputs[AUTO_COMPLETE_IDX].1;

    prompt.input.replace_with(|mut s| {
        s.truncate(5);
        s
    });
    assert!(prompt.handle_key(KeyCode::Tab.into()));

    assert_eq!(prompt.cursor_pos(layout), (line_offset + 6, 0));

    assert!(prompt.handle_key(KeyCode::Tab.into()));
    assert_eq!(prompt.validate(), Ok(Validation::Continue));

    assert_eq!(prompt.cursor_pos(layout), (line_offset + 6, 0));
}
