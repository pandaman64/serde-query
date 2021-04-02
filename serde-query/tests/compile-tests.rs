#[test]
fn ui() {
    use trybuild::TestCases;

    let t = TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
    t.pass("tests/compile-pass/*.rs");
}
