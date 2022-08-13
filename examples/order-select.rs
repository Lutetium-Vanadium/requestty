use requestty::Question;

fn main() {
    let order_select = Question::order_select("tasks")
        .message("Please organize the tasks")
        .choices(vec![
            "Task 1",
            "Task 2",
            "Task 3",
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