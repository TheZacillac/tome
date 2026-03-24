use serde::{Deserialize, Serialize};

/// Category grouping for glossary terms.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlossaryCategory {
    /// Core DNS protocol concepts
    Dns,
    /// Domain registration and management
    Registration,
    /// DNSSEC and security
    Security,
    /// Domain abuse and brand protection
    Abuse,
    /// DNS infrastructure and operations
    Infrastructure,
    /// Protocols (EPP, WHOIS, RDAP, etc.)
    Protocol,
    /// General industry terminology
    General,
}

impl std::fmt::Display for GlossaryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlossaryCategory::Dns => write!(f, "DNS"),
            GlossaryCategory::Registration => write!(f, "Registration"),
            GlossaryCategory::Security => write!(f, "Security"),
            GlossaryCategory::Abuse => write!(f, "Abuse"),
            GlossaryCategory::Infrastructure => write!(f, "Infrastructure"),
            GlossaryCategory::Protocol => write!(f, "Protocol"),
            GlossaryCategory::General => write!(f, "General"),
        }
    }
}

/// A glossary entry defining a term used in the domain name industry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTerm {
    /// The term being defined
    pub term: String,

    /// Common abbreviation or acronym, if applicable
    pub abbreviation: Option<String>,

    /// Short, one-line definition
    pub summary: String,

    /// Detailed definition and explanation
    pub description: String,

    /// Category this term belongs to
    pub category: GlossaryCategory,

    /// Related glossary terms
    pub related: Vec<String>,

    /// Relevant RFCs or external references
    pub references: Vec<String>,
}

/// Query and retrieve glossary terms.
pub struct GlossaryDatabase {
    entries: Vec<GlossaryTerm>,
}

impl GlossaryDatabase {
    /// Create a new glossary database from a list of entries.
    pub fn new(entries: Vec<GlossaryTerm>) -> Self {
        Self { entries }
    }

    /// Look up a term by exact name or abbreviation (case-insensitive).
    pub fn lookup(&self, term: &str) -> Option<&GlossaryTerm> {
        let normalized = term.to_lowercase();
        self.entries.iter().find(|e| {
            e.term.to_lowercase() == normalized
                || e.abbreviation
                    .as_ref()
                    .is_some_and(|a| a.to_lowercase() == normalized)
        })
    }

    /// Search glossary terms by partial match on term, abbreviation, summary, or description.
    pub fn search(&self, query: &str) -> Vec<&GlossaryTerm> {
        let query_lower = query.to_lowercase();
        if query_lower.trim().is_empty() {
            return Vec::new();
        }
        self.entries
            .iter()
            .filter(|e| {
                e.term.to_lowercase().contains(&query_lower)
                    || e.abbreviation
                        .as_ref()
                        .is_some_and(|a| a.to_lowercase().contains(&query_lower))
                    || e.summary.to_lowercase().contains(&query_lower)
                    || e.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// List all terms in a given category.
    pub fn by_category(&self, category: &GlossaryCategory) -> Vec<&GlossaryTerm> {
        self.entries
            .iter()
            .filter(|e| &e.category == category)
            .collect()
    }

    /// Return the total number of glossary terms.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Return all glossary terms.
    pub fn all(&self) -> &[GlossaryTerm] {
        &self.entries
    }
}
