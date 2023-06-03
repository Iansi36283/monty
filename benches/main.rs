#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use monty::{Executor, Exit, Object};

use pyo3::prelude::*;

#[bench]
fn add_two(bench: &mut Bencher) {
    let ex = Executor::new("1 + 2", "test.py", &[]).unwrap();
    let v = ex.run(vec![]).unwrap();
    match v {
        Exit::Return(Object::Int(v)) => assert_eq!(v, 3),
        _ => panic!("unexpected exit: {:?}", v),
    }

    bench.iter(|| {
        black_box(ex.run(vec![]).unwrap());
    });
}

#[bench]
fn add_two_cpython(bench: &mut Bencher) {
    Python::with_gil(|py| {
        let fun: PyObject = PyModule::from_code(
            py,
            "def main():
                return 1 + 2
            ",
            "test.py",
            "main",
        )
        .unwrap()
        .getattr("main")
        .unwrap()
        .into();

        let r = fun.call0(py).unwrap();
        let r: i64 = r.extract(py).unwrap();
        assert_eq!(r, 3);

        bench.iter(|| {
            let r_py = fun.call0(py).unwrap();
            let r: i64 = r_py.extract(py).unwrap();
            black_box(r);
        });
    });
}

const LOOP_MOD_13_CODE: &str = r#"
v = ''
for i in range(100):
    if i % 13 == 0:
        v += 'x'
len(v)
"#;

#[bench]
fn loop_mod_13(bench: &mut Bencher) {
    let ex = Executor::new(LOOP_MOD_13_CODE, "test.py", &[]).unwrap();
    let v = ex.run(vec![]).unwrap();
    match v {
        Exit::Return(Object::Int(v)) => assert_eq!(v, 8),
        _ => panic!("unexpected exit: {:?}", v),
    }

    bench.iter(|| {
        black_box(ex.run(vec![]).unwrap());
    });
}

#[bench]
fn loop_mod_13_cpython(bench: &mut Bencher) {
    Python::with_gil(|py| {
        let fun: PyObject = PyModule::from_code(
            py,
            "def main():
                v = ''
                for i in range(100):
                    if i % 13 == 0:
                        v += 'x'
                return len(v)
            ",
            "test.py",
            "main",
        )
        .unwrap()
        .getattr("main")
        .unwrap()
        .into();

        let r = fun.call0(py).unwrap();
        let r: i64 = r.extract(py).unwrap();
        assert_eq!(r, 8);

        bench.iter(|| {
            let r_py = fun.call0(py).unwrap();
            let r: i64 = r_py.extract(py).unwrap();
            black_box(r);
        });
    });
}
