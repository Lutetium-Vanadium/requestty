// TODO: delete
// this is a temporary file, for testing out the prompts
use inquisition::{Choice::Separator, ExpandItem, Question};
use std::{env, io};

fn main() {
    let (a, b) = match env::args().nth(1).as_deref() {
        Some("b") => (
            Question::confirm("a".into(), "Hello there 1".into(), true),
            Question::confirm("b".into(), "Hello there 2".into(), false),
        ),
        Some("s") => (
            Question::input("a".into(), "Hello there 1".into(), "No".into()),
            Question::input("b".into(), "Hello there 2".into(), "Yes".into()),
        ),
        Some("p") => (
            Question::password("a".into(), "password 1".into()).with_mask('*'),
            Question::password("b".into(), "password 2".into()),
        ),
        Some("i") => (
            Question::int("a".into(), "int 1".into(), 0),
            Question::int("b".into(), "int 2".into(), 3),
        ),
        Some("f") => (
            Question::float("a".into(), "float 1".into(), 0.123),
            Question::float("b".into(), "float 2".into(), 3.12),
        ),
        Some("e") => (
            Question::editor("a".into(), "editor 1".into()),
            Question::editor("b".into(), "editor 2".into()),
        ),

        Some("l") => (
            Question::list(
                "a".into(),
                "list 1".into(),
                vec![
                    Separator(Some("=== TITLE BOI ===".into())),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                ],
                0,
            ),
            Question::list(
                "b".into(),
                "list 2".into(),
                vec![
                    "0".into(),
                    Separator(None),
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator(Some("== Hello separator".into())),
                ],
                0,
            ),
        ),

        Some("c") => (
            Question::checkbox(
                "a".into(),
                "checkbox 1".into(),
                vec![
                    Separator(Some("=== TITLE BOI ===".into())),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                ],
            ),
            Question::checkbox(
                "b".into(),
                "checkbox 2".into(),
                vec![
                    "0".into(),
                    Separator(None),
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator(Some("== Hello separator".into())),
                ],
            ),
        ),

        Some("r") => (
            Question::raw_list(
                "a".into(),
                "list 1".into(),
                vec![
                    Separator(Some("=== TITLE BOI ===".into())),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                ],
                0,
            ),
            Question::raw_list(
                "b".into(),
                "list 2".into(),
                vec![
                    "0".into(),
                    Separator(None),
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator(Some("== Hello separator".into())),
                ],
                0,
            ),
        ),

        Some("x") => (
            Question::expand(
                "a".into(),
                "expand 1".into(),
                vec![
                    ExpandItem {
                        key: 'y',
                        name: "Overwrite".into(),
                    }
                    .into(),
                    ExpandItem {
                        key: 'a',
                        name: "Overwrite this one and all next".into(),
                    }
                    .into(),
                    ExpandItem {
                        key: 'd',
                        name: "Show diff".into(),
                    }
                    .into(),
                    Separator(None),
                    ExpandItem {
                        key: 'x',
                        name: "Abort".into(),
                    }
                    .into(),
                ],
                None,
            ),
            Question::expand(
                "b".into(),
                "expand 2".into(),
                vec![
                    ExpandItem {
                        key: 'a',
                        name: "Name for a".into(),
                    }
                    .into(),
                    Separator(None),
                    ExpandItem {
                        key: 'b',
                        name: "Name for b".into(),
                    }
                    .into(),
                    ExpandItem {
                        key: 'c',
                        name: "Name for c".into(),
                    }
                    .into(),
                    Separator(None),
                    ExpandItem {
                        key: 'd',
                        name: "Name for d".into(),
                    }
                    .into(),
                    Separator(Some("== Hello separator".into())),
                ],
                Some('b'),
            ),
        ),
        _ => panic!("no arg"),
    };

    let mut stdout = io::stdout();
    println!("{:?}", a.ask(&mut stdout));
    println!("{:?}", b.ask(&mut stdout));
}
