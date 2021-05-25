fn main() {
    inquisition::questions![Confirm {
        name: "name",
        default: true,
        transform: |_, _, _| Ok(()),
    }];
}
