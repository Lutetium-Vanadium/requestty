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
fn duplicate() {
    let t = Runner::new("duplicate");
    t.compile_fail("name");
    t.compile_fail("message");
    t.compile_fail("when");
    t.compile_fail("ask_if_answered");
    t.compile_fail("default");
    t.compile_fail("validate");
    t.compile_fail("filter");
    t.compile_fail("transform");
    t.compile_fail("validate_async");
    t.compile_fail("filter_async");
    t.compile_fail("transform_async");
    t.compile_fail("choices");
    t.compile_fail("page_size");
    t.compile_fail("should_loop");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn unknown() {
    let t = Runner::new("unknown");
    t.compile_fail("kind");
    t.compile_fail("option");
    t.compile_fail("async-unknown");
    t.compile_fail("async-option");
}

#[test]
fn missing() {
    let t = Runner::new("missing");
    t.compile_fail("name");
    t.compile_fail("plugin");
}

#[test]
fn checkbox() {
    let t = Runner::new("checkbox");

    t.pass("valid");
    t.compile_fail("default");
    t.compile_fail("default_with_sep");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn confirm() {
    let t = Runner::new("confirm");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("filter_async");
    t.compile_fail("validate");
    t.compile_fail("validate_async");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn editor() {
    let t = Runner::new("editor");

    t.pass("valid");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("plugin");
}

#[test]
fn expand() {
    let t = Runner::new("expand");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("filter_async");
    t.compile_fail("validate");
    t.compile_fail("validate_async");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn float() {
    let t = Runner::new("float");

    t.pass("valid");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn input() {
    let t = Runner::new("input");

    t.pass("valid");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn int() {
    let t = Runner::new("int");

    t.pass("valid");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn select() {
    let t = Runner::new("select");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("filter_async");
    t.compile_fail("validate");
    t.compile_fail("validate_async");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn password() {
    let t = Runner::new("password");

    t.pass("valid");
    t.compile_fail("default");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}

#[test]
fn plugin() {
    let t = Runner::new("plugin");

    t.pass("valid");
    t.compile_fail("default");
    t.compile_fail("transform");
    t.compile_fail("transform_async");
    t.compile_fail("filter");
    t.compile_fail("filter_async");
    t.compile_fail("validate");
    t.compile_fail("validate_async");
    t.compile_fail("choices");
    t.compile_fail("should_loop");
    t.compile_fail("page_size");
    t.compile_fail("mask");
    t.compile_fail("extension");
}

#[test]
fn raw_select() {
    let t = Runner::new("raw_select");

    t.pass("valid");
    t.compile_fail("filter");
    t.compile_fail("filter_async");
    t.compile_fail("validate");
    t.compile_fail("validate_async");
    t.compile_fail("mask");
    t.compile_fail("extension");
    t.compile_fail("plugin");
}
