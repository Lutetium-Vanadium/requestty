fn main() {
    requestty::questions![Editor {
        name: "name",
        default: "hello world",
        extension: ".rs",
        on_esc: requestty::OnEsc::Terminate,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
    }];
}
