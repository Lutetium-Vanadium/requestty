fn main() {
    println!("{:#?}", ask());
}

fn ask() -> inquisition::Result<Vec<String>> {
    let mut output = Vec::new();

    loop {
        output.push(
            inquisition::prompt_one(
                inquisition::Question::input("tv_show")
                    .message("What's your favourite TV show?"),
            )?
            .try_into_string()
            .unwrap(),
        );

        let ask_again = inquisition::Question::confirm("ask_again")
            .message(
                "Want to enter another TV show favorite (just hit enter for YES)?",
            )
            .default(true);

        if !inquisition::prompt_one(ask_again)?.try_into_bool().unwrap() {
            break;
        }
    }

    Ok(output)
}
