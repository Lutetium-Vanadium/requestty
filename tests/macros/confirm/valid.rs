fn main() {
    discourse::questions![Confirm {
        name: "name",
        default: true,
        transform: |_, _, _| Ok(()),
    }];
}
