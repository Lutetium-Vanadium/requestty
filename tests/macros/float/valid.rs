fn main() {
    discourse::questions![Float {
        name: "name",
        default: 0.0,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
    }];
}
