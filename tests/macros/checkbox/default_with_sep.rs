fn main() {
    inquisition::questions! [
        checkbox {
            name: "name",
            choices: [
                sep "separator" default true,
            ],
        }
    ];
}
