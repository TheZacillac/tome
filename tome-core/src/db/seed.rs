use rusqlite::Connection;

use crate::error::Result;
use crate::tld::Tld;

use super::upsert_tld;

/// Seed the database from a JSON array of TLD entries.
///
/// The JSON should be an array of objects matching the `Tld` struct.
/// Existing entries with the same TLD key will be replaced.
pub fn seed_tlds_from_json(conn: &Connection, json: &str) -> Result<usize> {
    let tlds: Vec<Tld> =
        serde_json::from_str(json).map_err(crate::error::TomeError::SerializationError)?;

    let count = tlds.len();

    conn.execute_batch("BEGIN TRANSACTION;")?;
    for tld in &tlds {
        if let Err(e) = upsert_tld(conn, tld) {
            conn.execute_batch("ROLLBACK;")?;
            return Err(e);
        }
    }
    conn.execute_batch("COMMIT;")?;

    Ok(count)
}

/// Seed the database from a JSON file at the given path.
pub fn seed_tlds_from_file(conn: &Connection, path: &std::path::Path) -> Result<usize> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| crate::error::TomeError::DataError(format!("Failed to read {}: {e}", path.display())))?;
    seed_tlds_from_json(conn, &json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{count_tlds, get_tld, open_in_memory};

    #[test]
    fn test_seed_from_json() {
        let conn = open_in_memory().unwrap();
        let json = r#"[
            {
                "tld": "test",
                "tld_type": "test",
                "name": "Test TLD",
                "registry": "IANA",
                "dnssec": "unsigned",
                "idn_support": false,
                "references": [],
                "allowed_countries": [],
                "create_periods": [1],
                "renew_periods": [1],
                "auto_renew_periods": [1],
                "transfer_periods": [],
                "features": ["CREATE"],
                "contacts": [],
                "allowed_dnssec_algorithms": []
            }
        ]"#;

        let count = seed_tlds_from_json(&conn, json).unwrap();
        assert_eq!(count, 1);
        assert_eq!(count_tlds(&conn, None).unwrap(), 1);

        let tld = get_tld(&conn, "test").unwrap().unwrap();
        assert_eq!(tld.name, "Test TLD");
        assert_eq!(tld.registry, "IANA");
    }
}
