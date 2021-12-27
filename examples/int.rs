fn main() {
    let question = requestty::Question::int("age")
        .message("What is your age?")
        .validate_on_key(|age, _| age > 0 && age < 130)
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
