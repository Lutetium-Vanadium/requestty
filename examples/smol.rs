include!("templates/pizza.rs");

fn main() -> inquisition::Result<()> {
    // It has to be called `smol_dep` in this example due to implementation reasons.
    // When used outside this crate, just `smol` will work.
    smol_dep::block_on(
        async {
            // There is no special async prompt, PromptModule itself can run both
            // synchronously and asynchronously
            let mut module = inquisition::PromptModule::new(pizza_questions());

            // you can also prompt a single question, and get a mutable reference to its
            // answer
            if module.prompt_async().await?.unwrap().as_bool().unwrap() {
                println!("Delivery is guaranteed to be under 40 minutes");
            }

            println!("{:#?}", module.prompt_all_async().await?);

            Ok(())
        },
    )
}
