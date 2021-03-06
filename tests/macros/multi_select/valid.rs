fn main() {
    let choice = "choice";
    let default_choice = true;

    requestty::questions! [
        MultiSelect {
            name: "name",
            on_esc: requestty::OnEsc::Terminate,
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
            page_size: 10,
            should_loop: true,
        }
    ];
}
