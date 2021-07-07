fn main() {
    let phone_validator = regex::RegexBuilder::new(r"^([01]{1})?[-.\s]?\(?(\d{3})\)?[-.\s]?(\d{3})[-.\s]?(\d{4})\s?((?:#|ext\.?\s?|x\.?\s?){1}(?:\d+)?)?$")
        .case_insensitive(true)
        .build()
        .unwrap();

    let questions = discourse::questions! [
        // Use std::array::IntoIter instead of allocating a Vec if available (>= v1.51)
        inline
        Confirm {
            name: "to_be_delivered",
            message: "Is this for delivery?",
            default: false,
        },
        Input {
            name: "phone",
            message: "What's your phone number?",
            validate: |value, _| {
                if phone_validator.is_match(value) {
                    Ok(())
                } else {
                    Err("Please enter a valid phone number".into())
                }
            },
        },
        Select {
            name: "size",
            message: "What size do you need?",
            choices: ["Large", "Medium", "Small"],
        },
        Int {
            name: "quantity",
            message: "How many do you need?",
            validate: |ans, _| {
                if ans > 0 {
                    Ok(())
                } else {
                    Err("You need to order at least 1 pizza".into())
                }
            },
        },
        Confirm {
            name: "custom_toppings",
            message: "Do you want to customize the toppings?",
            default: false,
        },
        Expand {
            name: "toppings",
            message: "What about the toppings?",
            when: |answers: &discourse::Answers| {
                !answers["custom_toppings"].as_bool().unwrap()
            },
            choices: [
                ('p', "Pepperoni and cheese"),
                ('a', "All dressed"),
                ('w', "Hawaiian"),
            ],
        },
        MultiSelect {
            name: "toppings",
            message: "Select toppings",
            when: |answers: &discourse::Answers| {
                answers["custom_toppings"].as_bool().unwrap()
            },
            // Array style choices (`[...]`) have special parsing
            choices: [
                // Use 'separator' or 'sep' for separators
                separator " = The Meats = ",
                // Otherwise they are considered choices
                "Pepperoni",
                "Ham",
                "Ground Meat",
                "Bacon",
                separator " = The Cheeses = ",
                // Use `<choice> default <value>` to give default value (only for MultiSelect)
                "Mozzarella" default true,
                "Cheddar",
                "Parmesan",
                separator " = The usual = ",
                "Mushroom",
                "Tomato",
                separator " = The extras = ",
                "Pineapple",
                "Olives",
                "Extra cheese",
                // Specifying nothing in front of the separator will give a default
                // separator
                separator,
            ],
        },
        RawSelect {
            name: "beverage",
            message: "You also get a free 2L beverage",
            choices: ["Pepsi", "7up", "Coke"],
        },
        Input {
            name: "comments",
            message: "Any comments on your purchase experience?",
            default: "Nope, all good!",
        },
        Select {
            name: "prize",
            message: "For leaving a comment, you get a freebie",
            choices: ["cake", "fries"],
            when: |answers: &discourse::Answers| {
                answers["comments"].as_string().unwrap() != "Nope, all good!"
            },
        },
    ];

    println!("{:#?}", discourse::prompt(questions));
}
