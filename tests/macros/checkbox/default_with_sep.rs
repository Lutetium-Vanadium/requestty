fn main() {
    discourse::questions! [
        Checkbox {
            name: "name",
            choices: [
                sep "separator" default true,
            ],
        }
    ];
}
