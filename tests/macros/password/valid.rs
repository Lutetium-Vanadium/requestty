fn main() {
    requestty::questions![Password {
        name: "name",
        mask: '*',
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
    }];
}
