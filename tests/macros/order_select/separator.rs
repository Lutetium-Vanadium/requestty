fn main() {
    requestty::questions! [
        OrderSelect {
            name: "name",
            choices: [
                sep "separator",
            ],
        }
    ];
}
