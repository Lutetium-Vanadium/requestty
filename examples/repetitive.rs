fn main() {
    println!("{:#?}", ask());
}

fn ask() -> requestty::Result<Vec<String>> {
    let mut output = Vec::new();

    loop {
        output.push(
            requestty::prompt_one(
                requestty::Question::input("tv_show").message("What's your favourite TV show?"),
            )?
            .try_into_string()
            .expect("Question::input returns a string"),
        );

        let ask_again = requestty::Question::confirm("ask_again")
            .message("Want to enter another TV show favorite (just hit enter for YES)?")
            .default(true);

        if !requestty::prompt_one(ask_again)?
            .as_bool()
            .expect("Question::confirm returns a bool")
        {
            break;
        }
    }

    Ok(output)
}
