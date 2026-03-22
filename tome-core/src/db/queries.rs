use rusqlite::{params, Connection};

use crate::error::Result;
use crate::glossary::{GlossaryCategory, GlossaryTerm};
use crate::record_type::{RecordStatus, RecordType};
use crate::tld::{ContactRequirements, DnssecSupport, Tld, TldType};

// ---------------------------------------------------------------------------
// TLD queries
// ---------------------------------------------------------------------------

const TLD_COLUMNS: &str =
    "tld, tld_type, name, registry, whois_server, rdap_url, dnssec, delegation_date,
     idn_support, idn_type, min_length, max_length,
     nameservers_min, nameservers_max,
     restrictions, allowed_countries, transfer_auth_required,
     creation_requires_prevalidation, validation_category,
     create_periods, renew_periods, auto_renew_periods, transfer_periods,
     redemption_period, add_grace_period, renew_grace_period,
     auto_renew_grace_period, transfer_grace_period, pending_delete_period,
     features, premium_support, restore_includes_renew, renewal_on_transfer,
     allowed_dnssec_records, allowed_dnssec_algorithms,
     whois_exposure, gdpr_category, jurisdiction";

const TLD_COLUMNS_PREFIXED: &str =
    "t.tld, t.tld_type, t.name, t.registry, t.whois_server, t.rdap_url, t.dnssec, t.delegation_date,
     t.idn_support, t.idn_type, t.min_length, t.max_length,
     t.nameservers_min, t.nameservers_max,
     t.restrictions, t.allowed_countries, t.transfer_auth_required,
     t.creation_requires_prevalidation, t.validation_category,
     t.create_periods, t.renew_periods, t.auto_renew_periods, t.transfer_periods,
     t.redemption_period, t.add_grace_period, t.renew_grace_period,
     t.auto_renew_grace_period, t.transfer_grace_period, t.pending_delete_period,
     t.features, t.premium_support, t.restore_includes_renew, t.renewal_on_transfer,
     t.allowed_dnssec_records, t.allowed_dnssec_algorithms,
     t.whois_exposure, t.gdpr_category, t.jurisdiction";

/// Insert or replace a TLD entry along with its references and contacts.
pub fn upsert_tld(conn: &Connection, tld: &Tld) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT OR REPLACE INTO tlds (
                tld, tld_type, name, registry,
                whois_server, rdap_url, dnssec, delegation_date,
                idn_support, idn_type, min_length, max_length,
                nameservers_min, nameservers_max,
                restrictions, allowed_countries, transfer_auth_required,
                creation_requires_prevalidation, validation_category,
                create_periods, renew_periods, auto_renew_periods, transfer_periods,
                redemption_period, add_grace_period, renew_grace_period,
                auto_renew_grace_period, transfer_grace_period, pending_delete_period,
                features, premium_support, restore_includes_renew, renewal_on_transfer,
                allowed_dnssec_records, allowed_dnssec_algorithms,
                whois_exposure, gdpr_category, jurisdiction
            ) VALUES ({placeholders})",
            placeholders = (1..=38).map(|i| format!("?{i}")).collect::<Vec<_>>().join(", ")
        ),
        params![
            tld.tld,
            enum_to_str(&tld.tld_type),
            tld.name,
            tld.registry,
            tld.whois_server,
            tld.rdap_url,
            enum_to_str(&tld.dnssec),
            tld.delegation_date,
            tld.idn_support,
            tld.idn_type,
            tld.min_length,
            tld.max_length,
            tld.nameservers_min,
            tld.nameservers_max,
            tld.restrictions,
            serde_json::to_string(&tld.allowed_countries).unwrap_or_default(),
            tld.transfer_auth_required,
            tld.creation_requires_prevalidation,
            tld.validation_category,
            serde_json::to_string(&tld.create_periods).unwrap_or_default(),
            serde_json::to_string(&tld.renew_periods).unwrap_or_default(),
            serde_json::to_string(&tld.auto_renew_periods).unwrap_or_default(),
            serde_json::to_string(&tld.transfer_periods).unwrap_or_default(),
            tld.redemption_period,
            tld.add_grace_period,
            tld.renew_grace_period,
            tld.auto_renew_grace_period,
            tld.transfer_grace_period,
            tld.pending_delete_period,
            serde_json::to_string(&tld.features).unwrap_or_default(),
            tld.premium_support,
            tld.restore_includes_renew,
            tld.renewal_on_transfer,
            tld.allowed_dnssec_records,
            serde_json::to_string(&tld.allowed_dnssec_algorithms).unwrap_or_default(),
            tld.whois_exposure,
            tld.gdpr_category,
            tld.jurisdiction,
        ],
    )?;

    // Replace references
    conn.execute("DELETE FROM tld_references WHERE tld = ?1", [&tld.tld])?;
    {
        let mut stmt =
            conn.prepare("INSERT INTO tld_references (tld, reference) VALUES (?1, ?2)")?;
        for r in &tld.references {
            stmt.execute(params![tld.tld, r])?;
        }
    }

    // Replace contacts
    conn.execute("DELETE FROM tld_contacts WHERE tld = ?1", [&tld.tld])?;
    {
        let mut stmt = conn.prepare(
            "INSERT INTO tld_contacts (tld, contact_type, min, max, required, organization_allowed, organization_required, scope)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for c in &tld.contacts {
            stmt.execute(params![
                tld.tld,
                c.contact_type,
                c.min,
                c.max,
                c.required,
                c.organization_allowed,
                c.organization_required,
                c.scope,
            ])?;
        }
    }

    Ok(())
}

