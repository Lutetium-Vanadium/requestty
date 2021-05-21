fn main() {
    inquisition::questions![Confirm {
        name: "name",
        default: true,
        transform: |_, _, _| Ok(()),
        async transform: |_, _, _| Box::pin(async { Ok(()) }),
    }];
}
