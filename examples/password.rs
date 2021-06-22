use discourse::Question;

fn letter_and_numbers(password: &str, _: &discourse::Answers) -> Result<(), String> {
    if password.contains(|c: char| c.is_ascii_digit()) && password.contains(char::is_alphabetic) {
        Ok(())
    } else {
        Err("Password needs to have at least 1 letter and 1 number.".to_owned())
    }
}

fn main() {
    let questions = vec![
        Question::password("password1")
            .message("Enter a password")
            .validate(letter_and_numbers)
            .build(),
        Question::password("password2")
            .message("Enter a masked password")
            .mask('*')
            .validate(letter_and_numbers)
            .build(),
    ];

    println!("{:#?}", discourse::prompt(questions));
}
