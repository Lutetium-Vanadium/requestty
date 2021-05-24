use csscolorparser::parse as parse_col;
use inquisition::plugin::Stylize;
use inquisition::Question;

fn main() {
    let questions = vec![
        Question::input("first_name")
            .message("What's your first name")
            .build(),
        Question::input("last_name")
            .message("What's your last name")
            .default("Doe")
            .build(),
        Question::input("fav_color")
            .message("What's your favourite colour")
            .validate(|ans, _| match parse_col(ans) {
                Ok(_) => Ok(()),
                Err(_) => Err("Please provide a valid css colour".into()),
            })
            .transform(|ans, _, backend| {
                let (r, g, b, _) = parse_col(ans).unwrap().rgba_u8();

                backend.write_styled(ans.rgb(r, g, b))
            })
            .build(),
    ];

    println!("{:#?}", inquisition::prompt(questions));
}
