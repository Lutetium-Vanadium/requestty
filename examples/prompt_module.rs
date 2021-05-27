use discourse::{Choice, DefaultSeparator, Question, Separator};

fn main() -> discourse::Result<()> {
    let phone_validator = regex::RegexBuilder::new(r"^([01]{1})?[-.\s]?\(?(\d{3})\)?[-.\s]?(\d{3})[-.\s]?(\d{4})\s?((?:#|ext\.?\s?|x\.?\s?){1}(?:\d+)?)?$")
        .case_insensitive(true)
        .build()
        .unwrap();

    let answers = discourse::Answers::default();

    // the prompt module can also be created with the `discourse::prompt_module` macro
    let mut module = discourse::PromptModule::new(vec![
        Question::confirm("to_be_delivered")
            .message("Is this for delivery?")
            .default(false)
            .build(),
        Question::input("phone")
            .message("What's your phone number?")
            .validate(|value, _| {
                if phone_validator.is_match(value) {
                    Ok(())
                } else {
                    Err("Please enter a valid phone number".into())
                }
            })
            .build(),
        Question::select("size")
            .message("What size do you need?")
            .choice("Large")
            .choice("Medium")
            .choice("Small")
            .build(),
        Question::int("quantity")
            .message("How many do you need?")
            .validate(|ans, _| {
                if ans > 0 {
                    Ok(())
                } else {
                    Err("You need to order at least one pizza".into())
                }
            })
            .build(),
        Question::confirm("custom_toppings")
            .message("Do you want to customize the toppings?")
            .default(false)
            .build(),
        Question::expand("toppings")
            .message("What about the toppings?")
            .when(|answers: &discourse::Answers| {
                !answers["custom_toppings"].as_bool().unwrap()
            })
            .choice('p', "Pepperoni and cheese")
            .choice('a', "All dressed")
            .choice('w', "Hawaiian")
            .build(),
        Question::checkbox("toppings")
            .message("Select toppings")
            .when(|answers: &discourse::Answers| {
                answers["custom_toppings"].as_bool().unwrap()
            })
            .separator(" = The Meats = ")
            .choices(vec!["Pepperoni", "Ham", "Ground Meat", "Bacon"])
            .separator(" = The Cheeses = ")
            .choice_with_default("Mozzarella", true)
            .choice("Cheddar")
            .choices(vec![
                Choice("Parmesan".into()),
                Separator(" = The usual = ".into()),
                "Mushroom".into(),
                "Tomato".into(),
                Separator(" = The extras = ".into()),
                "Pineapple".into(),
                "Olives".into(),
                "Extra cheese".into(),
                DefaultSeparator,
            ])
            .build(),
        Question::raw_select("beverage")
            .message("You also get a free 2L beverage")
            .choice("Pepsi")
            .choice("7up")
            .choice("Coke")
            .build(),
        Question::input("comments")
            .message("Any comments on your purchase experience?")
            .default("Nope, all good!")
            .build(),
        Question::select("prize")
            .message("For leaving a comment, you get a freebie")
            .choices(vec!["cake", "fries"])
            .when(|answers: &discourse::Answers| {
                return answers["comments"].as_string().unwrap()
                    != "Nope, all good!";
            })
            .build(),
    ])
    // you can use answers from before
    .with_answers(answers);

    // you can also prompt a single question, and get a mutable reference to its answer
    if module.prompt()?.unwrap().as_bool().unwrap() {
        println!("Delivery is guaranteed to be under 40 minutes");
    }

    println!("{:#?}", module.prompt_all()?);

    Ok(())
}
