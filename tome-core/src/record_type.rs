use serde::{Deserialize, Serialize};

/// Status of a DNS record type in terms of standardization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordStatus {
    /// Active and in common use
    Active,
    /// Defined but rarely used or experimental
    Experimental,
    /// Obsoleted by a newer record type
    Obsolete,
    /// Reserved but not yet in use
    Reserved,
}

impl std::fmt::Display for RecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordStatus::Active => write!(f, "Active"),
            RecordStatus::Experimental => write!(f, "Experimental"),
            RecordStatus::Obsolete => write!(f, "Obsolete"),
            RecordStatus::Reserved => write!(f, "Reserved"),
        }
    }
}

/// Comprehensive information about a DNS record type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordType {
    /// The record type mnemonic (e.g., "A", "AAAA", "MX", "CNAME")
    pub name: String,

    /// Numeric type code as assigned by IANA
    pub type_code: u16,

    /// Short, human-readable description
    pub summary: String,

    /// Detailed description of purpose and usage
    pub description: String,

    /// The RDATA format or fields (e.g., "IPv4 address", "priority, mail server")
    pub rdata_format: String,

    /// Example record in zone file format
    pub example: String,

    /// Defining RFCs
    pub rfcs: Vec<String>,

    /// Current standardization status
    pub status: RecordStatus,

    /// Whether this record type is commonly encountered
    pub common: bool,

    /// Related record types (e.g., A is related to AAAA)
    pub related: Vec<String>,
}

/// Query and retrieve DNS record type information.
pub struct RecordTypeDatabase {
    entries: Vec<RecordType>,
}

impl RecordTypeDatabase {
    /// Create a new record type database from a list of entries.
    pub fn new(entries: Vec<RecordType>) -> Self {
        Self { entries }
    }

    /// Look up a record type by name (case-insensitive).
    pub fn lookup(&self, name: &str) -> Option<&RecordType> {
        let normalized = name.to_uppercase();
        self.entries.iter().find(|e| e.name == normalized)
    }

    /// Look up a record type by its numeric type code.
    pub fn lookup_by_code(&self, code: u16) -> Option<&RecordType> {
        self.entries.iter().find(|e| e.type_code == code)
    }

    /// Search record types by partial name, summary, or description match.
    pub fn search(&self, query: &str) -> Vec<&RecordType> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&query_lower)
                    || e.summary.to_lowercase().contains(&query_lower)
                    || e.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// List only the commonly used record types.
    pub fn common(&self) -> Vec<&RecordType> {
        self.entries.iter().filter(|e| e.common).collect()
    }

    /// Return the total number of record types in the database.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Return all record types.
    pub fn all(&self) -> &[RecordType] {
        &self.entries
    }
}
