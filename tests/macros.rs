struct Runner {
    cases: trybuild::TestCases,
    name: &'static str,
}

impl Runner {
    fn new(name: &'static str) -> Self {
        Self {
            cases: trybuild::TestCases::new(),
            name,
        }
    }

    fn compile_fail(&self, test_name: &str) {
        self.cases
            .compile_fail(format!("tests/macros/{}/{}.rs", self.name, test_name))
    }

    fn pass(&self, test_name: &str) {
        self.cases
            .pass(format!("tests/macros/{}/{}.rs", self.name, test_name))
    }
}

#[test]
#[ignore = "proc-macro test"]
fn test_duplicate() {
    let t = Runner::new("duplicate");
    t.compile_fail("name");
    t.compile_fail("message");
    t.compile_fail("when");
    t.compile_fail("ask_if_answered");
    t.compile_fail("default");
    t.compile_fail("validate");
    t.compile_fail("validate_on_key");
    t.compile_fail("filter");
    t.compile_fail("transform");
    t.compile_fail("auto_complete");
    t.compile_fail("choices");
    t.compile_fail("page_size");
    t.compile_fail("should_loop");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_unknown() {
    let t = Runner::new("unknown");
    t.compile_fail("kind");
    t.compile_fail("option");
}

#[test]
#[ignore = "proc-macro test"]
fn test_missing() {
    let t = Runner::new("missing");
    t.compile_fail("name");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_from_local_variable() {
    let t = Runner::new("local_variable");
    t.pass("valid");
    t.compile_fail("not_found");
    t.compile_fail("unexpected_token");
}

#[test]
#[ignore = "proc-macro test"]
fn test_multi_select() {
    let t = Runner::new("multi_select");

    t.pass("valid");
    t.compile_fail("default");
    t.compile_fail("default_with_sep");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_order_select() {
    let t = Runner::new("order_select");

    t.pass("valid");
    t.compile_fail("default");
    t.compile_fail("separator");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_confirm() {
    let t = Runner::new("confirm");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("validate");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_editor() {
    let t = Runner::new("editor");

    t.pass("valid");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_expand() {
    let t = Runner::new("expand");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("validate");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_float() {
    let t = Runner::new("float");

    t.pass("valid");
    t.compile_fail("auto_complete");
    t.compile_fail("choices");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_input() {
    let t = Runner::new("input");

    t.pass("valid");
    t.compile_fail("choices");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_int() {
    let t = Runner::new("int");

    t.pass("valid");
    t.compile_fail("auto_complete");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_select() {
    let t = Runner::new("select");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("validate");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_password() {
    let t = Runner::new("password");

    t.pass("valid");
    t.compile_fail("default");
    t.compile_fail("auto_complete");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}

#[test]
#[ignore = "proc-macro test"]
fn test_custom_prompt() {
    let t = Runner::new("custom_prompt");

    t.pass("valid");
    t.compile_fail("default");
    t.compile_fail("on_esc");
    t.compile_fail("transform");
    t.compile_fail("filter");
    t.compile_fail("validate");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
}

#[test]
#[ignore = "proc-macro test"]
fn test_raw_select() {
    let t = Runner::new("raw_select");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("validate");
    t.compile_fail("validate_on_key");
    t.compile_fail("auto_complete");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("editor");
    t.compile_fail("prompt");
}
