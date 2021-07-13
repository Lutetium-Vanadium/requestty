fn main() {
    discourse::questions![Expand {
        name: "name",
        default: 'c',
        transform: |_, _, _| Ok(()),
        choices: [('c', "choice")],
        page_size: 10,
        should_loop: true,
    }];
}
