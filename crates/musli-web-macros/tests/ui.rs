#[cfg(not(miri))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*-error.rs");
    t.pass("tests/ui/*-ok.rs");
}
