include!("templates/pizza.rs");

fn main() -> inquisition::Result<()> {
    let answers = inquisition::Answers::default();

    // the prompt module can also be created with the `inquisition::prompt_module` macro
    let mut module = inquisition::PromptModule::new(pizza_questions())
        // you can use answers from before
        .with_answers(answers);

    // you can also prompt a single question, and get a mutable reference to its answer
    if module.prompt()?.unwrap().as_bool().unwrap() {
        println!("Delivery is guaranteed to be under 40 minutes");
    }

    println!("{:#?}", module.prompt_all()?);

    Ok(())
}
