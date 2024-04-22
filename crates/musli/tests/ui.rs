#[cfg(not(miri))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*_error.rs");
    t.pass("tests/ui/*_ok.rs");
}
