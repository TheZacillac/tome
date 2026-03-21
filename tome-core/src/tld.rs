use serde::{Deserialize, Serialize};

/// Classification of a top-level domain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TldType {
    /// Generic TLD (e.g., .com, .org, .net)
    Generic,
    /// Country-code TLD (e.g., .uk, .de, .jp)
    CountryCode,
    /// Sponsored TLD (e.g., .gov, .edu, .museum)
    Sponsored,
    /// Infrastructure TLD (e.g., .arpa)
    Infrastructure,
    /// New generic TLD (e.g., .app, .dev, .xyz)
    NewGeneric,
    /// Generic-restricted TLD (e.g., .biz, .name, .pro)
    GenericRestricted,
    /// Test TLD (e.g., .test)
    Test,
}

impl std::fmt::Display for TldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TldType::Generic => write!(f, "Generic (gTLD)"),
            TldType::CountryCode => write!(f, "Country-Code (ccTLD)"),
            TldType::Sponsored => write!(f, "Sponsored (sTLD)"),
            TldType::Infrastructure => write!(f, "Infrastructure"),
            TldType::NewGeneric => write!(f, "New Generic (ngTLD)"),
            TldType::GenericRestricted => write!(f, "Generic-Restricted (grTLD)"),
            TldType::Test => write!(f, "Test"),
        }
    }
}

/// DNSSEC support status for a TLD.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DnssecSupport {
    /// DNSSEC is supported and signed
    Signed,
    /// DNSSEC is not supported
    Unsigned,
    /// DNSSEC status is unknown
    Unknown,
}

impl std::fmt::Display for DnssecSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnssecSupport::Signed => write!(f, "Signed"),
            DnssecSupport::Unsigned => write!(f, "Unsigned"),
            DnssecSupport::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Comprehensive information about a top-level domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tld {
    /// The TLD string (e.g., "com", "uk", "app")
    pub tld: String,

    /// Classification of this TLD
    pub tld_type: TldType,

    /// Full name or description
    pub name: String,

    /// The registry operator
    pub registry: String,

    /// WHOIS server hostname, if available
    pub whois_server: Option<String>,

    /// RDAP base URL, if available
    pub rdap_url: Option<String>,

    /// DNSSEC support status
    pub dnssec: DnssecSupport,

    /// IDN (Internationalized Domain Name) support
    pub idn_support: bool,

    /// Registration restrictions or policies, if any
    pub restrictions: Option<String>,

    /// Date the TLD was delegated to the root zone (YYYY-MM-DD)
    pub delegation_date: Option<String>,

    /// Relevant RFCs or IANA references
    pub references: Vec<String>,
}

/// Query and retrieve TLD information.
pub struct TldDatabase {
    entries: Vec<Tld>,
}

impl TldDatabase {
    /// Create a new TLD database from a list of entries.
    pub fn new(entries: Vec<Tld>) -> Self {
        Self { entries }
    }

    /// Look up a TLD by its string (case-insensitive).
    pub fn lookup(&self, tld: &str) -> Option<&Tld> {
        let normalized = tld.to_lowercase().trim_start_matches('.').to_string();
        self.entries.iter().find(|e| e.tld == normalized)
    }

    /// Search TLDs by partial name or description match.
    pub fn search(&self, query: &str) -> Vec<&Tld> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.tld.contains(&query_lower)
                    || e.name.to_lowercase().contains(&query_lower)
                    || e.registry.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// List all TLDs of a given type.
    pub fn by_type(&self, tld_type: &TldType) -> Vec<&Tld> {
        self.entries
            .iter()
            .filter(|e| &e.tld_type == tld_type)
            .collect()
    }

    /// Return the total number of TLDs in the database.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Return all TLDs.
    pub fn all(&self) -> &[Tld] {
        &self.entries
    }
}