/// Look up a single TLD by name (case-insensitive).
pub fn get_tld(conn: &Connection, tld: &str) -> Result<Option<Tld>> {
    let normalized = tld.to_lowercase().trim_start_matches('.').to_string();
    let mut stmt = conn.prepare(&format!("SELECT {TLD_COLUMNS} FROM tlds WHERE tld = ?1"))?;

    let result = stmt.query_row([&normalized], row_to_tld_row);

    match result {
        Ok(row) => {
            let references = get_tld_references(conn, &row.tld)?;
            let contacts = get_tld_contacts(conn, &row.tld)?;
            Ok(Some(row.into_tld(references, contacts)))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Search TLDs using full-text search.
pub fn search_tlds(conn: &Connection, query: &str) -> Result<Vec<Tld>> {
    let fts_query = format!("\"{}\"*", query.replace('"', "\"\""));
    let mut stmt = conn.prepare(&format!(
        "SELECT {TLD_COLUMNS_PREFIXED} FROM tlds t
         JOIN tlds_fts f ON t.rowid = f.rowid
         WHERE tlds_fts MATCH ?1
         ORDER BY rank"
    ))?;

    let rows: Vec<TldRow> = stmt
        .query_map([&fts_query], row_to_tld_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    hydrate_tld_rows(conn, rows)
}

/// List all TLDs, optionally filtered by type.
pub fn list_tlds(conn: &Connection, tld_type: Option<&TldType>) -> Result<Vec<Tld>> {
    let type_str;
    let (sql, query_params): (String, Vec<&dyn rusqlite::types::ToSql>) = match tld_type {
        Some(t) => {
            type_str = enum_to_str(t);
            (
                format!("SELECT {TLD_COLUMNS} FROM tlds WHERE tld_type = ?1 ORDER BY tld"),
                vec![&type_str as &dyn rusqlite::types::ToSql],
            )
        }
        None => (
            format!("SELECT {TLD_COLUMNS} FROM tlds ORDER BY tld"),
            vec![],
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<TldRow> = stmt
        .query_map(query_params.as_slice(), row_to_tld_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    hydrate_tld_rows(conn, rows)
}

/// Count TLDs, optionally filtered by type.
pub fn count_tlds(conn: &Connection, tld_type: Option<&TldType>) -> Result<usize> {
    let count: usize = match tld_type {
        Some(t) => conn.query_row(
            "SELECT COUNT(*) FROM tlds WHERE tld_type = ?1",
            [enum_to_str(t)],
            |row| row.get(0),
        )?,
        None => conn.query_row("SELECT COUNT(*) FROM tlds", [], |row| row.get(0))?,
    };
    Ok(count)
}

fn get_tld_references(conn: &Connection, tld: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT reference FROM tld_references WHERE tld = ?1")?;
    let refs = stmt
        .query_map([tld], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(refs)
}

fn get_tld_contacts(conn: &Connection, tld: &str) -> Result<Vec<ContactRequirements>> {
    let mut stmt = conn.prepare(
        "SELECT contact_type, min, max, required, organization_allowed, organization_required, scope
         FROM tld_contacts WHERE tld = ?1 ORDER BY contact_type",
    )?;
    let contacts = stmt
        .query_map([tld], |row| {
            Ok(ContactRequirements {
                contact_type: row.get(0)?,
                min: row.get(1)?,
                max: row.get(2)?,
                required: row.get(3)?,
                organization_allowed: row.get(4)?,
                organization_required: row.get(5)?,
                scope: row.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(contacts)
}

fn hydrate_tld_rows(conn: &Connection, rows: Vec<TldRow>) -> Result<Vec<Tld>> {
    rows.into_iter()
        .map(|row| {
            let references = get_tld_references(conn, &row.tld)?;
            let contacts = get_tld_contacts(conn, &row.tld)?;
            Ok(row.into_tld(references, contacts))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Record type queries
// ---------------------------------------------------------------------------

/// Insert or replace a record type entry along with its RFCs and related types.
pub fn upsert_record_type(conn: &Connection, rt: &RecordType) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO record_types (name, type_code, summary, description, rdata_format, example, status, common)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            rt.name,
            rt.type_code,
            rt.summary,
            rt.description,
            rt.rdata_format,
            rt.example,
            enum_to_str(&rt.status),
            rt.common,
        ],
    )?;

    conn.execute(
        "DELETE FROM record_type_rfcs WHERE record_name = ?1",
        [&rt.name],
    )?;
    let mut stmt =
        conn.prepare("INSERT INTO record_type_rfcs (record_name, rfc) VALUES (?1, ?2)")?;
    for rfc in &rt.rfcs {
        stmt.execute(params![rt.name, rfc])?;
    }

    conn.execute(
        "DELETE FROM record_type_related WHERE record_name = ?1",
        [&rt.name],
    )?;
    let mut stmt =
        conn.prepare("INSERT INTO record_type_related (record_name, related) VALUES (?1, ?2)")?;
    for rel in &rt.related {
        stmt.execute(params![rt.name, rel])?;
    }

    Ok(())
}

/// Look up a record type by name (case-insensitive).
pub fn get_record_type(conn: &Connection, name: &str) -> Result<Option<RecordType>> {
    let normalized = name.to_uppercase();
    let mut stmt = conn.prepare(
        "SELECT name, type_code, summary, description, rdata_format, example, status, common
         FROM record_types WHERE name = ?1",
    )?;

    let result = stmt.query_row([&normalized], |row| {
        Ok(RecordTypeRow {
            name: row.get(0)?,
            type_code: row.get(1)?,
            summary: row.get(2)?,
            description: row.get(3)?,
            rdata_format: row.get(4)?,
            example: row.get(5)?,
            status: row.get(6)?,
            common: row.get(7)?,
        })
    });

    match result {
        Ok(row) => {
            let rfcs = get_record_type_rfcs(conn, &row.name)?;
            let related = get_record_type_related(conn, &row.name)?;
            Ok(Some(row.into_record_type(rfcs, related)))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Look up a record type by its numeric type code.
pub fn get_record_type_by_code(conn: &Connection, code: u16) -> Result<Option<RecordType>> {
    let mut stmt = conn.prepare(
        "SELECT name, type_code, summary, description, rdata_format, example, status, common
         FROM record_types WHERE type_code = ?1",
    )?;

    let result = stmt.query_row([code], |row| {
        Ok(RecordTypeRow {
            name: row.get(0)?,
            type_code: row.get(1)?,
            summary: row.get(2)?,
            description: row.get(3)?,
            rdata_format: row.get(4)?,
            example: row.get(5)?,
            status: row.get(6)?,
            common: row.get(7)?,
        })
    });

    match result {
        Ok(row) => {
            let rfcs = get_record_type_rfcs(conn, &row.name)?;
            let related = get_record_type_related(conn, &row.name)?;
            Ok(Some(row.into_record_type(rfcs, related)))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Search record types using full-text search.
pub fn search_record_types(conn: &Connection, query: &str) -> Result<Vec<RecordType>> {
    let fts_query = format!("\"{}\"*", query.replace('"', "\"\""));
    let mut stmt = conn.prepare(
        "SELECT r.name, r.type_code, r.summary, r.description, r.rdata_format, r.example, r.status, r.common
         FROM record_types r
         JOIN record_types_fts f ON r.rowid = f.rowid
         WHERE record_types_fts MATCH ?1
         ORDER BY rank",
    )?;

    let rows: Vec<RecordTypeRow> = stmt
        .query_map([&fts_query], |row| {
            Ok(RecordTypeRow {
                name: row.get(0)?,
                type_code: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                rdata_format: row.get(4)?,
                example: row.get(5)?,
                status: row.get(6)?,
                common: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|row| {
            let rfcs = get_record_type_rfcs(conn, &row.name)?;
            let related = get_record_type_related(conn, &row.name)?;
            Ok(row.into_record_type(rfcs, related))
        })
        .collect()
}

/// List all record types, optionally only common ones.
pub fn list_record_types(conn: &Connection, common_only: bool) -> Result<Vec<RecordType>> {
    let sql = if common_only {
        "SELECT name, type_code, summary, description, rdata_format, example, status, common
         FROM record_types WHERE common = 1 ORDER BY type_code"
    } else {
        "SELECT name, type_code, summary, description, rdata_format, example, status, common
         FROM record_types ORDER BY type_code"
    };

    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<RecordTypeRow> = stmt
        .query_map([], |row| {
            Ok(RecordTypeRow {
                name: row.get(0)?,
                type_code: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                rdata_format: row.get(4)?,
                example: row.get(5)?,
                status: row.get(6)?,
                common: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|row| {
            let rfcs = get_record_type_rfcs(conn, &row.name)?;
            let related = get_record_type_related(conn, &row.name)?;
            Ok(row.into_record_type(rfcs, related))
        })
        .collect()
}

/// Count record types.
pub fn count_record_types(conn: &Connection, common_only: bool) -> Result<usize> {
    let count: usize = if common_only {
        conn.query_row(
            "SELECT COUNT(*) FROM record_types WHERE common = 1",
            [],
            |row| row.get(0),
        )?
    } else {
        conn.query_row("SELECT COUNT(*) FROM record_types", [], |row| row.get(0))?
    };
    Ok(count)
}

fn get_record_type_rfcs(conn: &Connection, name: &str) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT rfc FROM record_type_rfcs WHERE record_name = ?1")?;
    let rfcs = stmt
        .query_map([name], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(rfcs)
}

fn get_record_type_related(conn: &Connection, name: &str) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT related FROM record_type_related WHERE record_name = ?1")?;
    let related = stmt
        .query_map([name], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(related)
}

// ---------------------------------------------------------------------------
// Glossary queries
// ---------------------------------------------------------------------------

/// Insert or replace a glossary term along with its related terms and references.
pub fn upsert_glossary_term(conn: &Connection, term: &GlossaryTerm) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO glossary (term, abbreviation, summary, description, category)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            term.term,
            term.abbreviation,
            term.summary,
            term.description,
            enum_to_str(&term.category),
        ],
    )?;

    conn.execute(
        "DELETE FROM glossary_related WHERE term = ?1",
        [&term.term],
    )?;
    let mut stmt =
        conn.prepare("INSERT INTO glossary_related (term, related) VALUES (?1, ?2)")?;
    for rel in &term.related {
        stmt.execute(params![term.term, rel])?;
    }

    conn.execute(
        "DELETE FROM glossary_references WHERE term = ?1",
        [&term.term],
    )?;
    let mut stmt =
        conn.prepare("INSERT INTO glossary_references (term, reference) VALUES (?1, ?2)")?;
    for r in &term.references {
        stmt.execute(params![term.term, r])?;
    }

    Ok(())
}

/// Look up a glossary term by name or abbreviation (case-insensitive).
pub fn get_glossary_term(conn: &Connection, query: &str) -> Result<Option<GlossaryTerm>> {
    let normalized = query.to_lowercase();
    let mut stmt = conn.prepare(
        "SELECT term, abbreviation, summary, description, category
         FROM glossary WHERE LOWER(term) = ?1 OR LOWER(abbreviation) = ?1",
    )?;

    let result = stmt.query_row([&normalized], |row| {
        Ok(GlossaryRow {
            term: row.get(0)?,
            abbreviation: row.get(1)?,
            summary: row.get(2)?,
            description: row.get(3)?,
            category: row.get(4)?,
        })
    });

    match result {
        Ok(row) => {
            let related = get_glossary_related(conn, &row.term)?;
            let references = get_glossary_references(conn, &row.term)?;
            Ok(Some(row.into_glossary_term(related, references)))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Search glossary terms using full-text search.
pub fn search_glossary(conn: &Connection, query: &str) -> Result<Vec<GlossaryTerm>> {
    let fts_query = format!("\"{}\"*", query.replace('"', "\"\""));
    let mut stmt = conn.prepare(
        "SELECT g.term, g.abbreviation, g.summary, g.description, g.category
         FROM glossary g
         JOIN glossary_fts f ON g.rowid = f.rowid
         WHERE glossary_fts MATCH ?1
         ORDER BY rank",
    )?;

    let rows: Vec<GlossaryRow> = stmt
        .query_map([&fts_query], |row| {
            Ok(GlossaryRow {
                term: row.get(0)?,
                abbreviation: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|row| {
            let related = get_glossary_related(conn, &row.term)?;
            let references = get_glossary_references(conn, &row.term)?;
            Ok(row.into_glossary_term(related, references))
        })
        .collect()
}

/// List all glossary terms, optionally filtered by category.
pub fn list_glossary(
    conn: &Connection,
    category: Option<&GlossaryCategory>,
) -> Result<Vec<GlossaryTerm>> {
    let cat_str;
    let (sql, query_params): (&str, Vec<&dyn rusqlite::types::ToSql>) = match category {
        Some(c) => {
            cat_str = enum_to_str(c);
            (
                "SELECT term, abbreviation, summary, description, category
                 FROM glossary WHERE category = ?1 ORDER BY term",
                vec![&cat_str as &dyn rusqlite::types::ToSql],
            )
        }
        None => (
            "SELECT term, abbreviation, summary, description, category
             FROM glossary ORDER BY term",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<GlossaryRow> = stmt
        .query_map(query_params.as_slice(), |row| {
            Ok(GlossaryRow {
                term: row.get(0)?,
                abbreviation: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|row| {
            let related = get_glossary_related(conn, &row.term)?;
            let references = get_glossary_references(conn, &row.term)?;
            Ok(row.into_glossary_term(related, references))
        })
        .collect()
}

/// Count glossary terms.
pub fn count_glossary(conn: &Connection, category: Option<&GlossaryCategory>) -> Result<usize> {
    let count: usize = match category {
        Some(c) => conn.query_row(
            "SELECT COUNT(*) FROM glossary WHERE category = ?1",
            [enum_to_str(c)],
            |row| row.get(0),
        )?,
        None => conn.query_row("SELECT COUNT(*) FROM glossary", [], |row| row.get(0))?,
    };
    Ok(count)
}

fn get_glossary_related(conn: &Connection, term: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT related FROM glossary_related WHERE term = ?1")?;
    let related = stmt
        .query_map([term], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(related)
}

fn get_glossary_references(conn: &Connection, term: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT reference FROM glossary_references WHERE term = ?1")?;
    let refs = stmt
        .query_map([term], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(refs)
}

// ---------------------------------------------------------------------------
// Helpers: intermediate row types and enum conversion
// ---------------------------------------------------------------------------

fn enum_to_str<T: std::fmt::Debug>(val: &T) -> String {
    let debug = format!("{:?}", val);
    let mut result = String::new();
    for (i, ch) in debug.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

fn parse_json_vec_string(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

fn parse_json_vec_u32(s: &str) -> Vec<u32> {
    serde_json::from_str(s).unwrap_or_default()
}

fn parse_tld_type(s: &str) -> TldType {
    match s {
        "generic" => TldType::Generic,
        "country_code" => TldType::CountryCode,
        "sponsored" => TldType::Sponsored,
        "infrastructure" => TldType::Infrastructure,
        "new_generic" => TldType::NewGeneric,
        "generic_restricted" => TldType::GenericRestricted,
        "test" => TldType::Test,
        _ => TldType::Generic,
    }
}

fn parse_dnssec(s: &str) -> DnssecSupport {
    match s {
        "signed" => DnssecSupport::Signed,
        "unsigned" => DnssecSupport::Unsigned,
        _ => DnssecSupport::Unknown,
    }
}

fn parse_record_status(s: &str) -> RecordStatus {
    match s {
        "active" => RecordStatus::Active,
        "experimental" => RecordStatus::Experimental,
        "obsolete" => RecordStatus::Obsolete,
        "reserved" => RecordStatus::Reserved,
        _ => RecordStatus::Active,
    }
}

fn parse_glossary_category(s: &str) -> GlossaryCategory {
    match s {
        "dns" => GlossaryCategory::Dns,
        "registration" => GlossaryCategory::Registration,
        "security" => GlossaryCategory::Security,
        "abuse" => GlossaryCategory::Abuse,
        "infrastructure" => GlossaryCategory::Infrastructure,
        "protocol" => GlossaryCategory::Protocol,
        "general" => GlossaryCategory::General,
        _ => GlossaryCategory::General,
    }
}

/// Maps a rusqlite row (from the TLD_SELECT column order) into a TldRow.
fn row_to_tld_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TldRow> {
    Ok(TldRow {
        tld: row.get(0)?,
        tld_type: row.get(1)?,
        name: row.get(2)?,
        registry: row.get(3)?,
        whois_server: row.get(4)?,
        rdap_url: row.get(5)?,
        dnssec: row.get(6)?,
        delegation_date: row.get(7)?,
        idn_support: row.get(8)?,
        idn_type: row.get(9)?,
        min_length: row.get(10)?,
        max_length: row.get(11)?,
        nameservers_min: row.get(12)?,
        nameservers_max: row.get(13)?,
        restrictions: row.get(14)?,
        allowed_countries: row.get(15)?,
        transfer_auth_required: row.get(16)?,
        creation_requires_prevalidation: row.get(17)?,
        validation_category: row.get(18)?,
        create_periods: row.get(19)?,
        renew_periods: row.get(20)?,
        auto_renew_periods: row.get(21)?,
        transfer_periods: row.get(22)?,
        redemption_period: row.get(23)?,
        add_grace_period: row.get(24)?,
        renew_grace_period: row.get(25)?,
        auto_renew_grace_period: row.get(26)?,
        transfer_grace_period: row.get(27)?,
        pending_delete_period: row.get(28)?,
        features: row.get(29)?,
        premium_support: row.get(30)?,
        restore_includes_renew: row.get(31)?,
        renewal_on_transfer: row.get(32)?,
        allowed_dnssec_records: row.get(33)?,
        allowed_dnssec_algorithms: row.get(34)?,
        whois_exposure: row.get(35)?,
        gdpr_category: row.get(36)?,
        jurisdiction: row.get(37)?,
    })
}

struct TldRow {
    tld: String,
    tld_type: String,
    name: String,
    registry: String,
    whois_server: Option<String>,
    rdap_url: Option<String>,
    dnssec: String,
    delegation_date: Option<String>,
    idn_support: bool,
    idn_type: Option<String>,
    min_length: Option<u32>,
    max_length: Option<u32>,
    nameservers_min: Option<u32>,
    nameservers_max: Option<u32>,
    restrictions: Option<String>,
    allowed_countries: String,
    transfer_auth_required: Option<bool>,
    creation_requires_prevalidation: Option<bool>,
    validation_category: Option<String>,
    create_periods: String,
    renew_periods: String,
    auto_renew_periods: String,
    transfer_periods: String,
    redemption_period: Option<u32>,
    add_grace_period: Option<u32>,
    renew_grace_period: Option<u32>,
    auto_renew_grace_period: Option<u32>,
    transfer_grace_period: Option<u32>,
    pending_delete_period: Option<u32>,
    features: String,
    premium_support: Option<String>,
    restore_includes_renew: Option<bool>,
    renewal_on_transfer: Option<String>,
    allowed_dnssec_records: Option<u32>,
    allowed_dnssec_algorithms: String,
    whois_exposure: Option<String>,
    gdpr_category: Option<String>,
    jurisdiction: Option<String>,
}

impl TldRow {
    fn into_tld(
        self,
        references: Vec<String>,
        contacts: Vec<ContactRequirements>,
    ) -> Tld {
        Tld {
            tld: self.tld,
            tld_type: parse_tld_type(&self.tld_type),
            name: self.name,
            registry: self.registry,
            whois_server: self.whois_server,
            rdap_url: self.rdap_url,
            dnssec: parse_dnssec(&self.dnssec),
            delegation_date: self.delegation_date,
            references,
            idn_support: self.idn_support,
            idn_type: self.idn_type,
            min_length: self.min_length,
            max_length: self.max_length,
            nameservers_min: self.nameservers_min,
            nameservers_max: self.nameservers_max,
            restrictions: self.restrictions,
            allowed_countries: parse_json_vec_string(&self.allowed_countries),
            transfer_auth_required: self.transfer_auth_required,
            creation_requires_prevalidation: self.creation_requires_prevalidation,
            validation_category: self.validation_category,
            create_periods: parse_json_vec_u32(&self.create_periods),
            renew_periods: parse_json_vec_u32(&self.renew_periods),
            auto_renew_periods: parse_json_vec_u32(&self.auto_renew_periods),
            transfer_periods: parse_json_vec_u32(&self.transfer_periods),
            redemption_period: self.redemption_period,
            add_grace_period: self.add_grace_period,
            renew_grace_period: self.renew_grace_period,
            auto_renew_grace_period: self.auto_renew_grace_period,
            transfer_grace_period: self.transfer_grace_period,
            pending_delete_period: self.pending_delete_period,
            features: parse_json_vec_string(&self.features),
            premium_support: self.premium_support,
            restore_includes_renew: self.restore_includes_renew,
            renewal_on_transfer: self.renewal_on_transfer,
            contacts,
            allowed_dnssec_records: self.allowed_dnssec_records,
            allowed_dnssec_algorithms: parse_json_vec_u32(&self.allowed_dnssec_algorithms),
            whois_exposure: self.whois_exposure,
            gdpr_category: self.gdpr_category,
            jurisdiction: self.jurisdiction,
        }
    }
}

struct RecordTypeRow {
    name: String,
    type_code: u16,
    summary: String,
    description: String,
    rdata_format: String,
    example: String,
    status: String,
    common: bool,
}

impl RecordTypeRow {
    fn into_record_type(self, rfcs: Vec<String>, related: Vec<String>) -> RecordType {
        RecordType {
            name: self.name,
            type_code: self.type_code,
            summary: self.summary,
            description: self.description,
            rdata_format: self.rdata_format,
            example: self.example,
            rfcs,
            status: parse_record_status(&self.status),
            common: self.common,
            related,
        }
    }
}

struct GlossaryRow {
    term: String,
    abbreviation: Option<String>,
    summary: String,
    description: String,
    category: String,
}

impl GlossaryRow {
    fn into_glossary_term(self, related: Vec<String>, references: Vec<String>) -> GlossaryTerm {
        GlossaryTerm {
            term: self.term,
            abbreviation: self.abbreviation,
            summary: self.summary,
            description: self.description,
            category: parse_glossary_category(&self.category),
            related,
            references,
        }
    }
}
