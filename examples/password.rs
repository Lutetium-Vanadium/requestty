use requestty::Question;

fn is_valid(password: &str, _: &requestty::Answers) -> bool {
    password.contains(|c: char| c.is_ascii_digit()) && password.contains(char::is_alphabetic)
}

fn letter_and_numbers(password: &str, ans: &requestty::Answers) -> Result<(), String> {
    if is_valid(password, ans) {
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
            .validate_on_key(is_valid)
            .validate(letter_and_numbers)
            .build(),
    ];

    println!("{:#?}", requestty::prompt(questions));
}
