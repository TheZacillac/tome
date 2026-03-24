use rusqlite::{params, Connection, Row};
use std::path::Path;

use crate::error::Result;

/// The embedded schema SQL, compiled into the binary.
const SCHEMA_SQL: &str = include_str!("../schema/tld_schema.sql");

// ---------------------------------------------------------------------------
// TomeDb — main database handle
// ---------------------------------------------------------------------------

/// SQLite-backed TLD reference database.
///
/// Wraps a `rusqlite::Connection` and provides typed query methods for all
/// tables defined in the TLD schema.
pub struct TomeDb {
    conn: Connection,
}

impl TomeDb {
    /// Open (or create) a database at the given file path and initialize the schema.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Create an in-memory database (useful for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Run the schema DDL. Safe to call repeatedly — all statements use
    /// `IF NOT EXISTS`.
    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        self.conn.execute_batch(SCHEMA_SQL)?;
        Ok(())
    }

    /// Return a reference to the underlying connection for advanced use.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    // -----------------------------------------------------------------------
    // TLDs — core table
    // -----------------------------------------------------------------------

    /// Insert a TLD into the core `tlds` table. Returns the new row id.
    pub fn insert_tld(&self, tld: &TldRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tlds (tld, tld_ascii, tld_type, delegation_status, iana_id,
                               iana_delegation_date, iana_last_updated, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                tld.tld,
                tld.tld_ascii,
                tld.tld_type,
                tld.delegation_status,
                tld.iana_id,
                tld.iana_delegation_date,
                tld.iana_last_updated,
                tld.description,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Look up a TLD by its label (case-insensitive, strips leading dot).
    pub fn get_tld(&self, tld: &str) -> Result<Option<TldRow>> {
        let normalized = tld.to_lowercase().trim_start_matches('.').to_string();
        let mut stmt = self.conn.prepare(
            "SELECT id, tld, tld_ascii, dot_tld, tld_type, delegation_status,
                    iana_id, iana_delegation_date, iana_last_updated, description,
                    created_at, updated_at
             FROM tlds WHERE tld = ?1",
        )?;
        let row = stmt.query_row(params![normalized], TldRow::from_row).optional()?;
        Ok(row)
    }

    /// Search TLDs by partial match on tld label or description.
    pub fn search_tlds(&self, query: &str) -> Result<Vec<TldRow>> {
        let pattern = format!("%{}%", query.to_lowercase());
        let mut stmt = self.conn.prepare(
            "SELECT id, tld, tld_ascii, dot_tld, tld_type, delegation_status,
                    iana_id, iana_delegation_date, iana_last_updated, description,
                    created_at, updated_at
             FROM tlds
             WHERE tld LIKE ?1 OR LOWER(description) LIKE ?1
             ORDER BY tld",
        )?;
        let rows = stmt
            .query_map(params![pattern], TldRow::from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List all TLDs of a given type (e.g. "gTLD", "ccTLD", "nTLD").
    pub fn list_tlds_by_type(&self, tld_type: &str) -> Result<Vec<TldRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tld, tld_ascii, dot_tld, tld_type, delegation_status,
                    iana_id, iana_delegation_date, iana_last_updated, description,
                    created_at, updated_at
             FROM tlds WHERE tld_type = ?1 ORDER BY tld",
        )?;
        let rows = stmt
            .query_map(params![tld_type], TldRow::from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List all TLDs.
    pub fn list_tlds(&self) -> Result<Vec<TldRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tld, tld_ascii, dot_tld, tld_type, delegation_status,
                    iana_id, iana_delegation_date, iana_last_updated, description,
                    created_at, updated_at
             FROM tlds ORDER BY tld",
        )?;
        let rows = stmt
            .query_map([], TldRow::from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Return the total number of TLDs.
    pub fn count_tlds(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tlds", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    // -----------------------------------------------------------------------
    // Registry Operators
    // -----------------------------------------------------------------------

    /// Insert a registry operator. Returns the new row id.
    pub fn insert_registry_operator(&self, op: &RegistryOperatorRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO registry_operators
                (name, short_name, website, abuse_contact_email, abuse_contact_phone,
                 hq_country, parent_company, icann_accredited, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                op.name,
                op.short_name,
                op.website,
                op.abuse_contact_email,
                op.abuse_contact_phone,
                op.hq_country,
                op.parent_company,
                op.icann_accredited,
                op.notes,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Link a TLD to a registry operator.
    pub fn link_tld_registry(
        &self,
        tld_id: i64,
        registry_operator_id: i64,
        role: &str,
        effective_date: Option<&str>,
        registry_agreement_url: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tld_registry (tld_id, registry_operator_id, role, effective_date, registry_agreement_url)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tld_id, registry_operator_id, role, effective_date, registry_agreement_url],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Country Mapping
    // -----------------------------------------------------------------------

    /// Insert a country mapping for a ccTLD.
    pub fn insert_country_mapping(&self, cm: &CountryMappingRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tld_country_mapping
                (tld_id, iso_3166_alpha2, iso_3166_alpha3, iso_3166_numeric,
                 country_name, sovereign_state, local_presence_required, local_presence_notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                cm.tld_id,
                cm.iso_3166_alpha2,
                cm.iso_3166_alpha3,
                cm.iso_3166_numeric,
                cm.country_name,
                cm.sovereign_state,
                cm.local_presence_required,
                cm.local_presence_notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Registration Policy
    // -----------------------------------------------------------------------

    /// Insert or replace registration policy for a TLD.
    pub fn upsert_registration_policy(&self, rp: &RegistrationPolicyRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tld_registration_policy
                (tld_id, registration_model, eligibility_requirements,
                 requires_local_presence, requires_trademark, requires_sector_nexus,
                 sector_restrictions, min_registration_years, max_registration_years,
                 supports_privacy_proxy, sunrise_start, sunrise_end,
                 landrush_start, landrush_end, general_availability, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                rp.tld_id,
                rp.registration_model,
                rp.eligibility_requirements,
                rp.requires_local_presence,
                rp.requires_trademark,
                rp.requires_sector_nexus,
                rp.sector_restrictions,
                rp.min_registration_years,
                rp.max_registration_years,
                rp.supports_privacy_proxy,
                rp.sunrise_start,
                rp.sunrise_end,
                rp.landrush_start,
                rp.landrush_end,
                rp.general_availability,
                rp.notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Technical Specifications
    // -----------------------------------------------------------------------

    /// Insert or replace technical specs for a TLD.
    pub fn upsert_technical(&self, t: &TechnicalRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tld_technical
                (tld_id, min_label_length, max_label_length, allowed_characters,
                 allows_idn, idn_scripts_supported, idn_language_tags,
                 supports_dnssec, dnssec_algorithms, min_nameservers, max_nameservers,
                 supports_ds_records, supports_glue_records, epp_repository_id,
                 supports_epp_transfer, supports_authcode, supports_rgp,
                 supports_bulk_operations, registry_lock_available, registrar_lock_available,
                 notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                t.tld_id,
                t.min_label_length,
                t.max_label_length,
                t.allowed_characters,
                t.allows_idn,
                t.idn_scripts_supported,
                t.idn_language_tags,
                t.supports_dnssec,
                t.dnssec_algorithms,
                t.min_nameservers,
                t.max_nameservers,
                t.supports_ds_records,
                t.supports_glue_records,
                t.epp_repository_id,
                t.supports_epp_transfer,
                t.supports_authcode,
                t.supports_rgp,
                t.supports_bulk_operations,
                t.registry_lock_available,
                t.registrar_lock_available,
                t.notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // WHOIS / RDAP
    // -----------------------------------------------------------------------

    /// Insert or replace WHOIS/RDAP info for a TLD.
    pub fn upsert_whois_rdap(&self, w: &WhoisRdapRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tld_whois_rdap
                (tld_id, whois_server, whois_port, whois_model, rdap_base_url,
                 rdap_conformance, gdpr_redacted, gdpr_redaction_notes, rate_limit_info, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                w.tld_id,
                w.whois_server,
                w.whois_port,
                w.whois_model,
                w.rdap_base_url,
                w.rdap_conformance,
                w.gdpr_redacted,
                w.gdpr_redaction_notes,
                w.rate_limit_info,
                w.notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Insert or replace lifecycle info for a TLD.
    pub fn upsert_lifecycle(&self, lc: &LifecycleRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tld_lifecycle
                (tld_id, add_grace_period_days, renew_grace_period_days,
                 auto_renew_grace_period_days, transfer_lock_days,
                 redemption_grace_period_days, pending_delete_days,
                 expiration_policy, transfer_cooldown_days, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                lc.tld_id,
                lc.add_grace_period_days,
                lc.renew_grace_period_days,
                lc.auto_renew_grace_period_days,
                lc.transfer_lock_days,
                lc.redemption_grace_period_days,
                lc.pending_delete_days,
                lc.expiration_policy,
                lc.transfer_cooldown_days,
                lc.notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Pricing
    // -----------------------------------------------------------------------

    /// Insert a pricing entry for a TLD. Returns the new row id.
    pub fn insert_pricing(&self, p: &PricingRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tld_pricing
                (tld_id, price_type, currency, wholesale_price, retail_price_low,
                 retail_price_high, premium_tier, effective_date, end_date, source, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                p.tld_id,
                p.price_type,
                p.currency,
                p.wholesale_price,
                p.retail_price_low,
                p.retail_price_high,
                p.premium_tier,
                p.effective_date,
                p.end_date,
                p.source,
                p.notes,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // -----------------------------------------------------------------------
    // Brand Protection
    // -----------------------------------------------------------------------

    /// Insert or replace brand protection info for a TLD.
    pub fn upsert_brand_protection(&self, bp: &BrandProtectionRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tld_brand_protection
                (tld_id, in_tmch, claims_period_active, claims_period_end,
                 supports_dpml, dpml_provider, supports_adndrc,
                 phishing_abuse_risk, typosquatting_risk,
                 commonly_blocked_corporate, defensive_registration_recommended, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                bp.tld_id,
                bp.in_tmch,
                bp.claims_period_active,
                bp.claims_period_end,
                bp.supports_dpml,
                bp.dpml_provider,
                bp.supports_adndrc,
                bp.phishing_abuse_risk,
                bp.typosquatting_risk,
                bp.commonly_blocked_corporate,
                bp.defensive_registration_recommended,
                bp.notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Transfer Rules
    // -----------------------------------------------------------------------

    /// Insert or replace transfer rules for a TLD.
    pub fn upsert_transfer_rules(&self, tr: &TransferRulesRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tld_transfer_rules
                (tld_id, inter_registrar_transfer, requires_authcode, authcode_format,
                 authcode_validity_days, transfer_approval_method, approval_timeout_days,
                 new_registration_lock_days, post_transfer_lock_days, transfer_adds_year,
                 bulk_transfer_supported, registry_transfer_supported,
                 custom_transfer_process, custom_process_notes, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                tr.tld_id,
                tr.inter_registrar_transfer,
                tr.requires_authcode,
                tr.authcode_format,
                tr.authcode_validity_days,
                tr.transfer_approval_method,
                tr.approval_timeout_days,
                tr.new_registration_lock_days,
                tr.post_transfer_lock_days,
                tr.transfer_adds_year,
                tr.bulk_transfer_supported,
                tr.registry_transfer_supported,
                tr.custom_transfer_process,
                tr.custom_process_notes,
                tr.notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Security
    // -----------------------------------------------------------------------

    /// Insert or replace security info for a TLD.
    pub fn upsert_security(&self, s: &SecurityRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tld_security
                (tld_id, zone_signed_dnssec, ds_digest_types, key_signing_algorithm,
                 zone_signing_algorithm, hsts_preloaded,
                 certificate_transparency_enforced, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                s.tld_id,
                s.zone_signed_dnssec,
                s.ds_digest_types,
                s.key_signing_algorithm,
                s.zone_signing_algorithm,
                s.hsts_preloaded,
                s.certificate_transparency_enforced,
                s.notes,
            ],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Nameservers
    // -----------------------------------------------------------------------

    /// Insert a nameserver entry for a TLD. Returns the new row id.
    pub fn insert_nameserver(&self, ns: &NameserverRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tld_nameservers
                (tld_id, hostname, ipv4_address, ipv6_address, anycast, operator, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                ns.tld_id,
                ns.hostname,
                ns.ipv4_address,
                ns.ipv6_address,
                ns.anycast,
                ns.operator,
                ns.notes,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // -----------------------------------------------------------------------
    // Dispute Policies
    // -----------------------------------------------------------------------

    /// Insert a dispute policy for a TLD. Returns the new row id.
    pub fn insert_dispute_policy(&self, dp: &DisputePolicyRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tld_dispute_policies
                (tld_id, policy_type, policy_name, policy_url, provider, mandatory, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                dp.tld_id,
                dp.policy_type,
                dp.policy_name,
                dp.policy_url,
                dp.provider,
                dp.mandatory,
                dp.notes,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // -----------------------------------------------------------------------
    // Market Data
    // -----------------------------------------------------------------------

    /// Insert a market data snapshot. Returns the new row id.
    pub fn insert_market_data(&self, md: &MarketDataRow) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tld_market_data
                (tld_id, snapshot_date, total_domains, net_adds_month, renewal_rate_pct,
                 registrar_count, tranco_top1m_count, parking_rate_pct, source, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                md.tld_id,
                md.snapshot_date,
                md.total_domains,
                md.net_adds_month,
                md.renewal_rate_pct,
                md.registrar_count,
                md.tranco_top1m_count,
                md.parking_rate_pct,
                md.source,
                md.notes,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // -----------------------------------------------------------------------
    // Overview View
    // -----------------------------------------------------------------------

    /// Query the `v_tld_overview` view for a single TLD.
    pub fn get_tld_overview(&self, tld: &str) -> Result<Option<TldOverviewRow>> {
        let normalized = tld.to_lowercase().trim_start_matches('.').to_string();
        let mut stmt = self.conn.prepare(
            "SELECT tld, tld_type, delegation_status, registry_operator,
                    country_name, iso_3166_alpha2, registration_model,
                    requires_local_presence, supports_dnssec, allows_idn,
                    registry_lock_available, whois_server, rdap_base_url,
                    phishing_abuse_risk, defensive_registration_recommended
             FROM v_tld_overview WHERE tld = ?1",
        )?;
        let row = stmt
            .query_row(params![normalized], TldOverviewRow::from_row)
            .optional()?;
        Ok(row)
    }

    /// Query the `v_tld_overview` view for all TLDs.
    pub fn list_tld_overviews(&self) -> Result<Vec<TldOverviewRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT tld, tld_type, delegation_status, registry_operator,
                    country_name, iso_3166_alpha2, registration_model,
                    requires_local_presence, supports_dnssec, allows_idn,
                    registry_lock_available, whois_server, rdap_base_url,
                    phishing_abuse_risk, defensive_registration_recommended
             FROM v_tld_overview ORDER BY tld",
        )?;
        let rows = stmt
            .query_map([], TldOverviewRow::from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // -----------------------------------------------------------------------
    // Change Log
    // -----------------------------------------------------------------------

    /// Record a change in the audit log.
    pub fn log_change(&self, entry: &ChangeLogEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tld_change_log
                (tld_id, changed_table, changed_field, old_value, new_value, changed_by, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.tld_id,
                entry.changed_table,
                entry.changed_field,
                entry.old_value,
                entry.new_value,
                entry.changed_by,
                entry.source,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
}

// ---------------------------------------------------------------------------
// Row types — thin structs mapping 1:1 to database tables
// ---------------------------------------------------------------------------

/// Row from the `tlds` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TldRow {
    pub id: Option<i64>,
    pub tld: String,
    pub tld_ascii: Option<String>,
    pub dot_tld: Option<String>,
    pub tld_type: String,
    pub delegation_status: String,
    pub iana_id: Option<String>,
    pub iana_delegation_date: Option<String>,
    pub iana_last_updated: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl TldRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            tld: row.get(1)?,
            tld_ascii: row.get(2)?,
            dot_tld: row.get(3)?,
            tld_type: row.get(4)?,
            delegation_status: row.get(5)?,
            iana_id: row.get(6)?,
            iana_delegation_date: row.get(7)?,
            iana_last_updated: row.get(8)?,
            description: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

/// Row from the `registry_operators` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryOperatorRow {
    pub id: Option<i64>,
    pub name: String,
    pub short_name: Option<String>,
    pub website: Option<String>,
    pub abuse_contact_email: Option<String>,
    pub abuse_contact_phone: Option<String>,
    pub hq_country: Option<String>,
    pub parent_company: Option<String>,
    pub icann_accredited: bool,
    pub notes: Option<String>,
}

/// Row from the `tld_country_mapping` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CountryMappingRow {
    pub tld_id: i64,
    pub iso_3166_alpha2: String,
    pub iso_3166_alpha3: Option<String>,
    pub iso_3166_numeric: Option<i32>,
    pub country_name: String,
    pub sovereign_state: Option<String>,
    pub local_presence_required: bool,
    pub local_presence_notes: Option<String>,
}

/// Row from the `tld_registration_policy` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistrationPolicyRow {
    pub tld_id: i64,
    pub registration_model: String,
    pub eligibility_requirements: Option<String>,
    pub requires_local_presence: bool,
    pub requires_trademark: bool,
    pub requires_sector_nexus: bool,
    pub sector_restrictions: Option<String>,
    pub min_registration_years: i32,
    pub max_registration_years: i32,
    pub supports_privacy_proxy: bool,
    pub sunrise_start: Option<String>,
    pub sunrise_end: Option<String>,
    pub landrush_start: Option<String>,
    pub landrush_end: Option<String>,
    pub general_availability: Option<String>,
    pub notes: Option<String>,
}

/// Row from the `tld_technical` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TechnicalRow {
    pub tld_id: i64,
    pub min_label_length: i32,
    pub max_label_length: i32,
    pub allowed_characters: Option<String>,
    pub allows_idn: bool,
    /// JSON array string, e.g. `["Latin","Cyrillic"]`
    pub idn_scripts_supported: Option<String>,
    /// JSON array string, e.g. `["zh-Hans","ja"]`
    pub idn_language_tags: Option<String>,
    pub supports_dnssec: bool,
    /// JSON array string, e.g. `["RSA/SHA-256"]`
    pub dnssec_algorithms: Option<String>,
    pub min_nameservers: i32,
    pub max_nameservers: i32,
    pub supports_ds_records: bool,
    pub supports_glue_records: bool,
    pub epp_repository_id: Option<String>,
    pub supports_epp_transfer: bool,
    pub supports_authcode: bool,
    pub supports_rgp: bool,
    pub supports_bulk_operations: Option<bool>,
    pub registry_lock_available: bool,
    pub registrar_lock_available: bool,
    pub notes: Option<String>,
}

/// Row from the `tld_whois_rdap` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhoisRdapRow {
    pub tld_id: i64,
    pub whois_server: Option<String>,
    pub whois_port: i32,
    pub whois_model: Option<String>,
    pub rdap_base_url: Option<String>,
    /// JSON array string
    pub rdap_conformance: Option<String>,
    pub gdpr_redacted: bool,
    pub gdpr_redaction_notes: Option<String>,
    pub rate_limit_info: Option<String>,
    pub notes: Option<String>,
}

/// Row from the `tld_lifecycle` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LifecycleRow {
    pub tld_id: i64,
    pub add_grace_period_days: Option<i32>,
    pub renew_grace_period_days: Option<i32>,
    pub auto_renew_grace_period_days: Option<i32>,
    pub transfer_lock_days: Option<i32>,
    pub redemption_grace_period_days: Option<i32>,
    pub pending_delete_days: Option<i32>,
    pub expiration_policy: Option<String>,
    pub transfer_cooldown_days: Option<i32>,
    pub notes: Option<String>,
}

/// Row from the `tld_pricing` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PricingRow {
    pub id: Option<i64>,
    pub tld_id: i64,
    pub price_type: String,
    pub currency: String,
    pub wholesale_price: Option<f64>,
    pub retail_price_low: Option<f64>,
    pub retail_price_high: Option<f64>,
    pub premium_tier: Option<String>,
    pub effective_date: Option<String>,
    pub end_date: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// Row from the `tld_brand_protection` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrandProtectionRow {
    pub tld_id: i64,
    pub in_tmch: bool,
    pub claims_period_active: bool,
    pub claims_period_end: Option<String>,
    pub supports_dpml: bool,
    pub dpml_provider: Option<String>,
    pub supports_adndrc: bool,
    pub phishing_abuse_risk: String,
    pub typosquatting_risk: String,
    pub commonly_blocked_corporate: bool,
    pub defensive_registration_recommended: bool,
    pub notes: Option<String>,
}

/// Row from the `tld_nameservers` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NameserverRow {
    pub id: Option<i64>,
    pub tld_id: i64,
    pub hostname: String,
    pub ipv4_address: Option<String>,
    pub ipv6_address: Option<String>,
    pub anycast: bool,
    pub operator: Option<String>,
    pub notes: Option<String>,
}

/// Row from the `tld_dispute_policies` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DisputePolicyRow {
    pub id: Option<i64>,
    pub tld_id: i64,
    pub policy_type: String,
    pub policy_name: Option<String>,
    pub policy_url: Option<String>,
    pub provider: Option<String>,
    pub mandatory: bool,
    pub notes: Option<String>,
}

/// Row from the `tld_transfer_rules` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransferRulesRow {
    pub tld_id: i64,
    pub inter_registrar_transfer: bool,
    pub requires_authcode: bool,
    pub authcode_format: Option<String>,
    pub authcode_validity_days: Option<i32>,
    pub transfer_approval_method: Option<String>,
    pub approval_timeout_days: Option<i32>,
    pub new_registration_lock_days: Option<i32>,
    pub post_transfer_lock_days: Option<i32>,
    pub transfer_adds_year: bool,
    pub bulk_transfer_supported: bool,
    pub registry_transfer_supported: bool,
    pub custom_transfer_process: bool,
    pub custom_process_notes: Option<String>,
    pub notes: Option<String>,
}

/// Row from the `tld_security` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityRow {
    pub tld_id: i64,
    pub zone_signed_dnssec: bool,
    /// JSON array string
    pub ds_digest_types: Option<String>,
    pub key_signing_algorithm: Option<String>,
    pub zone_signing_algorithm: Option<String>,
    pub hsts_preloaded: bool,
    pub certificate_transparency_enforced: Option<bool>,
    pub notes: Option<String>,
}

/// Row from the `tld_market_data` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketDataRow {
    pub id: Option<i64>,
    pub tld_id: i64,
    pub snapshot_date: String,
    pub total_domains: Option<i64>,
    pub net_adds_month: Option<i32>,
    pub renewal_rate_pct: Option<f64>,
    pub registrar_count: Option<i32>,
    pub tranco_top1m_count: Option<i32>,
    pub parking_rate_pct: Option<f64>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// Entry for the `tld_change_log` audit table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangeLogEntry {
    pub tld_id: i64,
    pub changed_table: String,
    pub changed_field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Option<String>,
    pub source: Option<String>,
}

/// Row from the `v_tld_overview` view.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TldOverviewRow {
    pub tld: String,
    pub tld_type: String,
    pub delegation_status: String,
    pub registry_operator: Option<String>,
    pub country_name: Option<String>,
    pub iso_3166_alpha2: Option<String>,
    pub registration_model: Option<String>,
    pub requires_local_presence: Option<bool>,
    pub supports_dnssec: Option<bool>,
    pub allows_idn: Option<bool>,
    pub registry_lock_available: Option<bool>,
    pub whois_server: Option<String>,
    pub rdap_base_url: Option<String>,
    pub phishing_abuse_risk: Option<String>,
    pub defensive_registration_recommended: Option<bool>,
}

impl TldOverviewRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            tld: row.get(0)?,
            tld_type: row.get(1)?,
            delegation_status: row.get(2)?,
            registry_operator: row.get(3)?,
            country_name: row.get(4)?,
            iso_3166_alpha2: row.get(5)?,
            registration_model: row.get(6)?,
            requires_local_presence: row.get(7)?,
            supports_dnssec: row.get(8)?,
            allows_idn: row.get(9)?,
            registry_lock_available: row.get(10)?,
            whois_server: row.get(11)?,
            rdap_base_url: row.get(12)?,
            phishing_abuse_risk: row.get(13)?,
            defensive_registration_recommended: row.get(14)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Convenience trait for optional query results
// ---------------------------------------------------------------------------

/// Extension trait to convert `QueryRowError::QueryReturnedNoRows` into `None`.
trait OptionalRow<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalRow<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = TomeDb::open_in_memory().expect("failed to open in-memory db");
        assert_eq!(db.count_tlds().expect("count failed"), 0);
    }

    #[test]
    fn test_insert_and_lookup_tld() {
        let db = TomeDb::open_in_memory().expect("failed to open db");

        let tld = TldRow {
            id: None,
            tld: "com".to_string(),
            tld_ascii: None,
            dot_tld: None,
            tld_type: "gTLD".to_string(),
            delegation_status: "active".to_string(),
            iana_id: Some("COM".to_string()),
            iana_delegation_date: Some("1985-01-01".to_string()),
            iana_last_updated: None,
            description: Some("Generic top-level domain".to_string()),
            created_at: None,
            updated_at: None,
        };

        let id = db.insert_tld(&tld).expect("insert failed");
        assert!(id > 0);

        let found = db.get_tld("com").expect("lookup failed");
        assert!(found.is_some());
        let found = found.expect("should exist");
        assert_eq!(found.tld, "com");
        assert_eq!(found.tld_type, "gTLD");
        assert_eq!(found.dot_tld.as_deref(), Some(".com"));

        // case-insensitive + leading dot
        let found2 = db.get_tld(".COM").expect("lookup failed");
        assert!(found2.is_some());
    }

    #[test]
    fn test_search_tlds() {
        let db = TomeDb::open_in_memory().expect("failed to open db");

        for (tld, desc) in [
            ("com", "Commercial"),
            ("net", "Network"),
            ("org", "Organization"),
        ] {
            db.insert_tld(&TldRow {
                id: None,
                tld: tld.to_string(),
                tld_ascii: None,
                dot_tld: None,
                tld_type: "gTLD".to_string(),
                delegation_status: "active".to_string(),
                iana_id: None,
                iana_delegation_date: None,
                iana_last_updated: None,
                description: Some(desc.to_string()),
                created_at: None,
                updated_at: None,
            })
            .expect("insert failed");
        }

        let results = db.search_tlds("net").expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tld, "net");

        let results = db.search_tlds("org").expect("search failed");
        // matches both "org" tld and "Organization" description
        assert!(!results.is_empty());
    }

    #[test]
    fn test_registry_operator_and_overview() {
        let db = TomeDb::open_in_memory().expect("failed to open db");

        let tld_id = db
            .insert_tld(&TldRow {
                id: None,
                tld: "com".to_string(),
                tld_ascii: None,
                dot_tld: None,
                tld_type: "gTLD".to_string(),
                delegation_status: "active".to_string(),
                iana_id: None,
                iana_delegation_date: None,
                iana_last_updated: None,
                description: None,
                created_at: None,
                updated_at: None,
            })
            .expect("insert tld failed");

        let op_id = db
            .insert_registry_operator(&RegistryOperatorRow {
                id: None,
                name: "VeriSign, Inc.".to_string(),
                short_name: Some("VeriSign".to_string()),
                website: None,
                abuse_contact_email: None,
                abuse_contact_phone: None,
                hq_country: Some("US".to_string()),
                parent_company: None,
                icann_accredited: true,
                notes: None,
            })
            .expect("insert operator failed");

        db.link_tld_registry(tld_id, op_id, "operator", None, None)
            .expect("link failed");

        db.upsert_whois_rdap(&WhoisRdapRow {
            tld_id,
            whois_server: Some("whois.verisign-grs.com".to_string()),
            whois_port: 43,
            whois_model: Some("thin".to_string()),
            rdap_base_url: Some("https://rdap.verisign.com/com/v1/".to_string()),
            rdap_conformance: None,
            gdpr_redacted: true,
            gdpr_redaction_notes: None,
            rate_limit_info: None,
            notes: None,
        })
        .expect("upsert whois failed");

        let overview = db.get_tld_overview("com").expect("overview failed");
        assert!(overview.is_some());
        let ov = overview.expect("should exist");
        assert_eq!(ov.tld, "com");
        assert_eq!(ov.registry_operator.as_deref(), Some("VeriSign, Inc."));
        assert_eq!(
            ov.whois_server.as_deref(),
            Some("whois.verisign-grs.com")
        );
    }

    #[test]
    fn test_count_and_list() {
        let db = TomeDb::open_in_memory().expect("failed to open db");

        for tld in ["com", "net", "uk"] {
            let tld_type = if tld == "uk" { "ccTLD" } else { "gTLD" };
            db.insert_tld(&TldRow {
                id: None,
                tld: tld.to_string(),
                tld_ascii: None,
                dot_tld: None,
                tld_type: tld_type.to_string(),
                delegation_status: "active".to_string(),
                iana_id: None,
                iana_delegation_date: None,
                iana_last_updated: None,
                description: None,
                created_at: None,
                updated_at: None,
            })
            .expect("insert failed");
        }

        assert_eq!(db.count_tlds().expect("count"), 3);

        let gtlds = db.list_tlds_by_type("gTLD").expect("by_type");
        assert_eq!(gtlds.len(), 2);

        let cctlds = db.list_tlds_by_type("ccTLD").expect("by_type");
        assert_eq!(cctlds.len(), 1);
        assert_eq!(cctlds[0].tld, "uk");
    }

    #[test]
    fn test_change_log() {
        let db = TomeDb::open_in_memory().expect("failed to open db");

        let tld_id = db
            .insert_tld(&TldRow {
                id: None,
                tld: "com".to_string(),
                tld_ascii: None,
                dot_tld: None,
                tld_type: "gTLD".to_string(),
                delegation_status: "active".to_string(),
                iana_id: None,
                iana_delegation_date: None,
                iana_last_updated: None,
                description: None,
                created_at: None,
                updated_at: None,
            })
            .expect("insert failed");

        let log_id = db
            .log_change(&ChangeLogEntry {
                tld_id,
                changed_table: "tlds".to_string(),
                changed_field: Some("description".to_string()),
                old_value: None,
                new_value: Some("Generic TLD".to_string()),
                changed_by: Some("system".to_string()),
                source: Some("initial load".to_string()),
            })
            .expect("log_change failed");

        assert!(log_id > 0);
    }
}
