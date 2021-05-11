fn main() {
    inquisition::questions![confirm {
        name: "name",
        default: true,
        transform: |_, _, _| Ok(()),
        async transform: |_, _, _| Box::pin(async { Ok(()) }),
    }];
}
