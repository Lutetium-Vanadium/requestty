fn main() {
    requestty::questions![Confirm {
        name: "name",
        default: true,
        transform: |_, _, _| Ok(()),
        on_esc: requestty::OnEsc::Terminate,
    }];
}
