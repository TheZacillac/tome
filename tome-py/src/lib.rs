use pyo3::prelude::*;
use pyo3::types::PyDict;
use tome_core::{GlossaryDatabase, RecordTypeDatabase, TldDatabase};

use std::sync::OnceLock;

static TLD_DB: OnceLock<TldDatabase> = OnceLock::new();
static RECORD_DB: OnceLock<RecordTypeDatabase> = OnceLock::new();
static GLOSSARY_DB: OnceLock<GlossaryDatabase> = OnceLock::new();

fn to_py_dict<'py>(py: Python<'py>, value: &impl serde::Serialize) -> PyResult<Bound<'py, PyDict>> {
    let json_str = serde_json::to_string(value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let json_mod = py.import("json")?;
    let py_obj = json_mod.call_method1("loads", (json_str,))?;
    py_obj
        .downcast::<PyDict>()
        .map(|d| d.clone())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyTypeError, _>(e.to_string()))
}

/// Look up a top-level domain by name.
#[pyfunction]
fn tld_lookup(py: Python<'_>, query: &str) -> PyResult<Option<PyObject>> {
    let db = TLD_DB.get_or_init(|| TldDatabase::new(Vec::new()));
    match db.lookup(query) {
        Some(tld) => Ok(Some(to_py_dict(py, tld)?.into())),
        None => Ok(None),
    }
}

/// Search TLDs by partial match.
#[pyfunction]
fn tld_search(py: Python<'_>, query: &str) -> PyResult<Vec<PyObject>> {
    let db = TLD_DB.get_or_init(|| TldDatabase::new(Vec::new()));
    db.search(query)
        .iter()
        .map(|tld| Ok(to_py_dict(py, tld)?.into()))
        .collect()
}

/// Look up a DNS record type by name or code.
#[pyfunction]
fn record_lookup(py: Python<'_>, query: &str) -> PyResult<Option<PyObject>> {
    let db = RECORD_DB.get_or_init(|| RecordTypeDatabase::new(Vec::new()));
    let result = if let Ok(code) = query.parse::<u16>() {
        db.lookup_by_code(code)
    } else {
        db.lookup(query)
    };
    match result {
        Some(rt) => Ok(Some(to_py_dict(py, rt)?.into())),
        None => Ok(None),
    }
}

/// Search DNS record types by partial match.
#[pyfunction]
fn record_search(py: Python<'_>, query: &str) -> PyResult<Vec<PyObject>> {
    let db = RECORD_DB.get_or_init(|| RecordTypeDatabase::new(Vec::new()));
    db.search(query)
        .iter()
        .map(|rt| Ok(to_py_dict(py, rt)?.into()))
        .collect()
}

/// Look up a glossary term.
#[pyfunction]
fn glossary_lookup(py: Python<'_>, query: &str) -> PyResult<Option<PyObject>> {
    let db = GLOSSARY_DB.get_or_init(|| GlossaryDatabase::new(Vec::new()));
    match db.lookup(query) {
        Some(term) => Ok(Some(to_py_dict(py, term)?.into())),
        None => Ok(None),
    }
}

/// Search glossary terms by partial match.
#[pyfunction]
fn glossary_search(py: Python<'_>, query: &str) -> PyResult<Vec<PyObject>> {
    let db = GLOSSARY_DB.get_or_init(|| GlossaryDatabase::new(Vec::new()));
    db.search(query)
        .iter()
        .map(|term| Ok(to_py_dict(py, term)?.into()))
        .collect()
}

#[pymodule]
fn _tome(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tld_lookup, m)?)?;
    m.add_function(wrap_pyfunction!(tld_search, m)?)?;
    m.add_function(wrap_pyfunction!(record_lookup, m)?)?;
    m.add_function(wrap_pyfunction!(record_search, m)?)?;
    m.add_function(wrap_pyfunction!(glossary_lookup, m)?)?;
    m.add_function(wrap_pyfunction!(glossary_search, m)?)?;
    Ok(())
}
