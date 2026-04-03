//! Seed data for the TLD database.
//!
//! Every value in this module is sourced from authoritative, publicly verifiable
//! registries. **Do not add data here unless it can be independently confirmed.**
//!
//! ## Sources
//!
//! | Data                    | Authority                                          |
//! |-------------------------|-----------------------------------------------------|
//! | TLD strings, operators  | IANA Root Zone Database <https://www.iana.org/domains/root/db> |
//! | WHOIS servers           | IANA Root Zone Database (per-TLD pages)              |
//! | RDAP URLs               | IANA RDAP Bootstrap <https://data.iana.org/rdap/dns.json> |
//! | Delegation dates        | IANA Root Zone Database (per-TLD pages)              |
//! | ISO 3166 country codes  | ISO 3166-1 standard                                  |
//! | Effective SLDs          | Registry operator published policies                 |

use crate::db::{
    CountryMappingRow, RegistryOperatorRow, TldRow, TomeDb, TransferRulesRow, WhoisRdapRow,
};
use crate::error::Result;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Populate the database with verified seed data.
///
/// Idempotent: skips seeding if the database already contains TLD data.
/// All inserts run within a single transaction — failure rolls back cleanly.
pub fn seed(db: &TomeDb) -> Result<()> {
    // Skip if already seeded
    if db.count_tlds()? > 0 {
        return Ok(());
    }
    db.connection().execute_batch("BEGIN")?;
    match (|| {
        let ops = seed_registry_operators(db)?;
        seed_tlds(db, &ops)?;
        Ok::<(), crate::error::TomeError>(())
    })() {
        Ok(()) => {
            db.connection().execute_batch("COMMIT")?;
            Ok(())
        }
        Err(e) => {
            let _ = db.connection().execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// Registry operator indices (returned by seed_registry_operators)
// ---------------------------------------------------------------------------

/// Holds the auto-assigned row IDs of every seeded registry operator so that
/// TLD rows can reference them by name without a second query.
struct Operators {
    verisign: i64,
    pir: i64,
    educause: i64,
    cisa: i64,
    dod_nic: i64,
    iana: i64,
    iab: i64,
    identity_digital: i64,
    registry_services_llc: i64,
    nominet: i64,
    denic: i64,
    afnic: i64,
    jprs: i64,
    auda: i64,
    cira: i64,
    cnnic: i64,
    cgi_br: i64,
    nixi: i64,
    iit_cnr: i64,
    sidn: i64,
    cc_tld_ru: i64,
    iis: i64,
    switch_ch: i64,
    red_es: i64,
    icb_io: i64,
    internetnz: i64,
    zadna: i64,
    nic_mexico: i64,
    kisa: i64,
    nic_at: i64,
    dns_belgium: i64,
    punktum_dk: i64,
    traficom: i64,
    iedr: i64,
    norid: i64,
    nask: i64,
    dns_pt: i64,
    sgnic: i64,
    google_registry: i64,
    xyz_com: i64,
    radix: i64,
    sita: i64,
    dot_cooperation: i64,
    museum_dma: i64,
}

// ---------------------------------------------------------------------------
// Registry operators
// ---------------------------------------------------------------------------

fn seed_registry_operators(db: &TomeDb) -> Result<Operators> {
    let r =
        |name: &str, short: Option<&str>, hq: Option<&str>, icann: bool| -> RegistryOperatorRow {
            RegistryOperatorRow {
                id: None,
                name: name.to_string(),
                short_name: short.map(str::to_string),
                website: None,
                abuse_contact_email: None,
                abuse_contact_phone: None,
                hq_country: hq.map(str::to_string),
                parent_company: None,
                icann_accredited: icann,
                notes: None,
            }
        };

    Ok(Operators {
        verisign: db.insert_registry_operator(&r(
            "VeriSign Global Registry Services",
            Some("VeriSign"),
            Some("US"),
            true,
        ))?,
        pir: db.insert_registry_operator(&r(
            "Public Interest Registry",
            Some("PIR"),
            Some("US"),
            true,
        ))?,
        educause: db.insert_registry_operator(&r("EDUCAUSE", None, Some("US"), false))?,
        cisa: db.insert_registry_operator(&r(
            "Cybersecurity and Infrastructure Security Agency",
            Some("CISA"),
            Some("US"),
            false,
        ))?,
        dod_nic: db.insert_registry_operator(&r(
            "DoD Network Information Center",
            Some("DoD NIC"),
            Some("US"),
            false,
        ))?,
        iana: db.insert_registry_operator(&r(
            "Internet Assigned Numbers Authority",
            Some("IANA"),
            Some("US"),
            false,
        ))?,
        iab: db.insert_registry_operator(&r(
            "Internet Architecture Board",
            Some("IAB"),
            Some("US"),
            false,
        ))?,
        identity_digital: db.insert_registry_operator(&r(
            "Identity Digital Limited",
            Some("Identity Digital"),
            Some("US"),
            true,
        ))?,
        registry_services_llc: db.insert_registry_operator(&r(
            "Registry Services, LLC",
            None,
            Some("US"),
            true,
        ))?,
        nominet: db.insert_registry_operator(&r(
            "Nominet UK",
            Some("Nominet"),
            Some("GB"),
            false,
        ))?,
        denic: db.insert_registry_operator(&r("DENIC eG", Some("DENIC"), Some("DE"), false))?,
        afnic: db.insert_registry_operator(&r(
            "Association Française pour le Nommage Internet en Coopération",
            Some("AFNIC"),
            Some("FR"),
            false,
        ))?,
        jprs: db.insert_registry_operator(&r(
            "Japan Registry Services Co., Ltd.",
            Some("JPRS"),
            Some("JP"),
            false,
        ))?,
        auda: db.insert_registry_operator(&r(
            ".au Domain Administration",
            Some("auDA"),
            Some("AU"),
            false,
        ))?,
        cira: db.insert_registry_operator(&r(
            "Canadian Internet Registration Authority",
            Some("CIRA"),
            Some("CA"),
            false,
        ))?,
        cnnic: db.insert_registry_operator(&r(
            "China Internet Network Information Center",
            Some("CNNIC"),
            Some("CN"),
            false,
        ))?,
        cgi_br: db.insert_registry_operator(&r(
            "Comitê Gestor da Internet no Brasil",
            Some("CGI.br"),
            Some("BR"),
            false,
        ))?,
        nixi: db.insert_registry_operator(&r(
            "National Internet Exchange of India",
            Some("NIXI"),
            Some("IN"),
            false,
        ))?,
        iit_cnr: db.insert_registry_operator(&r("IIT - CNR", None, Some("IT"), false))?,
        sidn: db.insert_registry_operator(&r(
            "Stichting Internet Domeinregistratie Nederland",
            Some("SIDN"),
            Some("NL"),
            false,
        ))?,
        cc_tld_ru: db.insert_registry_operator(&r(
            "Coordination Center for TLD RU",
            None,
            Some("RU"),
            false,
        ))?,
        iis: db.insert_registry_operator(&r(
            "The Internet Infrastructure Foundation",
            Some("Internetstiftelsen"),
            Some("SE"),
            false,
        ))?,
        switch_ch: db.insert_registry_operator(&r(
            "SWITCH The Swiss Education & Research Network",
            Some("SWITCH"),
            Some("CH"),
            false,
        ))?,
        red_es: db.insert_registry_operator(&r("Red.es", None, Some("ES"), false))?,
        icb_io: db.insert_registry_operator(&r(
            "Internet Computer Bureau Limited",
            Some("ICB"),
            Some("GB"),
            false,
        ))?,
        internetnz: db.insert_registry_operator(&r("InternetNZ", None, Some("NZ"), false))?,
        zadna: db.insert_registry_operator(&r(
            "ZA Domain Name Authority",
            Some("ZADNA"),
            Some("ZA"),
            false,
        ))?,
        nic_mexico: db.insert_registry_operator(&r(
            "NIC-Mexico, ITESM - Campus Monterrey",
            Some("NIC Mexico"),
            Some("MX"),
            false,
        ))?,
        kisa: db.insert_registry_operator(&r(
            "Korea Internet & Security Agency",
            Some("KISA"),
            Some("KR"),
            false,
        ))?,
        nic_at: db.insert_registry_operator(&r(
            "nic.at GmbH",
            Some("nic.at"),
            Some("AT"),
            false,
        ))?,
        dns_belgium: db.insert_registry_operator(&r(
            "DNS Belgium vzw/asbl",
            Some("DNS Belgium"),
            Some("BE"),
            false,
        ))?,
        punktum_dk: db.insert_registry_operator(&r(
            "Punktum dk A/S",
            Some("Punktum dk"),
            Some("DK"),
            false,
        ))?,
        traficom: db.insert_registry_operator(&r(
            "Finnish Transport and Communications Agency",
            Some("Traficom"),
            Some("FI"),
            false,
        ))?,
        iedr: db.insert_registry_operator(&r(
            "IE Domain Registry Limited",
            Some("IEDR"),
            Some("IE"),
            false,
        ))?,
        norid: db.insert_registry_operator(&r("Norid A/S", Some("Norid"), Some("NO"), false))?,
        nask: db.insert_registry_operator(&r(
            "Research and Academic Computer Network",
            Some("NASK"),
            Some("PL"),
            false,
        ))?,
        dns_pt: db.insert_registry_operator(&r(
            "Associação DNS.PT",
            Some("DNS.PT"),
            Some("PT"),
            false,
        ))?,
        sgnic: db.insert_registry_operator(&r(
            "Singapore Network Information Centre Pte Ltd",
            Some("SGNIC"),
            Some("SG"),
            false,
        ))?,
        google_registry: db.insert_registry_operator(&r(
            "Charleston Road Registry Inc.",
            Some("Google Registry"),
            Some("US"),
            true,
        ))?,
        xyz_com: db.insert_registry_operator(&r("XYZ.COM LLC", Some("XYZ"), Some("US"), true))?,
        radix: db.insert_registry_operator(&r(
            "Radix Technologies Inc.",
            Some("Radix"),
            Some("AE"),
            true,
        ))?,
        sita: db.insert_registry_operator(&r(
            "Société Internationale de Télécommunications Aéronautiques",
            Some("SITA"),
            Some("BE"),
            false,
        ))?,
        dot_cooperation: db.insert_registry_operator(&r(
            "DotCooperation LLC",
            None,
            Some("US"),
            false,
        ))?,
        museum_dma: db.insert_registry_operator(&r(
            "Museum Domain Management Association",
            None,
            Some("SE"),
            false,
        ))?,
    })
}

// ---------------------------------------------------------------------------
// TLD seed data
// ---------------------------------------------------------------------------

/// A single TLD's worth of seed data, collected for batch insertion.
struct SeedEntry {
    tld: &'static str,
    tld_type: &'static str,
    description: &'static str,
    delegation_date: Option<&'static str>,
    operator_id: i64,
    whois_server: Option<&'static str>,
    rdap_url: Option<&'static str>,
    country: Option<CountryInfo>,
    /// Whether transferring this TLD adds a year to the expiration.
    /// `Some(true)` = yes, `Some(false)` = no, `None` = unknown.
    transfer_adds_year: Option<bool>,
}

struct CountryInfo {
    alpha2: &'static str,
    alpha3: &'static str,
    numeric: i32,
    name: &'static str,
}

fn seed_tlds(db: &TomeDb, ops: &Operators) -> Result<()> {
    let entries: Vec<SeedEntry> = vec![
        // =================================================================
        // ORIGINAL gTLDs (delegated 1985-01-01) + .int, .arpa
        // Source: IANA Root Zone Database
        // =================================================================
        SeedEntry {
            tld: "com",
            tld_type: "gTLD",
            description: "Generic top-level domain for commercial entities",
            delegation_date: Some("1985-01-01"),
            operator_id: ops.verisign,
            whois_server: Some("whois.verisign-grs.com"),
            rdap_url: Some("https://rdap.verisign.com/com/v1/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "net",
            tld_type: "gTLD",
            description: "Generic top-level domain for network infrastructure",
            delegation_date: Some("1985-01-01"),
            operator_id: ops.verisign,
            whois_server: Some("whois.verisign-grs.com"),
            rdap_url: Some("https://rdap.verisign.com/net/v1/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org",
            tld_type: "gTLD",
            description: "Generic top-level domain for organizations",
            delegation_date: Some("1985-01-01"),
            operator_id: ops.pir,
            whois_server: Some("whois.publicinterestregistry.org"),
            rdap_url: Some("https://rdap.publicinterestregistry.org/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "edu",
            tld_type: "gTLD",
            description: "Sponsored top-level domain for accredited post-secondary US educational institutions",
            delegation_date: Some("1985-01-01"),
            operator_id: ops.educause,
            whois_server: Some("whois.educause.edu"),
            rdap_url: None,
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "gov",
            tld_type: "gTLD",
            description: "Sponsored top-level domain for US government entities",
            delegation_date: Some("1985-01-01"),
            operator_id: ops.cisa,
            whois_server: Some("whois.nic.gov"),
            rdap_url: Some("https://rdap.nic.gov/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "mil",
            tld_type: "gTLD",
            description: "Sponsored top-level domain for the US military",
            delegation_date: Some("1985-01-01"),
            operator_id: ops.dod_nic,
            whois_server: None,
            rdap_url: None,
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "int",
            tld_type: "gTLD",
            description: "Sponsored top-level domain for international treaty organizations",
            delegation_date: Some("1988-11-03"),
            operator_id: ops.iana,
            whois_server: Some("whois.iana.org"),
            rdap_url: Some("https://rdap.iana.org/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "arpa",
            tld_type: "gTLD",
            description: "Infrastructure top-level domain for Internet infrastructure (Address and Routing Parameter Area)",
            delegation_date: Some("1985-01-01"),
            operator_id: ops.iab,
            whois_server: Some("whois.iana.org"),
            rdap_url: None,
            country: None,
            transfer_adds_year: Some(true),
        },

        // =================================================================
        // LATER gTLDs
        // Source: IANA Root Zone Database
        // =================================================================
        SeedEntry {
            tld: "info",
            tld_type: "gTLD",
            description: "Generic top-level domain for informational sites",
            delegation_date: Some("2001-06-26"),
            operator_id: ops.identity_digital,
            whois_server: None,
            rdap_url: Some("https://rdap.identitydigital.services/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "biz",
            tld_type: "gTLD",
            description: "Generic-restricted top-level domain for businesses",
            delegation_date: Some("2001-06-26"),
            operator_id: ops.registry_services_llc,
            whois_server: Some("whois.nic.biz"),
            rdap_url: Some("https://rdap.nic.biz/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "aero",
            tld_type: "gTLD",
            description: "Sponsored top-level domain for the air-transport industry",
            delegation_date: None,
            operator_id: ops.sita,
            whois_server: None,
            rdap_url: Some("https://rdap.identitydigital.services/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "coop",
            tld_type: "gTLD",
            description: "Sponsored top-level domain for cooperative associations",
            delegation_date: None,
            operator_id: ops.dot_cooperation,
            whois_server: None,
            rdap_url: Some("https://rdap.registry.coop/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "museum",
            tld_type: "gTLD",
            description: "Sponsored top-level domain for museums",
            delegation_date: None,
            operator_id: ops.museum_dma,
            whois_server: None,
            rdap_url: Some("https://rdap.nic.museum/"),
            country: None,
            transfer_adds_year: Some(true),
        },

        // =================================================================
        // ccTLDs — major country-code TLDs
        // Sources: IANA Root Zone Database, ISO 3166-1
        // =================================================================
        SeedEntry {
            tld: "uk",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for the United Kingdom",
            delegation_date: Some("1985-07-24"),
            operator_id: ops.nominet,
            whois_server: Some("whois.nic.uk"),
            rdap_url: Some("https://rdap.nominet.uk/uk/"),
            country: Some(CountryInfo { alpha2: "GB", alpha3: "GBR", numeric: 826, name: "United Kingdom" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "de",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Germany",
            delegation_date: Some("1986-11-05"),
            operator_id: ops.denic,
            whois_server: Some("whois.denic.de"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "DE", alpha3: "DEU", numeric: 276, name: "Germany" }),
            transfer_adds_year: Some(false),
        },
        SeedEntry {
            tld: "fr",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for France",
            delegation_date: Some("1986-09-02"),
            operator_id: ops.afnic,
            whois_server: Some("whois.nic.fr"),
            rdap_url: Some("https://rdap.nic.fr/"),
            country: Some(CountryInfo { alpha2: "FR", alpha3: "FRA", numeric: 250, name: "France" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "jp",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Japan",
            delegation_date: Some("1986-08-05"),
            operator_id: ops.jprs,
            whois_server: Some("whois.jprs.jp"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "JP", alpha3: "JPN", numeric: 392, name: "Japan" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "au",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Australia",
            delegation_date: Some("1986-03-05"),
            operator_id: ops.auda,
            whois_server: Some("whois.auda.org.au"),
            rdap_url: Some("https://rdap.cctld.au/rdap/"),
            country: Some(CountryInfo { alpha2: "AU", alpha3: "AUS", numeric: 36, name: "Australia" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "ca",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Canada",
            delegation_date: Some("1987-05-14"),
            operator_id: ops.cira,
            whois_server: Some("whois.cira.ca"),
            rdap_url: Some("https://rdap.ca.fury.ca/rdap/"),
            country: Some(CountryInfo { alpha2: "CA", alpha3: "CAN", numeric: 124, name: "Canada" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "cn",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for China",
            delegation_date: Some("1990-11-28"),
            operator_id: ops.cnnic,
            whois_server: Some("whois.cnnic.cn"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "CN", alpha3: "CHN", numeric: 156, name: "China" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "br",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Brazil",
            delegation_date: Some("1989-04-18"),
            operator_id: ops.cgi_br,
            whois_server: Some("whois.registro.br"),
            rdap_url: Some("https://rdap.registro.br/"),
            country: Some(CountryInfo { alpha2: "BR", alpha3: "BRA", numeric: 76, name: "Brazil" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "in",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for India",
            delegation_date: Some("1989-05-08"),
            operator_id: ops.nixi,
            whois_server: Some("whois.nixiregistry.in"),
            rdap_url: Some("https://rdap.nixiregistry.in/rdap/"),
            country: Some(CountryInfo { alpha2: "IN", alpha3: "IND", numeric: 356, name: "India" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "it",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Italy",
            delegation_date: Some("1987-12-23"),
            operator_id: ops.iit_cnr,
            whois_server: Some("whois.nic.it"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "IT", alpha3: "ITA", numeric: 380, name: "Italy" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "nl",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for the Netherlands",
            delegation_date: Some("1986-04-25"),
            operator_id: ops.sidn,
            whois_server: Some("whois.domain-registry.nl"),
            rdap_url: Some("https://rdap.sidn.nl/"),
            country: Some(CountryInfo { alpha2: "NL", alpha3: "NLD", numeric: 528, name: "Netherlands" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "ru",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Russia",
            delegation_date: Some("1994-04-07"),
            operator_id: ops.cc_tld_ru,
            whois_server: Some("whois.tcinet.ru"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "RU", alpha3: "RUS", numeric: 643, name: "Russian Federation" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "se",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Sweden",
            delegation_date: Some("1986-09-04"),
            operator_id: ops.iis,
            whois_server: Some("whois.iis.se"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "SE", alpha3: "SWE", numeric: 752, name: "Sweden" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "ch",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Switzerland",
            delegation_date: Some("1987-05-20"),
            operator_id: ops.switch_ch,
            whois_server: Some("whois.nic.ch"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "CH", alpha3: "CHE", numeric: 756, name: "Switzerland" }),
            transfer_adds_year: Some(false),
        },
        SeedEntry {
            tld: "us",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for the United States",
            delegation_date: Some("1985-02-15"),
            operator_id: ops.registry_services_llc,
            whois_server: Some("whois.nic.us"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "US", alpha3: "USA", numeric: 840, name: "United States of America" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "es",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Spain",
            delegation_date: Some("1988-04-14"),
            operator_id: ops.red_es,
            whois_server: Some("whois.nic.es"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "ES", alpha3: "ESP", numeric: 724, name: "Spain" }),
            transfer_adds_year: Some(false),
        },
        SeedEntry {
            tld: "io",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for British Indian Ocean Territory",
            delegation_date: Some("1997-09-16"),
            operator_id: ops.icb_io,
            whois_server: Some("whois.nic.io"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "IO", alpha3: "IOT", numeric: 86, name: "British Indian Ocean Territory" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "nz",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for New Zealand",
            delegation_date: Some("1987-01-19"),
            operator_id: ops.internetnz,
            whois_server: Some("whois.irs.net.nz"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "NZ", alpha3: "NZL", numeric: 554, name: "New Zealand" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "za",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for South Africa",
            delegation_date: Some("1990-11-07"),
            operator_id: ops.zadna,
            whois_server: None,
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "ZA", alpha3: "ZAF", numeric: 710, name: "South Africa" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "mx",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Mexico",
            delegation_date: Some("1989-02-01"),
            operator_id: ops.nic_mexico,
            whois_server: Some("whois.mx"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "MX", alpha3: "MEX", numeric: 484, name: "Mexico" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "kr",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for the Republic of Korea",
            delegation_date: Some("1986-09-29"),
            operator_id: ops.kisa,
            whois_server: Some("whois.kr"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "KR", alpha3: "KOR", numeric: 410, name: "Republic of Korea" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "at",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Austria",
            delegation_date: Some("1988-01-20"),
            operator_id: ops.nic_at,
            whois_server: Some("whois.nic.at"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "AT", alpha3: "AUT", numeric: 40, name: "Austria" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "be",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Belgium",
            delegation_date: Some("1988-08-05"),
            operator_id: ops.dns_belgium,
            whois_server: Some("whois.dns.be"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "BE", alpha3: "BEL", numeric: 56, name: "Belgium" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "dk",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Denmark",
            delegation_date: Some("1987-07-14"),
            operator_id: ops.punktum_dk,
            whois_server: Some("whois.punktum.dk"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "DK", alpha3: "DNK", numeric: 208, name: "Denmark" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "fi",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Finland",
            delegation_date: Some("1986-12-17"),
            operator_id: ops.traficom,
            whois_server: Some("whois.fi"),
            rdap_url: Some("https://rdap.fi/rdap/"),
            country: Some(CountryInfo { alpha2: "FI", alpha3: "FIN", numeric: 246, name: "Finland" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "ie",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Ireland",
            delegation_date: Some("1988-01-27"),
            operator_id: ops.iedr,
            whois_server: Some("whois.weare.ie"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "IE", alpha3: "IRL", numeric: 372, name: "Ireland" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "no",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Norway",
            delegation_date: Some("1987-03-17"),
            operator_id: ops.norid,
            whois_server: Some("whois.norid.no"),
            rdap_url: Some("https://rdap.norid.no/"),
            country: Some(CountryInfo { alpha2: "NO", alpha3: "NOR", numeric: 578, name: "Norway" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "pl",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Poland",
            delegation_date: Some("1990-07-30"),
            operator_id: ops.nask,
            whois_server: Some("whois.dns.pl"),
            rdap_url: Some("https://rdap.dns.pl/"),
            country: Some(CountryInfo { alpha2: "PL", alpha3: "POL", numeric: 616, name: "Poland" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "pt",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Portugal",
            delegation_date: Some("1988-06-30"),
            operator_id: ops.dns_pt,
            whois_server: Some("whois.dns.pt"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "PT", alpha3: "PRT", numeric: 620, name: "Portugal" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "sg",
            tld_type: "ccTLD",
            description: "Country-code top-level domain for Singapore",
            delegation_date: Some("1988-10-19"),
            operator_id: ops.sgnic,
            whois_server: Some("whois.sgnic.sg"),
            rdap_url: Some("https://rdap.sgnic.sg/rdap/"),
            country: Some(CountryInfo { alpha2: "SG", alpha3: "SGP", numeric: 702, name: "Singapore" }),
            transfer_adds_year: Some(true),
        },

        // =================================================================
        // nTLDs — new generic TLDs
        // Source: IANA Root Zone Database
        // =================================================================
        SeedEntry {
            tld: "app",
            tld_type: "nTLD",
            description: "New generic top-level domain for applications",
            delegation_date: Some("2015-06-25"),
            operator_id: ops.google_registry,
            whois_server: None,
            rdap_url: Some("https://pubapi.registry.google/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "dev",
            tld_type: "nTLD",
            description: "New generic top-level domain for developers",
            delegation_date: Some("2014-11-20"),
            operator_id: ops.google_registry,
            whois_server: None,
            rdap_url: Some("https://pubapi.registry.google/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "xyz",
            tld_type: "nTLD",
            description: "New generic top-level domain (general purpose)",
            delegation_date: Some("2014-02-06"),
            operator_id: ops.xyz_com,
            whois_server: Some("whois.nic.xyz"),
            rdap_url: Some("https://rdap.centralnic.com/xyz/"),
            country: None,
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "online",
            tld_type: "nTLD",
            description: "New generic top-level domain for online presence",
            delegation_date: Some("2015-03-06"),
            operator_id: ops.radix,
            whois_server: Some("whois.nic.online"),
            rdap_url: Some("https://rdap.radix.host/rdap/"),
            country: None,
            transfer_adds_year: Some(true),
        },

        // =================================================================
        // EFFECTIVE SLDs — second-level domains that function as TLDs
        // for registration purposes. Same registry operator as parent.
        //
        // These share the parent ccTLD's WHOIS/RDAP infrastructure.
        // =================================================================

        // -- United Kingdom --
        SeedEntry {
            tld: "co.uk",
            tld_type: "ccTLD",
            description: "Effective TLD for UK commercial entities",
            delegation_date: None,
            operator_id: ops.nominet,
            whois_server: Some("whois.nic.uk"),
            rdap_url: Some("https://rdap.nominet.uk/uk/"),
            country: Some(CountryInfo { alpha2: "GB", alpha3: "GBR", numeric: 826, name: "United Kingdom" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org.uk",
            tld_type: "ccTLD",
            description: "Effective TLD for UK non-profit organizations",
            delegation_date: None,
            operator_id: ops.nominet,
            whois_server: Some("whois.nic.uk"),
            rdap_url: Some("https://rdap.nominet.uk/uk/"),
            country: Some(CountryInfo { alpha2: "GB", alpha3: "GBR", numeric: 826, name: "United Kingdom" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "me.uk",
            tld_type: "ccTLD",
            description: "Effective TLD for UK personal sites",
            delegation_date: None,
            operator_id: ops.nominet,
            whois_server: Some("whois.nic.uk"),
            rdap_url: Some("https://rdap.nominet.uk/uk/"),
            country: Some(CountryInfo { alpha2: "GB", alpha3: "GBR", numeric: 826, name: "United Kingdom" }),
            transfer_adds_year: Some(true),
        },

        // -- Australia --
        SeedEntry {
            tld: "com.au",
            tld_type: "ccTLD",
            description: "Effective TLD for Australian commercial entities",
            delegation_date: None,
            operator_id: ops.auda,
            whois_server: Some("whois.auda.org.au"),
            rdap_url: Some("https://rdap.cctld.au/rdap/"),
            country: Some(CountryInfo { alpha2: "AU", alpha3: "AUS", numeric: 36, name: "Australia" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "net.au",
            tld_type: "ccTLD",
            description: "Effective TLD for Australian network entities",
            delegation_date: None,
            operator_id: ops.auda,
            whois_server: Some("whois.auda.org.au"),
            rdap_url: Some("https://rdap.cctld.au/rdap/"),
            country: Some(CountryInfo { alpha2: "AU", alpha3: "AUS", numeric: 36, name: "Australia" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org.au",
            tld_type: "ccTLD",
            description: "Effective TLD for Australian non-profit organizations",
            delegation_date: None,
            operator_id: ops.auda,
            whois_server: Some("whois.auda.org.au"),
            rdap_url: Some("https://rdap.cctld.au/rdap/"),
            country: Some(CountryInfo { alpha2: "AU", alpha3: "AUS", numeric: 36, name: "Australia" }),
            transfer_adds_year: Some(true),
        },

        // -- Japan --
        SeedEntry {
            tld: "co.jp",
            tld_type: "ccTLD",
            description: "Effective TLD for Japanese commercial entities",
            delegation_date: None,
            operator_id: ops.jprs,
            whois_server: Some("whois.jprs.jp"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "JP", alpha3: "JPN", numeric: 392, name: "Japan" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "ne.jp",
            tld_type: "ccTLD",
            description: "Effective TLD for Japanese network service providers",
            delegation_date: None,
            operator_id: ops.jprs,
            whois_server: Some("whois.jprs.jp"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "JP", alpha3: "JPN", numeric: 392, name: "Japan" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "or.jp",
            tld_type: "ccTLD",
            description: "Effective TLD for Japanese non-profit organizations",
            delegation_date: None,
            operator_id: ops.jprs,
            whois_server: Some("whois.jprs.jp"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "JP", alpha3: "JPN", numeric: 392, name: "Japan" }),
            transfer_adds_year: None,
        },

        // -- New Zealand --
        SeedEntry {
            tld: "co.nz",
            tld_type: "ccTLD",
            description: "Effective TLD for New Zealand commercial entities",
            delegation_date: None,
            operator_id: ops.internetnz,
            whois_server: Some("whois.irs.net.nz"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "NZ", alpha3: "NZL", numeric: 554, name: "New Zealand" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "net.nz",
            tld_type: "ccTLD",
            description: "Effective TLD for New Zealand network entities",
            delegation_date: None,
            operator_id: ops.internetnz,
            whois_server: Some("whois.irs.net.nz"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "NZ", alpha3: "NZL", numeric: 554, name: "New Zealand" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org.nz",
            tld_type: "ccTLD",
            description: "Effective TLD for New Zealand non-profit organizations",
            delegation_date: None,
            operator_id: ops.internetnz,
            whois_server: Some("whois.irs.net.nz"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "NZ", alpha3: "NZL", numeric: 554, name: "New Zealand" }),
            transfer_adds_year: Some(true),
        },

        // -- South Africa --
        SeedEntry {
            tld: "co.za",
            tld_type: "ccTLD",
            description: "Effective TLD for South African commercial entities",
            delegation_date: None,
            operator_id: ops.zadna,
            whois_server: None,
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "ZA", alpha3: "ZAF", numeric: 710, name: "South Africa" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "org.za",
            tld_type: "ccTLD",
            description: "Effective TLD for South African non-profit organizations",
            delegation_date: None,
            operator_id: ops.zadna,
            whois_server: None,
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "ZA", alpha3: "ZAF", numeric: 710, name: "South Africa" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "net.za",
            tld_type: "ccTLD",
            description: "Effective TLD for South African network entities",
            delegation_date: None,
            operator_id: ops.zadna,
            whois_server: None,
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "ZA", alpha3: "ZAF", numeric: 710, name: "South Africa" }),
            transfer_adds_year: None,
        },

        // -- Brazil --
        SeedEntry {
            tld: "com.br",
            tld_type: "ccTLD",
            description: "Effective TLD for Brazilian commercial entities",
            delegation_date: None,
            operator_id: ops.cgi_br,
            whois_server: Some("whois.registro.br"),
            rdap_url: Some("https://rdap.registro.br/"),
            country: Some(CountryInfo { alpha2: "BR", alpha3: "BRA", numeric: 76, name: "Brazil" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "net.br",
            tld_type: "ccTLD",
            description: "Effective TLD for Brazilian network entities",
            delegation_date: None,
            operator_id: ops.cgi_br,
            whois_server: Some("whois.registro.br"),
            rdap_url: Some("https://rdap.registro.br/"),
            country: Some(CountryInfo { alpha2: "BR", alpha3: "BRA", numeric: 76, name: "Brazil" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org.br",
            tld_type: "ccTLD",
            description: "Effective TLD for Brazilian non-profit organizations",
            delegation_date: None,
            operator_id: ops.cgi_br,
            whois_server: Some("whois.registro.br"),
            rdap_url: Some("https://rdap.registro.br/"),
            country: Some(CountryInfo { alpha2: "BR", alpha3: "BRA", numeric: 76, name: "Brazil" }),
            transfer_adds_year: Some(true),
        },

        // -- China --
        SeedEntry {
            tld: "com.cn",
            tld_type: "ccTLD",
            description: "Effective TLD for Chinese commercial entities",
            delegation_date: None,
            operator_id: ops.cnnic,
            whois_server: Some("whois.cnnic.cn"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "CN", alpha3: "CHN", numeric: 156, name: "China" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "net.cn",
            tld_type: "ccTLD",
            description: "Effective TLD for Chinese network entities",
            delegation_date: None,
            operator_id: ops.cnnic,
            whois_server: Some("whois.cnnic.cn"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "CN", alpha3: "CHN", numeric: 156, name: "China" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "org.cn",
            tld_type: "ccTLD",
            description: "Effective TLD for Chinese non-profit organizations",
            delegation_date: None,
            operator_id: ops.cnnic,
            whois_server: Some("whois.cnnic.cn"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "CN", alpha3: "CHN", numeric: 156, name: "China" }),
            transfer_adds_year: None,
        },

        // -- Mexico --
        SeedEntry {
            tld: "com.mx",
            tld_type: "ccTLD",
            description: "Effective TLD for Mexican commercial entities",
            delegation_date: None,
            operator_id: ops.nic_mexico,
            whois_server: Some("whois.mx"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "MX", alpha3: "MEX", numeric: 484, name: "Mexico" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org.mx",
            tld_type: "ccTLD",
            description: "Effective TLD for Mexican non-profit organizations",
            delegation_date: None,
            operator_id: ops.nic_mexico,
            whois_server: Some("whois.mx"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "MX", alpha3: "MEX", numeric: 484, name: "Mexico" }),
            transfer_adds_year: Some(true),
        },

        // -- South Korea --
        SeedEntry {
            tld: "co.kr",
            tld_type: "ccTLD",
            description: "Effective TLD for Korean commercial entities",
            delegation_date: None,
            operator_id: ops.kisa,
            whois_server: Some("whois.kr"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "KR", alpha3: "KOR", numeric: 410, name: "Republic of Korea" }),
            transfer_adds_year: None,
        },
        SeedEntry {
            tld: "or.kr",
            tld_type: "ccTLD",
            description: "Effective TLD for Korean non-profit organizations",
            delegation_date: None,
            operator_id: ops.kisa,
            whois_server: Some("whois.kr"),
            rdap_url: None,
            country: Some(CountryInfo { alpha2: "KR", alpha3: "KOR", numeric: 410, name: "Republic of Korea" }),
            transfer_adds_year: None,
        },

        // -- Singapore --
        SeedEntry {
            tld: "com.sg",
            tld_type: "ccTLD",
            description: "Effective TLD for Singaporean commercial entities",
            delegation_date: None,
            operator_id: ops.sgnic,
            whois_server: Some("whois.sgnic.sg"),
            rdap_url: Some("https://rdap.sgnic.sg/rdap/"),
            country: Some(CountryInfo { alpha2: "SG", alpha3: "SGP", numeric: 702, name: "Singapore" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "net.sg",
            tld_type: "ccTLD",
            description: "Effective TLD for Singaporean network entities",
            delegation_date: None,
            operator_id: ops.sgnic,
            whois_server: Some("whois.sgnic.sg"),
            rdap_url: Some("https://rdap.sgnic.sg/rdap/"),
            country: Some(CountryInfo { alpha2: "SG", alpha3: "SGP", numeric: 702, name: "Singapore" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org.sg",
            tld_type: "ccTLD",
            description: "Effective TLD for Singaporean non-profit organizations",
            delegation_date: None,
            operator_id: ops.sgnic,
            whois_server: Some("whois.sgnic.sg"),
            rdap_url: Some("https://rdap.sgnic.sg/rdap/"),
            country: Some(CountryInfo { alpha2: "SG", alpha3: "SGP", numeric: 702, name: "Singapore" }),
            transfer_adds_year: Some(true),
        },

        // -- India --
        SeedEntry {
            tld: "co.in",
            tld_type: "ccTLD",
            description: "Effective TLD for Indian commercial entities",
            delegation_date: None,
            operator_id: ops.nixi,
            whois_server: Some("whois.nixiregistry.in"),
            rdap_url: Some("https://rdap.nixiregistry.in/rdap/"),
            country: Some(CountryInfo { alpha2: "IN", alpha3: "IND", numeric: 356, name: "India" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "net.in",
            tld_type: "ccTLD",
            description: "Effective TLD for Indian network entities",
            delegation_date: None,
            operator_id: ops.nixi,
            whois_server: Some("whois.nixiregistry.in"),
            rdap_url: Some("https://rdap.nixiregistry.in/rdap/"),
            country: Some(CountryInfo { alpha2: "IN", alpha3: "IND", numeric: 356, name: "India" }),
            transfer_adds_year: Some(true),
        },
        SeedEntry {
            tld: "org.in",
            tld_type: "ccTLD",
            description: "Effective TLD for Indian non-profit organizations",
            delegation_date: None,
            operator_id: ops.nixi,
            whois_server: Some("whois.nixiregistry.in"),
            rdap_url: Some("https://rdap.nixiregistry.in/rdap/"),
            country: Some(CountryInfo { alpha2: "IN", alpha3: "IND", numeric: 356, name: "India" }),
            transfer_adds_year: Some(true),
        },
    ];

    // Insert all entries
    for e in &entries {
        let tld_id = db.insert_tld(&TldRow {
            id: None,
            tld: e.tld.to_string(),
            tld_ascii: None,
            dot_tld: None,
            tld_type: e.tld_type.to_string(),
            delegation_status: "active".to_string(),
            iana_id: None,
            iana_delegation_date: e.delegation_date.map(str::to_string),
            iana_last_updated: None,
            description: Some(e.description.to_string()),
            created_at: None,
            updated_at: None,
        })?;

        // Link to registry operator
        db.link_tld_registry(tld_id, e.operator_id, "operator", None, None)?;

        // WHOIS / RDAP
        if e.whois_server.is_some() || e.rdap_url.is_some() {
            db.upsert_whois_rdap(&WhoisRdapRow {
                tld_id,
                whois_server: e.whois_server.map(str::to_string),
                whois_port: 43,
                whois_model: None,
                rdap_base_url: e.rdap_url.map(str::to_string),
                rdap_conformance: None,
                gdpr_redacted: true,
                gdpr_redaction_notes: None,
                rate_limit_info: None,
                notes: None,
            })?;
        }

        // Country mapping for ccTLDs
        if let Some(c) = &e.country {
            db.insert_country_mapping(&CountryMappingRow {
                tld_id,
                iso_3166_alpha2: c.alpha2.to_string(),
                iso_3166_alpha3: Some(c.alpha3.to_string()),
                iso_3166_numeric: Some(c.numeric),
                country_name: c.name.to_string(),
                sovereign_state: None,
                local_presence_required: false,
                local_presence_notes: None,
            })?;
        }

        // Transfer rules
        if let Some(adds_year) = e.transfer_adds_year {
            db.upsert_transfer_rules(&TransferRulesRow {
                tld_id,
                inter_registrar_transfer: true,
                requires_authcode: true,
                authcode_format: None,
                authcode_validity_days: None,
                transfer_approval_method: None,
                approval_timeout_days: Some(5),
                new_registration_lock_days: Some(60),
                post_transfer_lock_days: Some(60),
                transfer_adds_year: adds_year,
                bulk_transfer_supported: false,
                registry_transfer_supported: false,
                custom_transfer_process: false,
                custom_process_notes: None,
                notes: None,
            })?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_populates_database() {
        let db = TomeDb::open_in_memory().expect("failed to open db");
        seed(&db).expect("seed failed");

        // We should have a substantial number of entries
        let count = db.count_tlds().expect("count failed");
        assert!(count >= 60, "expected at least 60 TLDs, got {count}");

        // Verify a known gTLD
        let com = db
            .get_tld("com")
            .expect("lookup failed")
            .expect(".com should exist");
        assert_eq!(com.tld_type, "gTLD");
        assert_eq!(com.iana_delegation_date.as_deref(), Some("1985-01-01"));

        // Verify its WHOIS/RDAP via overview
        let ov = db
            .get_tld_overview("com")
            .expect("overview failed")
            .expect("overview should exist");
        assert_eq!(ov.whois_server.as_deref(), Some("whois.verisign-grs.com"));
        assert_eq!(
            ov.rdap_base_url.as_deref(),
            Some("https://rdap.verisign.com/com/v1/")
        );
        assert_eq!(
            ov.registry_operator.as_deref(),
            Some("VeriSign Global Registry Services")
        );

        // Verify a ccTLD
        let uk = db
            .get_tld("uk")
            .expect("lookup failed")
            .expect(".uk should exist");
        assert_eq!(uk.tld_type, "ccTLD");

        let uk_ov = db
            .get_tld_overview("uk")
            .expect("overview")
            .expect("uk overview");
        assert_eq!(uk_ov.country_name.as_deref(), Some("United Kingdom"));
        assert_eq!(uk_ov.iso_3166_alpha2.as_deref(), Some("GB"));

        // Verify an effective SLD
        let co_uk = db
            .get_tld("co.uk")
            .expect("lookup")
            .expect("co.uk should exist");
        assert_eq!(co_uk.tld_type, "ccTLD");

        let co_uk_ov = db
            .get_tld_overview("co.uk")
            .expect("overview")
            .expect("co.uk overview");
        assert_eq!(co_uk_ov.registry_operator.as_deref(), Some("Nominet UK"));
        assert_eq!(co_uk_ov.country_name.as_deref(), Some("United Kingdom"));

        // Verify an nTLD
        let app = db
            .get_tld("app")
            .expect("lookup")
            .expect(".app should exist");
        assert_eq!(app.tld_type, "nTLD");

        let app_ov = db
            .get_tld_overview("app")
            .expect("overview")
            .expect("app overview");
        assert_eq!(
            app_ov.registry_operator.as_deref(),
            Some("Charleston Road Registry Inc.")
        );
        assert_eq!(
            app_ov.rdap_base_url.as_deref(),
            Some("https://pubapi.registry.google/rdap/")
        );

        // Verify infrastructure TLD
        let arpa = db
            .get_tld("arpa")
            .expect("lookup")
            .expect(".arpa should exist");
        assert_eq!(arpa.tld_type, "gTLD");
        assert!(arpa
            .description
            .as_deref()
            .unwrap_or("")
            .contains("Infrastructure"));

        // Verify .int delegation date
        let int = db
            .get_tld("int")
            .expect("lookup")
            .expect(".int should exist");
        assert_eq!(int.iana_delegation_date.as_deref(), Some("1988-11-03"));
    }

    #[test]
    fn test_seed_all_types_present() {
        let db = TomeDb::open_in_memory().expect("failed to open db");
        seed(&db).expect("seed failed");

        let gtlds = db.list_tlds_by_type("gTLD").expect("list gTLDs");
        let cctlds = db.list_tlds_by_type("ccTLD").expect("list ccTLDs");
        let ntlds = db.list_tlds_by_type("nTLD").expect("list nTLDs");

        assert!(!gtlds.is_empty(), "should have gTLDs");
        assert!(!cctlds.is_empty(), "should have ccTLDs");
        assert!(!ntlds.is_empty(), "should have nTLDs");

        // Verify at least the known counts
        assert!(
            gtlds.len() >= 13,
            "at least 13 gTLDs (originals + info/biz/aero/coop/museum)"
        );
        assert!(
            cctlds.len() >= 30,
            "at least 30 ccTLD entries (incl. effective SLDs)"
        );
        assert!(ntlds.len() >= 4, "at least 4 nTLDs");
    }
}
