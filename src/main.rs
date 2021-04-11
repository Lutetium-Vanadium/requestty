// TODO: delete
// this is a temporary file, for testing out the prompts
use inquisition::{Choice::Separator, Question};
use std::{array::IntoIter, env, io};

fn main() {
    let (a, b) = match env::args().nth(1).as_deref() {
        Some("b") => (
            Question::confirm("a").message("Hello there 1").build(),
            Question::confirm("b")
                // .message("Hello there 2")
                .default(true)
                .build(),
        ),
        Some("s") => (
            Question::input("a").message("Hello there 1").into(),
            Question::input("b")
                .message("Hello there 2")
                .default("Yes")
                .into(),
        ),
        Some("p") => (
            Question::password("a")
                .message("password 1")
                .mask('*')
                .into(),
            Question::password("b").message("password 2").into(),
        ),
        Some("i") => (
            Question::int("a").message("int 1").into(),
            Question::int("b").message("int 2").default(3).into(),
        ),
        Some("f") => (
            Question::float("a").message("float 1").into(),
            Question::float("b").message("float 2").default(3.12).into(),
        ),
        Some("e") => (
            Question::editor("a").message("editor 1").into(),
            Question::editor("b").message("editor 2").into(),
        ),

        Some("l") => (
            Question::list("a")
                .message("list 1")
                .choices(IntoIter::new([
                    Separator(Some("=== TITLE BOI ===".into())),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                ]))
                .into(),
            Question::list("b")
                .message("list 2")
                .choices(IntoIter::new([
                    "0".into(),
                    Separator(None),
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator(Some("== Hello separator".into())),
                ]))
                .default(3)
                .into(),
        ),

        Some("c") => (
            Question::checkbox("a")
                .message("checkbox 1")
                .choices(vec![
                    Separator(Some("=== TITLE BOI ===".into())),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                ])
                .into(),
            Question::checkbox("b")
                .message("checkbox 2")
                .choice_with_default("0", true)
                .default_separator()
                .choices_with_default(IntoIter::new([
                    ("1".into(), false),
                    ("2".into(), true),
                    ("3".into(), false),
                ]))
                .separator("== Hello separator")
                .into(),
        ),

        Some("r") => (
            Question::rawlist("a")
                .message("list 1")
                .choices(IntoIter::new([
                    Separator(Some("=== TITLE BOI ===".into())),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                ]))
                .into(),
            Question::rawlist("b")
                .message("list 2")
                .choices(IntoIter::new([
                    "0".into(),
                    Separator(None),
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator(Some("== Hello separator".into())),
                ]))
                .default(2)
                .into(),
        ),

        Some("x") => (
            Question::expand("a")
                .message("expand 1")
                .choices(IntoIter::new([
                    ('y', "Overwrite").into(),
                    ('a', "Overwrite this one and all next").into(),
                    ('d', "Show diff").into(),
                    Separator(None),
                    ('x', "Abort").into(),
                ]))
                .into(),
            Question::expand("b")
                .message("expand 2")
                .choices(IntoIter::new([
                    ('a', "Name for a").into(),
                    Separator(None),
                    ('b', "Name for b").into(),
                    ('c', "Name for c").into(),
                    Separator(None),
                    ('d', "Name for d").into(),
                    Separator(Some("== Hello separator".into())),
                ]))
                .default('b')
                .into(),
        ),
        _ => panic!("no arg"),
    };

    let mut stdout = io::stdout();
    println!("{:?}", a.ask(&mut stdout));
    println!("{:?}", b.ask(&mut stdout));
}
