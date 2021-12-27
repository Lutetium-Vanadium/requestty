fn main() {
    requestty::questions![Password {
        name: "name",
        mask: '*',
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        validate_on_key: |_, _| true,
        filter: |t, _| t,
    }];
}
