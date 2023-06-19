use monty::{Executor, Exit};

macro_rules! parse_error_tests {
    ($($name:ident: $code:literal, $expected:literal;)*) => {
        $(
            paste::item! {
                #[test]
                fn [< parse_error_ $name >]() {
                    match Executor::new($code, "test.py", &[]) {
                        Ok(v) => panic!("parse unexpected passed, output: {v:?}"),
                        Err(e) => assert_eq!(e.summary(), $expected),
                    }
                }
            }
        )*
    }
}

parse_error_tests! {
    add_int_str: "1 + '1'", "Exc: (1-1 to 1-8) TypeError: unsupported operand type(s) for +: 'int' and 'str'";
    complex: "1+2j", "TODO: complex constants";
}

macro_rules! execute_ok_tests {
    ($($name:ident: $code:literal, $expected:expr;)*) => {
        $(
            paste::item! {
                #[test]
                fn [< execute_ok_ $name >]() {
                    let ex = Executor::new($code, "test.py", &[]).unwrap();
                    let output = match ex.run(vec![]) {
                        Ok(Exit::Return(value)) => format!("{:?}", value),
                        otherwise => panic!("Unexpected exit: {:?}", otherwise),
                    };
                    let expected = $expected.trim_matches('\n');
                    assert_eq!(output, expected);
                }
            }
        )*
    }
}

execute_ok_tests! {
    add_ints: "1 + 1", "Int(2)";
    add_strs: "'a' + 'b'", r#"Str("ab")"#;
    // language=Python
    for_loop_str_append_assign_op: "
v = ''
for i in range(1000):
    if i % 13 == 0:
        v += 'x'
len(v)
", "Int(77)";
    // language=Python
    for_loop_str_append_assign: "
v = ''
for i in range(1000):
    if i % 13 == 0:
        v = v + 'x'
len(v)
", "Int(77)";
}

macro_rules! execute_raise_tests {
    ($($name:ident: $code:literal, $expected_exc:expr;)*) => {
        $(
            paste::item! {
                #[test]
                fn [< execute_raise_ $name >]() {
                    let ex = Executor::new($code, "test.py", &[]).unwrap();
                    let output = match ex.run(vec![]) {
                        Ok(Exit::Raise(exc_raise)) => format!("{:?}", exc_raise.exc),
                        otherwise => panic!("Unexpected raise: {:?}", otherwise),
                    };
                    let expected = $expected_exc.trim_matches('\n');
                    assert_eq!(output, expected);
                }
            }
        )*
    }
}

execute_raise_tests! {
    // language=Python
    type_error_instance: "
raise TypeError('testing')
", r#"TypeError("testing")"#;
//     // language=Python
//     type_error_type: "
// raise TypeError
// ", "TypeError";
}
