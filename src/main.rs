// TODO: delete
// this is a temporary file, for testing out the prompts
use requestty::{DefaultSeparator, Question, Separator};
use std::env;

fn main() {
    let s = String::from("Hello there ");

    let q = match env::args().nth(1).as_deref() {
        Some("b") => vec![
            Question::confirm("a").message("Hello there 1").build(),
            Question::confirm("b")
                .message("Hello there 2")
                .default(true)
                .build(),
        ],
        Some("s") => vec![
            Question::input("a").message("Hello there 2").into(),
            Question::input("b")
                .message(|_: &requestty::Answers| s[0..(s.len() - 1)].to_owned())
                .filter(|mut ans, _| {
                    ans.insert_str(0, &s);
                    ans
                })
                .validate(|_, _| Ok(()))
                .default("Yes")
                .into(),
        ],
        Some("p") => vec![
            Question::password("a")
                .message("password 1")
                .filter(|ans, _| ans + &s)
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
            Question::select("a")
                .message("select 1")
                .choices(vec![
                    "0".into(),
                    DefaultSeparator,
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator("== Hello separator".into()),
                ])
                .default(3)
                .into(),
            Question::select("b")
                .message("select 2")
                .choices(vec![
                    Separator("=== TITLE BOI ===".into()),
                    "hello worldssssss 1\nMulti-line description about it".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                    DefaultSeparator,
                    "hello worldssssss 6".into(),
                    "hello worldssssss 7".into(),
                    "hello worldssssss 8".into(),
                ])
                .page_size(6)
                .should_loop(false)
                .into(),
        ],

        Some("c") => vec![
            Question::multi_select("a")
                .message("multi select 1")
                .choice_with_default("0", true)
                .default_separator()
                .choices_with_default(vec![("1", false), ("2", true), ("3", false)])
                .separator("== Hello separator")
                .into(),
            Question::multi_select("b")
                .message("multi select 2")
                .choices(vec![
                    Separator("=== TITLE BOI ===".into()),
                    "hello worldssssss 1\nMulti-line description about it".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                    DefaultSeparator,
                    "hello worldssssss 6".into(),
                    "hello worldssssss 7".into(),
                    "hello worldssssss 8".into(),
                ])
                .page_size(6)
                .should_loop(false)
                .into(),
        ],

        Some("r") => vec![
            Question::raw_select("a")
                .message("select 1")
                .choices(vec![
                    "0".into(),
                    DefaultSeparator,
                    "1".into(),
                    "2".into(),
                    "3".into(),
                    Separator("== Hello separator".into()),
                ])
                .default(2)
                .into(),
            Question::raw_select("b")
                .message("select 2")
                .choices(vec![
                    Separator("=== TITLE BOI ===".into()),
                    "hello worldssssss 1\nMulti-line description about it".into(),
                    "hello worldssssss 2".into(),
                    "hello worldssssss 3".into(),
                    "hello worldssssss 4".into(),
                    "hello worldssssss 5".into(),
                    DefaultSeparator,
                    "hello worldssssss 6".into(),
                    "hello worldssssss 7".into(),
                    "hello worldssssss 8".into(),
                ])
                .page_size(6)
                // .should_loop(false)
                .into(),
        ],

        Some("x") => vec![
            Question::expand("a")
                .message("expand 1")
                .choices(vec![
                    ('y', "Overwrite").into(),
                    ('a', "Overwrite this one and all next").into(),
                    ('d', "Show diff").into(),
                    DefaultSeparator,
                    ('x', "Abort").into(),
                ])
                .into(),
            Question::expand("b")
                .message("expand 2")
                .choice('a', "Name for a")
                .default_separator()
                .choices(vec![
                    ('b', "Name for b\nMulti-line description"),
                    ('c', "Name for c"),
                ])
                .default_separator()
                .choice('d', "Name for d")
                .separator("== Hello separator")
                .default('b')
                .into(),
        ],
        _ => panic!("no arg"),
    };

    println!("{:#?}", requestty::prompt(q));
}
