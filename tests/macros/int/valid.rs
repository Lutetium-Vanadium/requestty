fn main() {
    discourse::questions![Int {
        name: "name",
        default: 0,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
    }];
}
