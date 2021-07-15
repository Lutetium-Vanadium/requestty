fn main() {
    requestty::questions! [
        MultiSelect {
            name: "name",
            choices: [
                sep "separator" default true,
            ],
        }
    ];
}
