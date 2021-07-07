fn main() {
    discourse::questions! [
        MultiSelect {
            name: "name",
            choices: [
                sep "separator" default true,
            ],
        }
    ];
}
