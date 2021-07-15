fn main() {
    requestty::questions![Confirm {
        name: "name",
        default: true,
        transform: |_, _, _| Ok(()),
    }];
}
