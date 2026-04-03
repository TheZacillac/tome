mod bridge;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use tome_core::db::{TldOverviewRow, TomeDb};
use tome_core::{GlossaryDatabase, RecordStatus, RecordTypeDatabase};

use std::sync::{Mutex, OnceLock};

// ---------------------------------------------------------------------------
// Lazy-initialized databases
// ---------------------------------------------------------------------------

/// The SQLite TLD database, seeded on first access.
///
/// Uses an in-memory database so there are no filesystem dependencies.
/// The seed data is compiled into the binary via `include_str!` in tome-core.
///
/// Wrapped in `Mutex` because `rusqlite::Connection` is `!Sync`.
/// Record type and glossary databases remain in-memory for now.
/// These will be migrated to SQLite in a future iteration.
static RECORD_DB: OnceLock<RecordTypeDatabase> = OnceLock::new();
static GLOSSARY_DB: OnceLock<GlossaryDatabase> = OnceLock::new();

/// Initialization result stored in the OnceLock so errors can be surfaced as
/// Python exceptions instead of panicking.
static TOME_DB_RESULT: OnceLock<Result<Mutex<TomeDb>, String>> = OnceLock::new();

fn with_db<T, F>(f: F) -> PyResult<T>
where
    F: FnOnce(&TomeDb) -> tome_core::Result<T>,
{
    let result = TOME_DB_RESULT.get_or_init(|| {
        let db = match TomeDb::open_in_memory() {
            Ok(db) => db,
            Err(e) => return Err(format!("failed to create in-memory TomeDb: {e}")),
        };
        if let Err(e) = tome_core::seed::seed(&db) {
            return Err(format!("failed to run initial seed: {e}"));
        }
        if let Err(e) = tome_core::seed_extended::seed_extended(&db) {
            return Err(format!("failed to run extended seed: {e}"));
        }
        Ok(Mutex::new(db))
    });
    let mutex = match result {
        Ok(m) => m,
        Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.clone())),
    };
    let db = mutex.lock().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("lock poisoned: {e}"))
    })?;
    f(&db).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

// ---------------------------------------------------------------------------
// Serialization helpers
// ---------------------------------------------------------------------------

fn to_py_dict<'py>(py: Python<'py>, value: &impl serde::Serialize) -> PyResult<Bound<'py, PyDict>> {
    let json_str = serde_json::to_string(value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let json_mod = py.import("json")?;
    let py_obj = json_mod.call_method1("loads", (json_str,))?;
    py_obj
        .downcast::<PyDict>()
        .cloned()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyTypeError, _>(e.to_string()))
}

fn enrich_dict_with_overview(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
    ov: &TldOverviewRow,
) -> PyResult<()> {
    if let Some(ref v) = ov.registry_operator {
        dict.set_item("registry_operator", v)?;
    }
    if let Some(ref v) = ov.country_name {
        dict.set_item("country_name", v)?;
    }
    if let Some(ref v) = ov.iso_3166_alpha2 {
        dict.set_item("iso_3166_alpha2", v)?;
    }
    if let Some(ref v) = ov.registration_model {
        dict.set_item("registration_model", v)?;
    }
    if let Some(ref v) = ov.whois_server {
        dict.set_item("whois_server", v)?;
    }
    if let Some(ref v) = ov.rdap_base_url {
        dict.set_item("rdap_base_url", v)?;
    }
    if let Some(v) = ov.requires_local_presence {
        dict.set_item("requires_local_presence", v)?;
    }
    if let Some(v) = ov.supports_dnssec {
        dict.set_item("supports_dnssec", v)?;
    }
    if let Some(v) = ov.allows_idn {
        dict.set_item("allows_idn", v)?;
    }
    if let Some(v) = ov.registry_lock_available {
        dict.set_item("registry_lock_available", v)?;
    }
    if let Some(ref v) = ov.phishing_abuse_risk {
        dict.set_item("phishing_abuse_risk", v)?;
    }
    if let Some(v) = ov.defensive_registration_recommended {
        dict.set_item("defensive_registration_recommended", v)?;
    }
    if let Some(v) = ov.transfer_adds_year {
        dict.set_item("transfer_adds_year", v)?;
    }
    let _ = py; // used for lifetime only
    Ok(())
}

fn overview_to_dict<'py>(py: Python<'py>, ov: &TldOverviewRow) -> PyResult<Bound<'py, PyDict>> {
    to_py_dict(py, ov)
}

// ---------------------------------------------------------------------------
// TLD functions — backed by SQLite + seed data
// ---------------------------------------------------------------------------

