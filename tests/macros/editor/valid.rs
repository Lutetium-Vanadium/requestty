fn main() {
    inquisition::questions![Editor {
        name: "name",
        default: "hello world",
        extension: ".rs",
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
    }];
}
