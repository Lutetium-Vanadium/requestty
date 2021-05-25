fn main() {
    inquisition::questions![Select {
        name: "name",
        default: 0,
        transform: |_, _, _| Ok(()),
        choices: ["choice"],
        page_size: 0,
        should_loop: true,
    }];
}
