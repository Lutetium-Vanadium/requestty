fn main() {
    let question = inquisition::Question::editor("bio")
        .message("Please write a short bio of at least 3 lines.")
        .validate(|answer, _| {
            if answer.lines().count() < 3 {
                Err("Must be at least 3 lines.".into())
            } else {
                Ok(())
            }
        })
        .build();

    println!("{:#?}", inquisition::prompt_one(question));
}
