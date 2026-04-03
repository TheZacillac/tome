-- ============================================================================
-- TLD Database Schema (SQLite)
-- Comprehensive reference database for TLD portfolio management
-- Adapted from PostgreSQL schema for use with rusqlite
-- ============================================================================

-- ----------------------------------------
-- CORE TLD TABLE
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tlds (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld                 TEXT NOT NULL UNIQUE,             -- e.g. 'com', 'co.uk', 'みんな'
    tld_ascii           TEXT,                             -- ACE/punycode form if IDN TLD
    dot_tld             TEXT GENERATED ALWAYS AS ('.' || tld) STORED,
    tld_type            TEXT NOT NULL CHECK (tld_type IN ('gTLD', 'ccTLD', 'nTLD')),
    delegation_status   TEXT NOT NULL DEFAULT 'active'
                        CHECK (delegation_status IN ('active', 'retired', 'revoked', 'not_assigned', 'pending')),
    iana_id             TEXT,                             -- IANA root zone DB identifier
    iana_delegation_date TEXT,                            -- DATE as ISO 8601
    iana_last_updated   TEXT,                             -- DATE as ISO 8601
    description         TEXT,                             -- purpose / intended use
    created_at          TEXT DEFAULT (datetime('now')),
    updated_at          TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_tlds_type ON tlds(tld_type);
CREATE INDEX IF NOT EXISTS idx_tlds_status ON tlds(delegation_status);

-- ----------------------------------------
-- COUNTRY CODE MAPPING (ccTLDs)
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_country_mapping (
    tld_id              INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    iso_3166_alpha2     TEXT NOT NULL,                    -- 2-letter country code
    iso_3166_alpha3     TEXT,                             -- 3-letter country code
    iso_3166_numeric    INTEGER,
    country_name        TEXT NOT NULL,
    sovereign_state     TEXT,                             -- for territories (e.g. .gg -> Guernsey)
    local_presence_required INTEGER DEFAULT 0,            -- boolean
    local_presence_notes    TEXT
);

-- ----------------------------------------
-- REGISTRY OPERATORS
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS registry_operators (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT NOT NULL,
    short_name          TEXT,
    website             TEXT,
    abuse_contact_email TEXT,
    abuse_contact_phone TEXT,
    hq_country          TEXT,                             -- ISO 3166 alpha-2
    parent_company      TEXT,
    icann_accredited    INTEGER DEFAULT 0,                -- boolean
    notes               TEXT,
    created_at          TEXT DEFAULT (datetime('now'))
);

-- ----------------------------------------
-- TLD <-> REGISTRY OPERATOR RELATIONSHIP
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_registry (
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    registry_operator_id INTEGER NOT NULL REFERENCES registry_operators(id),
    role                TEXT NOT NULL DEFAULT 'operator'
                        CHECK (role IN ('operator', 'backend_provider', 'sponsor')),
    effective_date      TEXT,                             -- DATE
    end_date            TEXT,                             -- NULL = current
    registry_agreement_url TEXT,
    PRIMARY KEY (tld_id, registry_operator_id, role)
);

-- ----------------------------------------
-- REGISTRATION POLICY
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_registration_policy (
    tld_id                  INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    registration_model      TEXT NOT NULL DEFAULT 'open'
                            CHECK (registration_model IN ('open', 'restricted', 'closed', 'grandfathered')),
    eligibility_requirements TEXT,
    requires_local_presence  INTEGER DEFAULT 0,           -- boolean
    requires_trademark       INTEGER DEFAULT 0,           -- boolean
    requires_sector_nexus    INTEGER DEFAULT 0,           -- boolean
    sector_restrictions      TEXT,
    min_registration_years   INTEGER DEFAULT 1,
    max_registration_years   INTEGER DEFAULT 10,
    supports_privacy_proxy   INTEGER DEFAULT 1,           -- boolean
    sunrise_start            TEXT,                         -- DATE
    sunrise_end              TEXT,
    landrush_start           TEXT,
    landrush_end             TEXT,
    general_availability     TEXT,
    notes                    TEXT
);

-- ----------------------------------------
-- TECHNICAL SPECIFICATIONS
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_technical (
    tld_id                  INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    min_label_length        INTEGER DEFAULT 1,
    max_label_length        INTEGER DEFAULT 63,
    allowed_characters      TEXT DEFAULT 'a-z, 0-9, hyphen',
    allows_idn              INTEGER DEFAULT 0,            -- boolean
    idn_scripts_supported   TEXT,                          -- JSON array e.g. '["Latin","Cyrillic","Han"]'
    idn_language_tags       TEXT,                          -- JSON array e.g. '["zh-Hans","ja","ko"]'
    supports_dnssec         INTEGER DEFAULT 1,             -- boolean
    dnssec_algorithms       TEXT,                          -- JSON array e.g. '["RSA/SHA-256","ECDSA P-256"]'
    min_nameservers         INTEGER DEFAULT 2,
    max_nameservers         INTEGER DEFAULT 13,
    supports_ds_records     INTEGER DEFAULT 1,             -- boolean
    supports_glue_records   INTEGER DEFAULT 1,             -- boolean
    epp_repository_id       TEXT,                          -- e.g. 'COM' for .com EPP
    supports_epp_transfer   INTEGER DEFAULT 1,             -- boolean
    supports_authcode       INTEGER DEFAULT 1,             -- boolean
    supports_rgp            INTEGER DEFAULT 1,             -- boolean (Redemption Grace Period)
    supports_bulk_operations INTEGER,                      -- boolean
    registry_lock_available INTEGER DEFAULT 0,             -- boolean
    registrar_lock_available INTEGER DEFAULT 1,            -- boolean
    notes                   TEXT
);

-- ----------------------------------------
-- WHOIS / RDAP
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_whois_rdap (
    tld_id              INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    whois_server        TEXT,                              -- e.g. 'whois.verisign-grs.com'
    whois_port          INTEGER DEFAULT 43,
    whois_model         TEXT CHECK (whois_model IS NULL OR whois_model IN ('thick', 'thin')),
    rdap_base_url       TEXT,                              -- e.g. 'https://rdap.verisign.com/com/v1/'
    rdap_conformance    TEXT,                              -- JSON array
    gdpr_redacted       INTEGER DEFAULT 1,                 -- boolean
    gdpr_redaction_notes TEXT,
    rate_limit_info     TEXT,
    notes               TEXT
);

-- ----------------------------------------
-- LIFECYCLE & PRICING
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_lifecycle (
    tld_id                      INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    add_grace_period_days       INTEGER DEFAULT 5,
    renew_grace_period_days     INTEGER,
    auto_renew_grace_period_days INTEGER,
    transfer_lock_days          INTEGER DEFAULT 60,
    redemption_grace_period_days INTEGER DEFAULT 30,
    pending_delete_days         INTEGER DEFAULT 5,
    expiration_policy           TEXT,                       -- what happens on expiry
    transfer_cooldown_days      INTEGER,
    notes                       TEXT
);

CREATE TABLE IF NOT EXISTS tld_pricing (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    price_type          TEXT NOT NULL                       -- 'registration', 'renewal', 'transfer', 'restore', 'premium'
                        CHECK (price_type IN ('registration', 'renewal', 'transfer', 'restore', 'premium')),
    currency            TEXT DEFAULT 'USD',
    wholesale_price     REAL,                               -- registry price
    retail_price_low    REAL,                               -- typical retail range low
    retail_price_high   REAL,                               -- typical retail range high
    premium_tier        TEXT,                               -- e.g. 'Platinum', 'Gold'
    effective_date      TEXT,                               -- DATE
    end_date            TEXT,
    source              TEXT,
    notes               TEXT,
    UNIQUE(tld_id, price_type, premium_tier, currency, effective_date)
);

CREATE INDEX IF NOT EXISTS idx_pricing_tld ON tld_pricing(tld_id);

-- ----------------------------------------
-- DISPUTE RESOLUTION & POLICY
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_dispute_policies (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    policy_type         TEXT NOT NULL
                        CHECK (policy_type IN ('UDRP', 'URS', 'PDDRP', 'custom')),
    policy_name         TEXT,
    policy_url          TEXT,
    provider            TEXT,                               -- e.g. 'WIPO', 'NAF', 'Nominet'
    mandatory           INTEGER DEFAULT 1,                  -- boolean
    notes               TEXT
);

CREATE INDEX IF NOT EXISTS idx_disputes_tld ON tld_dispute_policies(tld_id);

-- ----------------------------------------
-- BRAND PROTECTION
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_brand_protection (
    tld_id                      INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    in_tmch                     INTEGER DEFAULT 0,          -- boolean (Trademark Clearinghouse)
    claims_period_active        INTEGER DEFAULT 0,          -- boolean
    claims_period_end           TEXT,                        -- DATE
    supports_dpml               INTEGER DEFAULT 0,          -- boolean (Domains Protected Marks List)
    dpml_provider               TEXT,
    supports_adndrc             INTEGER DEFAULT 0,          -- boolean (Asian Domain Name DRC)
    phishing_abuse_risk         TEXT DEFAULT 'medium'
                                CHECK (phishing_abuse_risk IN ('low', 'medium', 'high', 'critical')),
    typosquatting_risk          TEXT DEFAULT 'medium'
                                CHECK (typosquatting_risk IN ('low', 'medium', 'high', 'critical')),
    commonly_blocked_corporate  INTEGER DEFAULT 0,          -- boolean
    defensive_registration_recommended INTEGER DEFAULT 0,   -- boolean
    notes                       TEXT
);

-- ----------------------------------------
-- NAMESERVERS (authoritative for the TLD zone)
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_nameservers (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    hostname            TEXT NOT NULL,
    ipv4_address        TEXT,                               -- IPv4 as text
    ipv6_address        TEXT,                               -- IPv6 as text
    anycast             INTEGER DEFAULT 0,                  -- boolean
    operator            TEXT,
    notes               TEXT,
    UNIQUE(tld_id, hostname)
);

CREATE INDEX IF NOT EXISTS idx_ns_tld ON tld_nameservers(tld_id);

-- ----------------------------------------
-- MARKET / USAGE DATA (time series)
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_market_data (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    snapshot_date       TEXT NOT NULL,                       -- DATE
    total_domains       INTEGER,                            -- domains under management
    net_adds_month      INTEGER,                            -- net new registrations that month
    renewal_rate_pct    REAL,                               -- percentage
    registrar_count     INTEGER,
    tranco_top1m_count  INTEGER,                            -- domains in Tranco top 1M
    parking_rate_pct    REAL,                               -- estimated % parked/unused
    source              TEXT,
    notes               TEXT,
    UNIQUE(tld_id, snapshot_date)
);

CREATE INDEX IF NOT EXISTS idx_market_tld_date ON tld_market_data(tld_id, snapshot_date);

-- ----------------------------------------
-- SUPPORTED EPP STATUS CODES
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_epp_statuses (
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    epp_status          TEXT NOT NULL,                       -- e.g. 'clientTransferProhibited'
    supported           INTEGER DEFAULT 1,                  -- boolean
    notes               TEXT,
    PRIMARY KEY (tld_id, epp_status)
);

-- ----------------------------------------
-- REGISTRAR ACCREDITATION
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_registrars (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    registrar_name      TEXT NOT NULL,
    iana_registrar_id   INTEGER,
    accreditation_date  TEXT,                                -- DATE
    status              TEXT DEFAULT 'active',
    notes               TEXT
);

CREATE INDEX IF NOT EXISTS idx_registrars_tld ON tld_registrars(tld_id);

-- ----------------------------------------
-- IDN TABLES (detailed IDN support per TLD)
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_idn_tables (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    language            TEXT NOT NULL,
    script              TEXT NOT NULL,
    idn_table_url       TEXT,
    version             TEXT,
    effective_date      TEXT,                                -- DATE
    notes               TEXT
);

CREATE INDEX IF NOT EXISTS idx_idn_tld ON tld_idn_tables(tld_id);

-- ----------------------------------------
-- TRANSFER RULES
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_transfer_rules (
    tld_id                      INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    inter_registrar_transfer    INTEGER DEFAULT 1,           -- boolean
    requires_authcode           INTEGER DEFAULT 1,           -- boolean
    authcode_format             TEXT,
    authcode_validity_days      INTEGER,
    transfer_approval_method    TEXT,                         -- 'email_foa', 'auto', 'manual', 'registrant_portal'
    approval_timeout_days       INTEGER DEFAULT 5,
    new_registration_lock_days  INTEGER DEFAULT 60,
    post_transfer_lock_days     INTEGER DEFAULT 60,
    transfer_adds_year          INTEGER DEFAULT 1,           -- boolean
    bulk_transfer_supported     INTEGER DEFAULT 0,           -- boolean
    registry_transfer_supported INTEGER DEFAULT 0,           -- boolean
    custom_transfer_process     INTEGER DEFAULT 0,           -- boolean
    custom_process_notes        TEXT,
    notes                       TEXT
);

-- ----------------------------------------
-- CERTIFICATE / SECURITY DETAILS
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_security (
    tld_id                  INTEGER PRIMARY KEY REFERENCES tlds(id) ON DELETE CASCADE,
    zone_signed_dnssec      INTEGER DEFAULT 1,               -- boolean
    ds_digest_types         TEXT,                             -- JSON array e.g. '["SHA-256","SHA-384"]'
    key_signing_algorithm   TEXT,
    zone_signing_algorithm  TEXT,
    hsts_preloaded          INTEGER DEFAULT 0,               -- boolean
    certificate_transparency_enforced INTEGER,                -- boolean
    notes                   TEXT
);

-- ----------------------------------------
-- AUDIT / CHANGE TRACKING
-- ----------------------------------------
CREATE TABLE IF NOT EXISTS tld_change_log (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tld_id              INTEGER NOT NULL REFERENCES tlds(id) ON DELETE CASCADE,
    changed_table       TEXT NOT NULL,
    changed_field       TEXT,
    old_value           TEXT,
    new_value           TEXT,
    changed_by          TEXT,
    changed_at          TEXT DEFAULT (datetime('now')),
    source              TEXT,
    notes               TEXT
);

CREATE INDEX IF NOT EXISTS idx_changelog_tld ON tld_change_log(tld_id);
CREATE INDEX IF NOT EXISTS idx_changelog_date ON tld_change_log(changed_at);

-- ----------------------------------------
-- VIEWS
-- ----------------------------------------
CREATE VIEW IF NOT EXISTS v_tld_overview AS
SELECT
    t.tld,
    t.tld_type,
    t.delegation_status,
    ro.name AS registry_operator,
    cm.country_name,
    cm.iso_3166_alpha2,
    rp.registration_model,
    rp.requires_local_presence,
    tech.supports_dnssec,
    tech.allows_idn,
    tech.registry_lock_available,
    wr.whois_server,
    wr.rdap_base_url,
    bp.phishing_abuse_risk,
    bp.defensive_registration_recommended,
    xfr.transfer_adds_year
FROM tlds t
LEFT JOIN tld_registry treg ON t.id = treg.tld_id AND treg.role = 'operator' AND treg.end_date IS NULL
LEFT JOIN registry_operators ro ON treg.registry_operator_id = ro.id
LEFT JOIN tld_country_mapping cm ON t.id = cm.tld_id
LEFT JOIN tld_registration_policy rp ON t.id = rp.tld_id
LEFT JOIN tld_technical tech ON t.id = tech.tld_id
LEFT JOIN tld_whois_rdap wr ON t.id = wr.tld_id
LEFT JOIN tld_brand_protection bp ON t.id = bp.tld_id
LEFT JOIN tld_transfer_rules xfr ON t.id = xfr.tld_id;

CREATE VIEW IF NOT EXISTS v_tld_lifecycle_pricing AS
SELECT
    t.tld,
    lc.add_grace_period_days,
    lc.redemption_grace_period_days,
    lc.pending_delete_days,
    lc.transfer_lock_days,
    p_reg.wholesale_price AS registration_wholesale,
    p_reg.retail_price_low AS registration_retail_low,
    p_ren.wholesale_price AS renewal_wholesale,
    p_res.wholesale_price AS restore_wholesale,
    tr.requires_authcode,
    tr.transfer_adds_year,
    tr.custom_transfer_process
FROM tlds t
LEFT JOIN tld_lifecycle lc ON t.id = lc.tld_id
LEFT JOIN tld_pricing p_reg ON t.id = p_reg.tld_id AND p_reg.price_type = 'registration' AND p_reg.premium_tier IS NULL AND p_reg.end_date IS NULL
LEFT JOIN tld_pricing p_ren ON t.id = p_ren.tld_id AND p_ren.price_type = 'renewal' AND p_ren.premium_tier IS NULL AND p_ren.end_date IS NULL
LEFT JOIN tld_pricing p_res ON t.id = p_res.tld_id AND p_res.price_type = 'restore' AND p_res.end_date IS NULL
LEFT JOIN tld_transfer_rules tr ON t.id = tr.tld_id;
