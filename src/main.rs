// TODO: delete
// this is a temporary file, for testing out the prompts
use inquisition::{DefaultSeparator, Question, Separator};
use std::{array::IntoIter, env};

fn main() {
    let q = match env::args().nth(1).as_deref() {
        Some("b") => vec![
            Question::confirm("a").message("Hello there 1").build(),
            Question::confirm("b")
                .message("Hello there 2")
                .default(true)
                .build(),
        ],
        Some("s") => vec![
            Question::input("a").message("Hello there 1").into(),
            Question::input("b")
                .message("Hello there 2")
                .default("Yes")
                .into(),
        ],
        Some("p") => vec![
            Question::password("a")
                .message("password 1")
                .mask('*')
                .into(),
            Question::password("b").message("password 2").into(),
        ],
        Some("i") => vec![
            Question::int("a").message("int 1").into(),
            Question::int("b").message("int 2").default(3).into(),
        ],
        Some("f") => vec![
            Question::float("a").message("float 1").into(),
            Question::float("b").message("float 2").default(3.12).into(),
        ],
        Some("e") => vec![
            Question::editor("a")
                .message("editor 1")
                .default("Hello there")
                .into(),
            Question::editor("b")
                .message("editor 2")
                .extension(".rs")
                .into(),
        ],

        Some("l") => vec![
            Question::list("a")
                .message("list 1")
                .choices(IntoIter::new([
                    "0".into(),
                    DefaultSeparator,
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator("== Hello separator".into()),
                ]))
                .default(3)
                .into(),
            Question::list("b")
                .message("list 2")
                .choices(IntoIter::new([
                    Separator("=== TITLE BOI ===".into()),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                    DefaultSeparator,
                    "hello worldssssss 6".into(),
                    "hello worldssssss 7".into(),
                    "hello worldssssss 8".into(),
                ]))
                .page_size(6)
                .into(),
        ],

        Some("c") => vec![
            Question::checkbox("a")
                .message("checkbox 1")
                .choice_with_default("0", true)
                .default_separator()
                .choices_with_default(IntoIter::new([
                    ("1".into(), false),
                    ("2".into(), true),
                    ("3".into(), false),
                ]))
                .separator("== Hello separator")
                .into(),
            Question::checkbox("b")
                .message("checkbox 2")
                .choices(IntoIter::new([
                    Separator("=== TITLE BOI ===".into()),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                    DefaultSeparator,
                    "hello worldssssss 6".into(),
                    "hello worldssssss 7".into(),
                    "hello worldssssss 8".into(),
                ]))
                .page_size(6)
                .should_loop(false)
                .into(),
        ],

        Some("r") => vec![
            Question::rawlist("a")
                .message("list 1")
                .choices(IntoIter::new([
                    "0".into(),
                    DefaultSeparator,
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator("== Hello separator".into()),
                ]))
                .default(2)
                .into(),
            Question::rawlist("b")
                .message("list 2")
                .choices(IntoIter::new([
                    Separator("=== TITLE BOI ===".into()),
                    "hello worldssssss 1".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                    DefaultSeparator,
                    "hello worldssssss 6".into(),
                    "hello worldssssss 7".into(),
                    "hello worldssssss 8".into(),
                ]))
                .page_size(6)
                // .should_loop(false)
                .into(),
        ],

        Some("x") => vec![
            Question::expand("a")
                .message("expand 1")
                .choices(IntoIter::new([
                    ('y', "Overwrite").into(),
                    ('a', "Overwrite this one and all next").into(),
                    ('d', "Show diff").into(),
                    DefaultSeparator,
                    ('x', "Abort").into(),
                ]))
                .into(),
            Question::expand("b")
                .message("expand 2")
                .choice('a', "Name for a")
                .default_separator()
                .choices(IntoIter::new([('b', "Name for b"), ('c', "Name for c")]))
                .default_separator()
                .choice('d', "Name for d")
                .separator("== Hello separator")
                .default('b')
                .into(),
        ],
        _ => panic!("no arg"),
    };

    println!("{:#?}", inquisition::prompt(q));
}