/// Look up a top-level domain by name.
///
/// Returns a dict with TLD info (type, delegation status, description, dates)
/// enriched with registry operator, country, WHOIS/RDAP data from related tables.
/// Returns None if not found.
#[pyfunction]
fn tld_lookup(py: Python<'_>, query: &str) -> PyResult<Option<PyObject>> {
    let (row, overview) = with_db(|db| {
        let row = db.get_tld(query)?;
        let overview = db.get_tld_overview(query)?;
        Ok((row, overview))
    })?;

    match row {
        Some(ref r) => {
            let dict = to_py_dict(py, r)?;
            // Enrich with overview data
            if let Some(ref ov) = overview {
                enrich_dict_with_overview(py, &dict, ov)?;
            }
            Ok(Some(dict.into()))
        }
        None => Ok(None),
    }
}

/// Search TLDs by partial match on label or description.
///
/// Returns a list of dicts.
#[pyfunction]
fn tld_search(py: Python<'_>, query: &str) -> PyResult<PyObject> {
    let rows = with_db(|db| db.search_tlds(query))?;
    let list = PyList::empty(py);
    for row in &rows {
        list.append(to_py_dict(py, row)?)?;
    }
    Ok(list.into())
}

/// Get the full overview for a TLD (joins core, registry, country, WHOIS, technical, brand tables).
///
/// Returns a dict or None.
#[pyfunction]
fn tld_overview(py: Python<'_>, query: &str) -> PyResult<Option<PyObject>> {
    let ov = with_db(|db| db.get_tld_overview(query))?;
    match ov {
        Some(ref o) => Ok(Some(overview_to_dict(py, o)?.into())),
        None => Ok(None),
    }
}

/// List all TLDs of a given type ("gTLD", "ccTLD", or "nTLD").
///
/// Returns a list of dicts.
#[pyfunction]
fn tld_list_by_type(py: Python<'_>, tld_type: &str) -> PyResult<PyObject> {
    let rows = with_db(|db| db.list_tlds_by_type(tld_type))?;
    let list = PyList::empty(py);
    for row in &rows {
        list.append(to_py_dict(py, row)?)?;
    }
    Ok(list.into())
}

/// Return the total number of TLDs in the database.
#[pyfunction]
fn tld_count() -> PyResult<usize> {
    with_db(|db| db.count_tlds())
}

// ---------------------------------------------------------------------------
// Record type functions — still in-memory (no data yet)
// ---------------------------------------------------------------------------

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

/// List DNS record types by status (active, experimental, obsolete, reserved).
#[pyfunction]
fn record_by_status(py: Python<'_>, status: &str) -> PyResult<Vec<PyObject>> {
    let status = match status.to_lowercase().as_str() {
        "active" => RecordStatus::Active,
        "experimental" => RecordStatus::Experimental,
        "obsolete" => RecordStatus::Obsolete,
        "reserved" => RecordStatus::Reserved,
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid status: {status}. Expected one of: active, experimental, obsolete, reserved"
            )));
        }
    };
    let db = RECORD_DB.get_or_init(|| RecordTypeDatabase::new(Vec::new()));
    db.by_status(&status)
        .iter()
        .map(|rt| Ok(to_py_dict(py, rt)?.into()))
        .collect()
}

// ---------------------------------------------------------------------------
// Glossary functions — still in-memory (no data yet)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

/// Install a tracing subscriber that forwards Rust log events into Python's
/// ``logging`` module.  Safe to call multiple times.
#[pyfunction]
fn init_rust_logging() {
    bridge::install_bridge();
}

#[pymodule]
#[pyo3(name = "_tome")]
fn _tome(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_rust_logging, m)?)?;
    // TLD (SQLite-backed)
    m.add_function(wrap_pyfunction!(tld_lookup, m)?)?;
    m.add_function(wrap_pyfunction!(tld_search, m)?)?;
    m.add_function(wrap_pyfunction!(tld_overview, m)?)?;
    m.add_function(wrap_pyfunction!(tld_list_by_type, m)?)?;
    m.add_function(wrap_pyfunction!(tld_count, m)?)?;
    // Record types (in-memory, no data yet)
    m.add_function(wrap_pyfunction!(record_lookup, m)?)?;
    m.add_function(wrap_pyfunction!(record_search, m)?)?;
    m.add_function(wrap_pyfunction!(record_by_status, m)?)?;
    // Glossary (in-memory, no data yet)
    m.add_function(wrap_pyfunction!(glossary_lookup, m)?)?;
    m.add_function(wrap_pyfunction!(glossary_search, m)?)?;
    Ok(())
}
