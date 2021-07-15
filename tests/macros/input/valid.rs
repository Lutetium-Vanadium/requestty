fn main() {
    requestty::questions![Input {
        name: "name",
        default: "hello world",
        should_loop: true,
        page_size: 10,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
        auto_complete: |t, _| requestty::question::completions![t],
    }];
}
