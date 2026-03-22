mod queries;
mod schema;
pub mod seed;

use std::path::Path;

use rusqlite::Connection;

use crate::error::Result;

pub use queries::*;
pub use schema::SCHEMA_VERSION;

/// Open (or create) a Tome database at the given path and ensure the schema is initialized.
pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    schema::initialize(&conn)?;
    Ok(conn)
}

/// Create an in-memory Tome database — useful for testing and ephemeral use.
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    schema::initialize(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::{GlossaryCategory, GlossaryTerm};
    use crate::record_type::{RecordStatus, RecordType};
    use crate::tld::{ContactRequirements, DnssecSupport, Tld, TldType};

    fn make_tld(tld: &str, tld_type: TldType, name: &str, registry: &str) -> Tld {
        Tld {
            tld: tld.to_string(),
            tld_type,
            name: name.to_string(),
            registry: registry.to_string(),
            whois_server: None,
            rdap_url: None,
            dnssec: DnssecSupport::Unknown,
            delegation_date: None,
            references: vec![],
            idn_support: false,
            idn_type: None,
            min_length: None,
            max_length: None,
            nameservers_min: None,
            nameservers_max: None,
            restrictions: None,
            allowed_countries: vec![],
            transfer_auth_required: None,
            creation_requires_prevalidation: None,
            validation_category: None,
            create_periods: vec![],
            renew_periods: vec![],
            auto_renew_periods: vec![],
            transfer_periods: vec![],
            redemption_period: None,
            add_grace_period: None,
            renew_grace_period: None,
            auto_renew_grace_period: None,
            transfer_grace_period: None,
            pending_delete_period: None,
            features: vec![],
            premium_support: None,
            restore_includes_renew: None,
            renewal_on_transfer: None,
            contacts: vec![],
            allowed_dnssec_records: None,
            allowed_dnssec_algorithms: vec![],
            whois_exposure: None,
            gdpr_category: None,
            jurisdiction: None,
        }
    }

    #[test]
    fn test_tld_roundtrip() {
        let conn = open_in_memory().unwrap();

        let mut tld = make_tld("com", TldType::Generic, "Commercial", "VeriSign Global Registry Services");
        tld.whois_server = Some("whois.verisign-grs.com".to_string());
        tld.rdap_url = Some("https://rdap.verisign.com/com/v1/".to_string());
        tld.dnssec = DnssecSupport::Signed;
        tld.idn_support = true;
        tld.min_length = Some(1);
        tld.max_length = Some(63);
        tld.nameservers_min = Some(2);
        tld.nameservers_max = Some(13);
        tld.delegation_date = Some("1985-01-01".to_string());
        tld.references = vec!["RFC 1591".to_string()];
        tld.create_periods = vec![1, 2, 3, 5, 10];
        tld.features = vec!["CREATE".to_string(), "RENEW".to_string(), "TRANSFER".to_string()];
        tld.contacts = vec![ContactRequirements {
            contact_type: "registrant".to_string(),
            min: Some(1),
            max: Some(1),
            required: Some(true),
            organization_allowed: Some(true),
            organization_required: Some(false),
            scope: Some("registrar".to_string()),
        }];
        tld.allowed_dnssec_algorithms = vec![8, 13, 14];
        tld.whois_exposure = Some("full".to_string());
        tld.gdpr_category = Some("data_export".to_string());

        upsert_tld(&conn, &tld).unwrap();

        let result = get_tld(&conn, "com").unwrap().unwrap();
        assert_eq!(result.tld, "com");
        assert_eq!(result.tld_type, TldType::Generic);
        assert_eq!(result.registry, "VeriSign Global Registry Services");
        assert_eq!(result.references, vec!["RFC 1591"]);
        assert_eq!(result.min_length, Some(1));
        assert_eq!(result.max_length, Some(63));
        assert_eq!(result.create_periods, vec![1, 2, 3, 5, 10]);
        assert_eq!(result.features, vec!["CREATE", "RENEW", "TRANSFER"]);
        assert_eq!(result.contacts.len(), 1);
        assert_eq!(result.contacts[0].contact_type, "registrant");
        assert_eq!(result.contacts[0].required, Some(true));
        assert_eq!(result.allowed_dnssec_algorithms, vec![8, 13, 14]);
        assert_eq!(result.whois_exposure, Some("full".to_string()));

        // Case-insensitive with leading dot
        let result = get_tld(&conn, ".COM").unwrap().unwrap();
        assert_eq!(result.tld, "com");
    }

    #[test]
    fn test_tld_search() {
        let conn = open_in_memory().unwrap();

        let tlds = vec![
            make_tld("com", TldType::Generic, "Commercial", "VeriSign"),
            make_tld("uk", TldType::CountryCode, "United Kingdom", "Nominet"),
        ];

        for t in &tlds {
            upsert_tld(&conn, t).unwrap();
        }

        let results = search_tlds(&conn, "United").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tld, "uk");

        let results = search_tlds(&conn, "VeriSign").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tld, "com");
    }

    #[test]
    fn test_record_type_roundtrip() {
        let conn = open_in_memory().unwrap();

        let rt = RecordType {
            name: "A".to_string(),
            type_code: 1,
            summary: "IPv4 address record".to_string(),
            description: "Maps a domain name to an IPv4 address.".to_string(),
            rdata_format: "IPv4 address".to_string(),
            example: "example.com. 300 IN A 93.184.216.34".to_string(),
            rfcs: vec!["RFC 1035".to_string()],
            status: RecordStatus::Active,
            common: true,
            related: vec!["AAAA".to_string()],
        };

        upsert_record_type(&conn, &rt).unwrap();

        let result = get_record_type(&conn, "a").unwrap().unwrap();
        assert_eq!(result.name, "A");
        assert_eq!(result.type_code, 1);
        assert!(result.common);
        assert_eq!(result.rfcs, vec!["RFC 1035"]);
        assert_eq!(result.related, vec!["AAAA"]);

        let result = get_record_type_by_code(&conn, 1).unwrap().unwrap();
        assert_eq!(result.name, "A");
    }

    #[test]
    fn test_glossary_roundtrip() {
        let conn = open_in_memory().unwrap();

        let term = GlossaryTerm {
            term: "Registrar".to_string(),
            abbreviation: None,
            summary: "An organization that registers domain names on behalf of registrants."
                .to_string(),
            description: "A registrar is an accredited entity that processes domain name registrations for end users.".to_string(),
            category: GlossaryCategory::Registration,
            related: vec!["Registry".to_string(), "Registrant".to_string()],
            references: vec!["RFC 5731".to_string()],
        };

        upsert_glossary_term(&conn, &term).unwrap();

        let result = get_glossary_term(&conn, "registrar").unwrap().unwrap();
        assert_eq!(result.term, "Registrar");
        assert_eq!(result.category, GlossaryCategory::Registration);
        let mut related = result.related.clone();
        related.sort();
        assert_eq!(related, vec!["Registrant", "Registry"]);
        assert_eq!(result.references, vec!["RFC 5731"]);
    }

    #[test]
    fn test_list_and_count() {
        let conn = open_in_memory().unwrap();

        let tlds = vec![
            make_tld("com", TldType::Generic, "Commercial", "VeriSign"),
            make_tld("uk", TldType::CountryCode, "United Kingdom", "Nominet"),
        ];
        for t in &tlds {
            upsert_tld(&conn, t).unwrap();
        }

        assert_eq!(count_tlds(&conn, None).unwrap(), 2);
        assert_eq!(count_tlds(&conn, Some(&TldType::Generic)).unwrap(), 1);
        assert_eq!(
            list_tlds(&conn, Some(&TldType::CountryCode))
                .unwrap()
                .len(),
            1
        );
    }
}
