fn main() {
    requestty::questions![RawSelect {
        name: "name",
        default: 0,
        on_esc: requestty::OnEsc::Terminate,
        transform: |_, _, _| Ok(()),
        choices: ["choice"],
        page_size: 10,
        should_loop: true,
    }];
}
