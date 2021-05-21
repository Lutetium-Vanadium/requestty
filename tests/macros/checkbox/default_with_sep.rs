fn main() {
    inquisition::questions! [
        Checkbox {
            name: "name",
            choices: [
                sep "separator" default true,
            ],
        }
    ];
}
