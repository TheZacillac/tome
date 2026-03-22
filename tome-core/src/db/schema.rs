use rusqlite::Connection;

use crate::error::Result;

/// Current schema version. Increment when adding migrations.
pub const SCHEMA_VERSION: u32 = 2;

/// Initialize the database schema, creating all tables if they don't exist.
pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );

        -- Top-level domains
        CREATE TABLE IF NOT EXISTS tlds (
            tld         TEXT PRIMARY KEY NOT NULL,
            tld_type    TEXT NOT NULL,
            name        TEXT NOT NULL,
            registry    TEXT NOT NULL,

            -- IANA reference data
            whois_server    TEXT,
            rdap_url        TEXT,
            dnssec          TEXT NOT NULL DEFAULT 'unknown',
            delegation_date TEXT,

            -- Domain syntax
            idn_support     INTEGER NOT NULL DEFAULT 0,
            idn_type        TEXT,
            min_length      INTEGER,
            max_length      INTEGER,

            -- Nameserver requirements
            nameservers_min INTEGER,
            nameservers_max INTEGER,

            -- Registration constraints
            restrictions    TEXT,
            allowed_countries   TEXT NOT NULL DEFAULT '[]',  -- JSON array
            transfer_auth_required          INTEGER,
            creation_requires_prevalidation INTEGER,
            validation_category             TEXT,

            -- Periods (stored as JSON arrays for multi-value, integers for single)
            create_periods      TEXT NOT NULL DEFAULT '[]',  -- JSON array of years
            renew_periods       TEXT NOT NULL DEFAULT '[]',
            auto_renew_periods  TEXT NOT NULL DEFAULT '[]',
            transfer_periods    TEXT NOT NULL DEFAULT '[]',
            redemption_period       INTEGER,  -- days
            add_grace_period        INTEGER,
            renew_grace_period      INTEGER,
            auto_renew_grace_period INTEGER,
            transfer_grace_period   INTEGER,
            pending_delete_period   INTEGER,

            -- Features
            features            TEXT NOT NULL DEFAULT '[]',  -- JSON array of strings
            premium_support     TEXT,
            restore_includes_renew  INTEGER,
            renewal_on_transfer     TEXT,

            -- DNSSEC
            allowed_dnssec_records      INTEGER,
            allowed_dnssec_algorithms   TEXT NOT NULL DEFAULT '[]',  -- JSON array of ints

            -- Privacy
            whois_exposure  TEXT,
            gdpr_category   TEXT,
            jurisdiction    TEXT
        );

        -- TLD references (RFCs, IANA links, etc.)
        CREATE TABLE IF NOT EXISTS tld_references (
            tld         TEXT NOT NULL REFERENCES tlds(tld) ON DELETE CASCADE,
            reference   TEXT NOT NULL,
            PRIMARY KEY (tld, reference)
        );

        -- TLD contact requirements
        CREATE TABLE IF NOT EXISTS tld_contacts (
            tld             TEXT NOT NULL REFERENCES tlds(tld) ON DELETE CASCADE,
            contact_type    TEXT NOT NULL,  -- registrant, admin, billing, tech
            min             INTEGER,
            max             INTEGER,
            required        INTEGER,
            organization_allowed    INTEGER,
            organization_required   INTEGER,
            scope           TEXT,
            PRIMARY KEY (tld, contact_type)
        );

        -- DNS record types
        CREATE TABLE IF NOT EXISTS record_types (
            name        TEXT PRIMARY KEY NOT NULL,
            type_code   INTEGER NOT NULL UNIQUE,
            summary     TEXT NOT NULL,
            description TEXT NOT NULL,
            rdata_format TEXT NOT NULL,
            example     TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'active',
            common      INTEGER NOT NULL DEFAULT 0
        );

        -- Record type RFCs
        CREATE TABLE IF NOT EXISTS record_type_rfcs (
            record_name TEXT NOT NULL REFERENCES record_types(name) ON DELETE CASCADE,
            rfc         TEXT NOT NULL,
            PRIMARY KEY (record_name, rfc)
        );

        -- Record type relationships (e.g., A <-> AAAA)
        CREATE TABLE IF NOT EXISTS record_type_related (
            record_name TEXT NOT NULL REFERENCES record_types(name) ON DELETE CASCADE,
            related     TEXT NOT NULL,
            PRIMARY KEY (record_name, related)
        );

        -- Glossary terms
        CREATE TABLE IF NOT EXISTS glossary (
            term        TEXT PRIMARY KEY NOT NULL,
            abbreviation TEXT,
            summary     TEXT NOT NULL,
            description TEXT NOT NULL,
            category    TEXT NOT NULL DEFAULT 'general'
        );

        -- Glossary related terms
        CREATE TABLE IF NOT EXISTS glossary_related (
            term        TEXT NOT NULL REFERENCES glossary(term) ON DELETE CASCADE,
            related     TEXT NOT NULL,
            PRIMARY KEY (term, related)
        );

        -- Glossary references
        CREATE TABLE IF NOT EXISTS glossary_references (
            term        TEXT NOT NULL REFERENCES glossary(term) ON DELETE CASCADE,
            reference   TEXT NOT NULL,
            PRIMARY KEY (term, reference)
        );

        -- Full-text search indexes
        CREATE VIRTUAL TABLE IF NOT EXISTS tlds_fts USING fts5(
            tld, name, registry, restrictions,
            content='tlds',
            content_rowid='rowid'
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS record_types_fts USING fts5(
            name, summary, description,
            content='record_types',
            content_rowid='rowid'
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS glossary_fts USING fts5(
            term, abbreviation, summary, description,
            content='glossary',
            content_rowid='rowid'
        );

        -- Triggers to keep FTS indexes in sync
        CREATE TRIGGER IF NOT EXISTS tlds_ai AFTER INSERT ON tlds BEGIN
            INSERT INTO tlds_fts(rowid, tld, name, registry, restrictions)
            VALUES (new.rowid, new.tld, new.name, new.registry, new.restrictions);
        END;

        CREATE TRIGGER IF NOT EXISTS tlds_ad AFTER DELETE ON tlds BEGIN
            INSERT INTO tlds_fts(tlds_fts, rowid, tld, name, registry, restrictions)
            VALUES ('delete', old.rowid, old.tld, old.name, old.registry, old.restrictions);
        END;

        CREATE TRIGGER IF NOT EXISTS tlds_au AFTER UPDATE ON tlds BEGIN
            INSERT INTO tlds_fts(tlds_fts, rowid, tld, name, registry, restrictions)
            VALUES ('delete', old.rowid, old.tld, old.name, old.registry, old.restrictions);
            INSERT INTO tlds_fts(rowid, tld, name, registry, restrictions)
            VALUES (new.rowid, new.tld, new.name, new.registry, new.restrictions);
        END;

        CREATE TRIGGER IF NOT EXISTS record_types_ai AFTER INSERT ON record_types BEGIN
            INSERT INTO record_types_fts(rowid, name, summary, description)
            VALUES (new.rowid, new.name, new.summary, new.description);
        END;

        CREATE TRIGGER IF NOT EXISTS record_types_ad AFTER DELETE ON record_types BEGIN
            INSERT INTO record_types_fts(record_types_fts, rowid, name, summary, description)
            VALUES ('delete', old.rowid, old.name, old.summary, old.description);
        END;

        CREATE TRIGGER IF NOT EXISTS record_types_au AFTER UPDATE ON record_types BEGIN
            INSERT INTO record_types_fts(record_types_fts, rowid, name, summary, description)
            VALUES ('delete', old.rowid, old.name, old.summary, old.description);
            INSERT INTO record_types_fts(rowid, name, summary, description)
            VALUES (new.rowid, new.name, new.summary, new.description);
        END;

        CREATE TRIGGER IF NOT EXISTS glossary_ai AFTER INSERT ON glossary BEGIN
            INSERT INTO glossary_fts(rowid, term, abbreviation, summary, description)
            VALUES (new.rowid, new.term, new.abbreviation, new.summary, new.description);
        END;

        CREATE TRIGGER IF NOT EXISTS glossary_ad AFTER DELETE ON glossary BEGIN
            INSERT INTO glossary_fts(glossary_fts, rowid, term, abbreviation, summary, description)
            VALUES ('delete', old.rowid, old.term, old.abbreviation, old.summary, old.description);
        END;

        CREATE TRIGGER IF NOT EXISTS glossary_au AFTER UPDATE ON glossary BEGIN
            INSERT INTO glossary_fts(glossary_fts, rowid, term, abbreviation, summary, description)
            VALUES ('delete', old.rowid, old.term, old.abbreviation, old.summary, old.description);
            INSERT INTO glossary_fts(rowid, term, abbreviation, summary, description)
            VALUES (new.rowid, new.term, new.abbreviation, new.summary, new.description);
        END;
        ",
    )?;

    // Set schema version if not already set
    let count: u32 = conn.query_row("SELECT COUNT(*) FROM schema_version", [], |row| {
        row.get(0)
    })?;
    if count == 0 {
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [SCHEMA_VERSION],
        )?;
    }

    Ok(())
}
