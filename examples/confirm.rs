fn main() {
    let question = requestty::Question::confirm("anonymous")
        .message("Do you want to remain anonymous?")
        .build();

    println!("{:#?}", requestty::prompt_one(question));
}
