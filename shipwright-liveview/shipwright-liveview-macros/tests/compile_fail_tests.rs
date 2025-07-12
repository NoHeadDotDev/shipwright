//! Compile-time failure tests using trybuild
//! These tests ensure that our HTML validation catches errors at compile time

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    
    // Test files in the tests/compile_fail directory
    t.compile_fail("tests/compile_fail/*.rs");
}