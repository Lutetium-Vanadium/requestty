use super::*;

macro_rules! test_numbers {
    (mod $mod_name:ident { $prompt_name:ident, $default:expr }) => {
        #[cfg(test)]
        mod $mod_name {
            use ui::{backend::TestBackend, layout::Layout};

            use super::*;

            #[test]
            fn test_render() {
                let size = (50, 20).into();
                let base_layout = Layout::new(5, size);
                let answers = Answers::default();

                let defaults = [(None, 17), (Some($default), 20)];

                let mut backend = TestBackend::new_with_layout(size, base_layout);

                for &(default, line_offset) in defaults.iter() {
                    let mut prompt = $prompt_name {
                        default: default.map(|n| (n, n.to_string())),
                        ..Default::default()
                    }
                    .into_prompt("message", &answers);

                    let base_name = default.map(|_| "default").unwrap_or("no_default");

                    let mut layout = base_layout;
                    backend.reset_with_layout(layout);
                    assert!(prompt.render(&mut layout, &mut backend).is_ok());
                    assert_eq!(layout, base_layout.with_line_offset(line_offset));
                    ui::assert_backend_snapshot!(format!("{}-1", base_name), backend);

                    prompt.input.set_value("3".repeat(50));

                    layout = base_layout;
                    backend.reset_with_layout(layout);
                    assert!(prompt.render(&mut layout, &mut backend).is_ok());
                    assert_eq!(layout, base_layout.with_offset(0, 1).with_line_offset(17));
                    ui::assert_backend_snapshot!(format!("{}-2", base_name), backend);
                }
            }

            #[test]
            fn test_height() {
                let size = (50, 20).into();
                let base_layout = Layout::new(5, size);
                let answers = Answers::default();

                let defaults = [(None, 17), (Some($default), 20)];

                for &(default, line_offset) in defaults.iter() {
                    let mut prompt = $prompt_name {
                        default: default.map(|n| (n, n.to_string())),
                        ..Default::default()
                    }
                    .into_prompt("message", &answers);

                    let mut layout = base_layout;

                    assert_eq!(prompt.height(&mut layout), 1);
                    assert_eq!(layout, base_layout.with_line_offset(line_offset));
                    layout = base_layout;

                    prompt.input.set_value("3".repeat(50));
                    assert_eq!(prompt.height(&mut layout), 2);
                    assert_eq!(layout, base_layout.with_offset(0, 1).with_line_offset(17));
                }
            }

            #[test]
            fn test_cursor_pos() {
                let size = (50, 20).into();
                let layout = Layout::new(5, size);
                let answers = Answers::default();

                let defaults = [None, Some($default)];

                for &default in defaults.iter() {
                    let mut prompt = $prompt_name {
                        default: default.map(|n| (n, n.to_string())),
                        ..Default::default()
                    }
                    .into_prompt("message", &answers);

                    assert_eq!(prompt.cursor_pos(layout), (17, 0));

                    prompt.input.set_value("3".repeat(50));
                    prompt.input.set_at(50);
                    assert_eq!(prompt.cursor_pos(layout), (17, 1));
                }
            }
        }
    };
}

test_numbers!(mod int { Int, 333 });
test_numbers!(mod float { Float, 3.3 });
