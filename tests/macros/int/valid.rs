fn main() {
    requestty::questions![Int {
        name: "name",
        default: 0,
        on_esc: requestty::OnEsc::Terminate,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        validate_on_key: |_, _| true,
        filter: |t, _| t,
    }];
}
