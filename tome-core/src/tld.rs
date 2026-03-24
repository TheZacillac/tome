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

/// Contact requirements for a specific contact type (registrant, admin, billing, tech).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactRequirements {
    /// Contact type: "registrant", "admin", "billing", or "tech"
    pub contact_type: String,
    /// Minimum number of contacts required
    #[serde(default)]
    pub min: Option<u32>,
    /// Maximum number of contacts allowed
    #[serde(default)]
    pub max: Option<u32>,
    /// Whether this contact type is required
    #[serde(default)]
    pub required: Option<bool>,
    /// Whether an organization name is allowed
    #[serde(default)]
    pub organization_allowed: Option<bool>,
    /// Whether an organization name is required
    #[serde(default)]
    pub organization_required: Option<bool>,
    /// Contact scope: "registry", "registry_partial", "registrar", "ignored"
    #[serde(default)]
    pub scope: Option<String>,
}

/// Comprehensive information about a top-level domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tld {
    // -- Core identity --
    /// The TLD string (e.g., "com", "uk", "app")
    pub tld: String,
    /// Classification of this TLD
    pub tld_type: TldType,
    /// Full name or description
    pub name: String,
    /// The registry operator
    pub registry: String,

    // -- IANA reference data --
    /// WHOIS server hostname, if available
    #[serde(default)]
    pub whois_server: Option<String>,
    /// RDAP base URL, if available
    #[serde(default)]
    pub rdap_url: Option<String>,
    /// DNSSEC support status
    pub dnssec: DnssecSupport,
    /// Date the TLD was delegated to the root zone (YYYY-MM-DD)
    #[serde(default)]
    pub delegation_date: Option<String>,
    /// Relevant RFCs or IANA references
    pub references: Vec<String>,

    // -- Domain syntax --
    /// IDN (Internationalized Domain Name) support
    pub idn_support: bool,
    /// IDN standard type (e.g., "IDNA2003", "IDNA2008", "UTS46", "UTS46_2003")
    #[serde(default)]
    pub idn_type: Option<String>,
    /// Minimum domain label length
    #[serde(default)]
    pub min_length: Option<u32>,
    /// Maximum domain label length
    #[serde(default)]
    pub max_length: Option<u32>,

    // -- Nameserver requirements --
    /// Minimum number of nameservers required
    #[serde(default)]
    pub nameservers_min: Option<u32>,
    /// Maximum number of nameservers allowed
    #[serde(default)]
    pub nameservers_max: Option<u32>,

    // -- Registration constraints --
    /// Registration restrictions or policies, if any
    #[serde(default)]
    pub restrictions: Option<String>,
    /// ISO country codes allowed for registration (empty = no restrictions)
    pub allowed_countries: Vec<String>,
    /// Whether domain transfer requires an auth code
    #[serde(default)]
    pub transfer_auth_required: Option<bool>,
    /// Whether creation requires prior registrant validation
    #[serde(default)]
    pub creation_requires_prevalidation: Option<bool>,
    /// Validation category if prevalidation is required
    #[serde(default)]
    pub validation_category: Option<String>,

    // -- Periods (in years unless noted) --
    /// Supported registration period lengths
    pub create_periods: Vec<u32>,
    /// Supported renewal period lengths
    pub renew_periods: Vec<u32>,
    /// Supported auto-renewal period lengths
    pub auto_renew_periods: Vec<u32>,
    /// Supported transfer period lengths
    pub transfer_periods: Vec<u32>,
    /// Redemption grace period in days
    #[serde(default)]
    pub redemption_period: Option<u32>,
    /// Add grace period in days
    #[serde(default)]
    pub add_grace_period: Option<u32>,
    /// Renew grace period in days
    #[serde(default)]
    pub renew_grace_period: Option<u32>,
    /// Auto-renew grace period in days
    #[serde(default)]
    pub auto_renew_grace_period: Option<u32>,
    /// Transfer grace period in days
    #[serde(default)]
    pub transfer_grace_period: Option<u32>,
    /// Pending delete period in days
    #[serde(default)]
    pub pending_delete_period: Option<u32>,

    // -- Features --
    /// Available features (e.g., CREATE, RENEW, TRANSFER, RESTORE, PRIVACY_PROTECT, REGISTRY_LOCK)
    pub features: Vec<String>,
    /// Premium domain support: "no", "regular", "create_only", "transfer_from_registry"
    #[serde(default)]
    pub premium_support: Option<String>,
    /// Whether restore fee includes a renewal
    #[serde(default)]
    pub restore_includes_renew: Option<bool>,
    /// Renewal behavior on transfer: "no_change", "renew_unless_grace", "renewal", "new_period"
    #[serde(default)]
    pub renewal_on_transfer: Option<String>,

    // -- Contact requirements --
    /// Contact requirements per type
    pub contacts: Vec<ContactRequirements>,

    // -- DNSSEC --
    /// Maximum number of DNSSEC records allowed
    #[serde(default)]
    pub allowed_dnssec_records: Option<u32>,
    /// Supported DNSSEC algorithm numbers
    pub allowed_dnssec_algorithms: Vec<u32>,

    // -- Privacy --
    /// WHOIS data exposure level: "none", "limited", "full", "unknown"
    #[serde(default)]
    pub whois_exposure: Option<String>,
    /// GDPR category: "eu_based", "adequacy", "data_export", "unknown"
    #[serde(default)]
    pub gdpr_category: Option<String>,
    /// Legal jurisdiction for registrations
    #[serde(default)]
    pub jurisdiction: Option<String>,
}

/// Query and retrieve TLD information.
pub struct TldDatabase {
    entries: Vec<Tld>,
}

impl TldDatabase {
    /// Create a new TLD database from a list of entries.
    /// Normalizes all TLD strings to lowercase with no leading dot.
    pub fn new(entries: Vec<Tld>) -> Self {
        let entries = entries
            .into_iter()
            .map(|mut e| {
                e.tld = e.tld.to_lowercase().trim_start_matches('.').to_string();
                e
            })
            .collect();
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
        let query_stripped = query_lower.trim_start_matches('.');
        if query_stripped.is_empty() {
            return Vec::new();
        }
        self.entries
            .iter()
            .filter(|e| {
                e.tld.contains(query_stripped)
                    || e.name.to_lowercase().contains(query_stripped)
                    || e.registry.to_lowercase().contains(query_stripped)
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
