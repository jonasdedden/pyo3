//! Example of custom annotations.

use pyo3::prelude::*;

#[pymodule]
pub mod annotations {
    use crate::pyclasses::EmptyClass;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyTuple};

    #[pyfunction(signature = (a: "list[int]", *_args: "str", _b: "int | None" = None, **_kwargs: "bool") -> "int")]
    fn with_custom_type_annotations<'py>(
        a: Bound<'py, PyAny>,
        _args: Bound<'py, PyTuple>,
        _b: Option<Bound<'py, PyAny>>,
        _kwargs: Option<Bound<'py, PyDict>>,
    ) -> Bound<'py, PyAny> {
        a
    }

    /// Annotations naming something outside `builtins` have to be imported by the stub
    #[pyfunction(signature = (when: "datetime.date") -> "datetime.date | None")]
    fn with_imported_type_annotations(when: Bound<'_, PyAny>) -> Bound<'_, PyAny> {
        when
    }

    #[pyfunction]
    fn cross_module_imports(_a: &EmptyClass) {}
}
