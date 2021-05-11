fn main() {
    let choice = "choice";
    let default_choice = true;

    inquisition::questions! [
        checkbox {
            name: "name",
            transform: |_, _, _| Ok(()),
            async transform: |_, _, _| Box::pin(async { Ok(()) }),
            validate: |_, _| Ok(()),
            async validate: |_, _| Box::pin(async { Ok(()) }),
            filter: |t, _| t,
            async filter: |t, _| Box::pin(async move { t }),
            choices: [
                sep,
                sep "separator",
                separator,
                separator "separator",
                "choice",
                "choice" default true,
                choice default default_choice || false,
            ],
            page_size: 0,
            should_loop: true,
        }
    ];
}
