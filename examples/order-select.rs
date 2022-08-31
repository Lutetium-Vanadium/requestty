use requestty::Question;

fn main() {
    let order_select = Question::order_select("home_tasks")
        .message("Please organize the tasks to be done at home")
        .choices(vec![
            "Make the bed",
            "Clean the dishes",
            "Mow the lawn",
        ])
        .validate(|c, _| {
            if c[0].text() == "Make the bed" {
                Ok(())
            } else {
                Err("You have to make the bed first".to_string())
            }
        })
        .build();

    println!("{:#?}", requestty::prompt_one(order_select));
}