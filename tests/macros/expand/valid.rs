fn main() {
    requestty::questions![Expand {
        name: "name",
        default: 'c',
        on_esc: requestty::OnEsc::Terminate,
        transform: |_, _, _| Ok(()),
        choices: [('c', "choice")],
        page_size: 10,
        should_loop: true,
    }];
}
