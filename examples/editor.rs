fn main() {
    let question = discourse::Question::editor("bio")
        .message("Please write a short bio of at least 3 lines.")
        .validate(|answer, _| {
            if answer.lines().count() < 3 {
                Err("Must be at least 3 lines.".into())
            } else {
                Ok(())
            }
        })
        .build();

    println!("{:#?}", discourse::prompt_one(question));
}
