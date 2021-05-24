fn main() {
    let question = inquisition::Question::expand("overwrite")
        .message("Conflict on `file.rs`")
        .separator(" = The Meats = ")
        .choices(vec![
            ('y', "Overwrite"),
            ('a', "Overwrite this one and all next"),
            ('d', "Show diff"),
        ])
        .default_separator()
        .choice('x', "Abort")
        .build();

    println!("{:#?}", inquisition::prompt_one(question));
}
