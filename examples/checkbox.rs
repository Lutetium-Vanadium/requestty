use inquisition::{Choice, Separator};

fn main() {
    let question = inquisition::Question::checkbox("toppings")
        .message("Select toppings")
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
        ])
        .validate(|answer, _| {
            if answer.iter().filter(|&&a| a).count() < 1 {
                Err("You must choose at least one topping.".into())
            } else {
                Ok(())
            }
        })
        .build();

    println!("{:#?}", inquisition::prompt_one(question));
}
