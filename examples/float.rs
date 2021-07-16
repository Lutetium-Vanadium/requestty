fn main() {
    let question = requestty::Question::float("number")
        .message("What is your favourite number?")
        .validate(|num, _| {
            if num.is_finite() {
                Ok(())
            } else {
                Err("Please enter a finite number".to_owned())
            }
        })
        .build();

    println!("{:#?}", requestty::prompt_one(question));
}
