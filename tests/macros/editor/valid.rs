fn main() {
    inquisition::questions! [
        editor {
            name: "name",
            default: "hello world",
            extension: ".rs",
            transform: |_, _, _| Ok(()),
            async transform: |_, _, _| Box::pin(async { Ok(()) }),
            validate: |_, _| Ok(()),
            async validate: |_, _| Box::pin(async { Ok(()) }),
            filter: |t, _| t,
            async filter: |t, _| Box::pin(async move { t }),
        }
    ];
}
