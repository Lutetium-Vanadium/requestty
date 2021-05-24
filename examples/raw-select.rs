use inquisition::{DefaultSeparator, Question};

fn main() {
    let questions = vec![
        Question::raw_select("theme")
            .message("What do you want to do?")
            .choices(vec![
                "Order a pizza".into(),
                "Make a reservation".into(),
                DefaultSeparator,
                "Ask for opening hours".into(),
                "Talk to the receptionist".into(),
            ])
            .build(),
        Question::select("size")
            .message("What size do you need?")
            .choices(vec![
                "Jumbo", "Large", "Standard", "Medium", "Small", "Micro",
            ])
            .build(),
    ];

    println!("{:#?}", inquisition::prompt(questions));
}
