fn main() {
    discourse::questions![Select {
        name: "name",
        default: 0,
        transform: |_, _, _| Ok(()),
        choices: ["choice"],
        page_size: 10,
        should_loop: true,
    }];
}
