fn main() {
    let choice = "choice";
    let default_choice = true;

    discourse::questions! [
        MultiSelect {
            name: "name",
            transform: |_, _, _| Ok(()),
            validate: |_, _| Ok(()),
            filter: |t, _| t,
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
