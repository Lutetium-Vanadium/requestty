fn main() {
    inquisition::questions! [
        expand {
            name: "name",
            default: 'c',
            transform: |_, _, _| Ok(()),
            async transform: |_, _, _| Box::pin(async { Ok(()) }),
            choices: [('c', "choice")],
            page_size: 0,
            should_loop: true,
        }
    ];
}
