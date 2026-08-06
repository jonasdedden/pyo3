//@revisions: default inspect
//@[default] without-experimental-inspect
//@[inspect] with-experimental-inspect

use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (x: "int int"))]
//~[inspect]^ ERROR: invalid Python type hint: unexpected `int`
fn argument_annotation(x: i32) {
    let _ = x;
}

#[pyfunction]
#[pyo3(signature = () -> "list[int")]
//~[inspect]^ ERROR: invalid Python type hint: expected `]`, found the end of the type hint
fn return_annotation() {}

fn main() {}
