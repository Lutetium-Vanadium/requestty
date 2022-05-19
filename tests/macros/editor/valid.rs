use std::process::Command;

fn main() {
    requestty::questions![Editor {
        name: "name",
        default: "hello world",
        extension: ".rs",
        editor: Command::new("vim"),
        on_esc: requestty::OnEsc::Terminate,
        transform: |_, _, _| Ok(()),
        validate: |_, _| Ok(()),
        filter: |t, _| t,
    }];
}
