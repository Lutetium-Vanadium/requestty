fn main() {
    discourse::questions![Input {
        name: "name",
        default: "hello world",
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
        auto_complete: |t, _| vec![t],
    }];
}
