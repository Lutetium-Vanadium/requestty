use requestty::Question;

fn main() {
    let order_select = Question::order_select("home_tasks")
        .message("Please organize the tasks to be done at home")
        .choices(vec![
            "Make the bed",
            "Clean the dishes",
            "Mow the lawn",
        ])
        .validate(|o, _| {
            if o[0] == 0 {
                Ok(())
            } else {
                Err("Task 1 needs to be done first.".to_string())
            }
        })
        .build();

    println!("{:#?}", requestty::prompt_one(order_select));
}