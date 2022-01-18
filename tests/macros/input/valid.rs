fn main() {
    requestty::questions![Input {
        name: "name",
        default: "hello world",
        on_esc: requestty::OnEsc::Terminate,
        should_loop: true,
        page_size: 10,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        validate_on_key: |_, _| true,
        filter: |t, _| t,
        auto_complete: |t, _| requestty::question::completions![t],
    }];
}
