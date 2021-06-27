fn main() {
    discourse::questions![Input {
        name: "name",
        default: "hello world",
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
        auto_complete: |t, _| discourse::question::Completions::from([t]),
    }];
}
