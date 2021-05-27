fn main() {
    let question = discourse::Question::select("long-list")
        .message("Select from this super long list")
        .choices(('A'..='Z').map(|c| c.to_string()))
        .choices((1..=5).map(|i| {
            format!(
                "Multiline option {}\nsuper cool feature \nmore lines",
                i
            )
        }))
        .choice("Super long option:\nLorem ipsum dolor sit amet, consectetuer \
        adipiscing elit. Aenean commodo ligula e get dolor. Aenean massa. Cum sociis \
        natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla \
        consequat massa quis enim. Donec pede justo, fringilla vel, aliquet nec, \
        vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, \
        justo. Nullam dictum felis eu pede mollis pretium.")
        .default_separator()
        .build();

    println!("{:#?}", discourse::prompt_one(question));
}
