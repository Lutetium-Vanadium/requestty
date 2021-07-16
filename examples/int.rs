fn main() {
    let question = requestty::Question::int("age")
        .message("What is your age?")
        .validate(|age, _| {
            if age > 0 && age < 130 {
                Ok(())
            } else {
                Err(format!("You cannot be {} years old!", age))
            }
        })
        .build();

    println!("{:#?}", requestty::prompt_one(question));
}
