fn main() {
    inquisition::questions![Input {
        name: "name",
        default: "hello world",
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
    }];
}
