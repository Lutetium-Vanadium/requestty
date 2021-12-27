fn main() {
    requestty::questions![Float {
        name: "name",
        default: 0.0,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        validate_on_key: |_, _| true,
        filter: |t, _| t,
    }];
}
