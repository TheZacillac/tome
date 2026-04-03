//! Extended seed data — remaining ccTLDs and popular nTLDs.
//!
//! Every entry here is sourced from:
//! - **IANA Root Zone Database** — TLD strings, delegation status, registry operators
//! - **ISO 3166-1** — country codes and country names for ccTLDs
//! - **ComLaude/MarkMonitor registrar API** — cross-referenced for validation
//!
//! Call [`seed_extended`] *after* [`crate::seed::seed`] — it skips TLDs that
//! were already populated by the initial seed.

use crate::db::{CountryMappingRow, RegistryOperatorRow, TldRow, TomeDb};
use crate::error::Result;

/// Populate the database with the extended TLD dataset.
///
/// Adds ~200 additional ccTLDs and ~50 popular nTLDs on top of the core
/// seed from [`crate::seed::seed`].  All inserts run within a single
/// transaction — failure rolls back cleanly.
pub fn seed_extended(db: &TomeDb) -> Result<()> {
    db.connection().execute_batch("BEGIN")?;
    match (|| {
        seed_remaining_cctlds(db)?;
        seed_popular_ntlds(db)?;
        seed_brand_ntlds(db)?;
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
// Helpers
// ---------------------------------------------------------------------------

/// Insert a TLD + operator in one shot. Returns the new TLD row id.
fn insert_tld_with_operator(
    db: &TomeDb,
    tld: &str,
    tld_type: &str,
    description: &str,
    operator_name: &str,
    operator_hq: Option<&str>,
) -> Result<i64> {
    let tld_id = db.insert_tld(&TldRow {
        id: None,
        tld: tld.to_string(),
        tld_ascii: None,
        dot_tld: None,
        tld_type: tld_type.to_string(),
        delegation_status: "active".to_string(),
        iana_id: None,
        iana_delegation_date: None,
        iana_last_updated: None,
        description: Some(description.to_string()),
        created_at: None,
        updated_at: None,
    })?;

    let op_id = db.insert_registry_operator(&RegistryOperatorRow {
        id: None,
        name: operator_name.to_string(),
        short_name: None,
        website: None,
        abuse_contact_email: None,
        abuse_contact_phone: None,
        hq_country: operator_hq.map(str::to_string),
        parent_company: None,
        icann_accredited: false,
        notes: None,
    })?;

    db.link_tld_registry(tld_id, op_id, "operator", None, None)?;
    Ok(tld_id)
}

// ---------------------------------------------------------------------------
// Remaining ccTLDs
// ---------------------------------------------------------------------------

/// (tld, operator_name, operator_hq_country, iso_alpha2, country_name)
///
/// Operators sourced from IANA Root Zone Database.
/// ISO codes from ISO 3166-1.
const REMAINING_CCTLDS: &[(&str, &str, &str, &str, &str)] = &[
    // A
    (
        "ac",
        "Internet Computer Bureau Limited",
        "GB",
        "AC",
        "Ascension Island",
    ),
    ("ad", "Andorra Telecom", "AD", "AD", "Andorra"),
    (
        "ae",
        "Telecommunications and Digital Government Regulatory Authority (TDRA)",
        "AE",
        "AE",
        "United Arab Emirates",
    ),
    (
        "af",
        "Ministry of Communications and IT",
        "AF",
        "AF",
        "Afghanistan",
    ),
    (
        "ag",
        "UHSA School of Medicine",
        "AG",
        "AG",
        "Antigua and Barbuda",
    ),
    ("ai", "Government of Anguilla", "AI", "AI", "Anguilla"),
    (
        "al",
        "Electronic and Postal Communications Authority (AKEP)",
        "AL",
        "AL",
        "Albania",
    ),
    ("am", "Internet Society NGO", "AM", "AM", "Armenia"),
    (
        "ao",
        "Ministry of Telecommunications and IT (MTTI)",
        "AO",
        "AO",
        "Angola",
    ),
    (
        "aq",
        "Antarctica Network Information Centre Limited",
        "NZ",
        "AQ",
        "Antarctica",
    ),
    ("ar", "NIC Argentina", "AR", "AR", "Argentina"),
    ("as", "AS Domain Registry", "AS", "AS", "American Samoa"),
    ("aw", "SETAR", "AW", "AW", "Aruba"),
    (
        "ax",
        "Ålands landskapsregering",
        "AX",
        "AX",
        "Åland Islands",
    ),
    ("az", "IntraNS", "AZ", "AZ", "Azerbaijan"),
    // B
    (
        "ba",
        "University Tele-Informatic Center (UTIC)",
        "BA",
        "BA",
        "Bosnia and Herzegovina",
    ),
    (
        "bb",
        "Ministry of Innovation, Science and Smart Technology",
        "BB",
        "BB",
        "Barbados",
    ),
    (
        "bd",
        "Posts and Telecommunications Division",
        "BD",
        "BD",
        "Bangladesh",
    ),
    ("bf", "ARCEP Burkina Faso", "BF", "BF", "Burkina Faso"),
    ("bg", "Register.BG", "BG", "BG", "Bulgaria"),
    (
        "bh",
        "Telecommunications Regulatory Authority (TRA)",
        "BH",
        "BH",
        "Bahrain",
    ),
    (
        "bi",
        "Centre National de l'Informatique",
        "BI",
        "BI",
        "Burundi",
    ),
    ("bj", "ARCEP Benin", "BJ", "BJ", "Benin"),
    (
        "bm",
        "Registry General Department, Ministry of Home Affairs",
        "BM",
        "BM",
        "Bermuda",
    ),
    (
        "bn",
        "Authority for Info-communications Technology Industry (AITI)",
        "BN",
        "BN",
        "Brunei Darussalam",
    ),
    (
        "bo",
        "Agencia para el Desarrollo de la Información de la Sociedad en Bolivia",
        "BO",
        "BO",
        "Bolivia",
    ),
    ("bs", "University of The Bahamas", "BS", "BS", "Bahamas"),
    (
        "bt",
        "Ministry of Information and Communications",
        "BT",
        "BT",
        "Bhutan",
    ),
    (
        "bw",
        "Botswana Communications Regulatory Authority (BOCRA)",
        "BW",
        "BW",
        "Botswana",
    ),
    (
        "by",
        "Belarusian Cloud Technologies LLC",
        "BY",
        "BY",
        "Belarus",
    ),
    ("bz", "University of Belize", "BZ", "BZ", "Belize"),
    // C
    (
        "cc",
        "eNIC Cocos (Keeling) Islands Pty. Ltd.",
        "CC",
        "CC",
        "Cocos (Keeling) Islands",
    ),
    (
        "cd",
        "Office Congolais des Postes et Télécommunications (OCPT)",
        "CD",
        "CD",
        "Congo, Democratic Republic of the",
    ),
    (
        "cf",
        "Société Centrafricaine de Télécommunications (SOCATEL)",
        "CF",
        "CF",
        "Central African Republic",
    ),
    ("cg", "Interpoint Switzerland", "CG", "CG", "Congo"),
    ("ci", "ARTCI", "CI", "CI", "Côte d'Ivoire"),
    (
        "ck",
        "Telecom Cook Islands Ltd.",
        "CK",
        "CK",
        "Cook Islands",
    ),
    ("cl", "NIC Chile (University of Chile)", "CL", "CL", "Chile"),
    ("cm", "ANTIC", "CM", "CM", "Cameroon"),
    (
        "co",
        "Ministry of Information and Communications Technologies (MinTIC)",
        "CO",
        "CO",
        "Colombia",
    ),
    (
        "cr",
        "Academia Nacional de Ciencias",
        "CR",
        "CR",
        "Costa Rica",
    ),
    ("cu", "CENIAInternet", "CU", "CU", "Cuba"),
    (
        "cv",
        "Agência Reguladora Multissectorial da Economia (ARME)",
        "CV",
        "CV",
        "Cabo Verde",
    ),
    ("cw", "University of Curaçao", "CW", "CW", "Curaçao"),
    (
        "cx",
        "Christmas Island Domain Administration Limited",
        "CX",
        "CX",
        "Christmas Island",
    ),
    ("cy", "University of Cyprus", "CY", "CY", "Cyprus"),
    ("cz", "CZ.NIC, z.s.p.o", "CZ", "CZ", "Czechia"),
    // D
    ("dj", "Djibouti Telecom S.A", "DJ", "DJ", "Djibouti"),
    ("dm", "DotDM Corporation", "DM", "DM", "Dominica"),
    (
        "do",
        "Pontificia Universidad Católica Madre y Maestra",
        "DO",
        "DO",
        "Dominican Republic",
    ),
    ("dz", "CERIST", "DZ", "DZ", "Algeria"),
    // E
    ("ec", "ECUADORDOMAIN S.A.", "EC", "EC", "Ecuador"),
    (
        "ee",
        "Eesti Interneti Sihtasutus (EIS)",
        "EE",
        "EE",
        "Estonia",
    ),
    (
        "eg",
        "Egyptian Universities Network (EUN)",
        "EG",
        "EG",
        "Egypt",
    ),
    (
        "er",
        "Eritrea Telecommunication Services Corporation (EriTel)",
        "ER",
        "ER",
        "Eritrea",
    ),
    ("et", "Ethio telecom", "ET", "ET", "Ethiopia"),
    ("eu", "EURid vzw", "BE", "EU", "European Union"),
    // F
    (
        "fj",
        "University of the South Pacific IT Services",
        "FJ",
        "FJ",
        "Fiji",
    ),
    (
        "fk",
        "Falkland Islands Government",
        "FK",
        "FK",
        "Falkland Islands (Malvinas)",
    ),
    (
        "fm",
        "FSM Telecommunications Corporation",
        "FM",
        "FM",
        "Micronesia, Federated States of",
    ),
    ("fo", "FO Council", "FO", "FO", "Faroe Islands"),
    // G
    ("ga", "ANINF", "GA", "GA", "Gabon"),
    (
        "gd",
        "National Telecommunications Regulatory Commission (NTRC)",
        "GD",
        "GD",
        "Grenada",
    ),
    ("ge", "Caucasus Online LLC", "GE", "GE", "Georgia"),
    ("gf", "CANAL+ TELECOM", "GF", "GF", "French Guiana"),
    ("gg", "Island Networks Ltd.", "GG", "GG", "Guernsey"),
    (
        "gh",
        "Network Computer Systems Limited",
        "GH",
        "GH",
        "Ghana",
    ),
    ("gi", "Sapphire Networks", "GI", "GI", "Gibraltar"),
    ("gl", "TELE Greenland A/S", "GL", "GL", "Greenland"),
    ("gm", "GM-NIC", "GM", "GM", "Gambia"),
    (
        "gn",
        "Centre National des Sciences Halieutiques de Boussoura",
        "GN",
        "GN",
        "Guinea",
    ),
    (
        "gp",
        "Networking Technologies Group",
        "GP",
        "GP",
        "Guadeloupe",
    ),
    ("gq", "GETESA", "GQ", "GQ", "Equatorial Guinea"),
    ("gr", "ICS-FORTH GR", "GR", "GR", "Greece"),
    (
        "gs",
        "Government of South Georgia and South Sandwich Islands",
        "GS",
        "GS",
        "South Georgia and the South Sandwich Islands",
    ),
    (
        "gt",
        "Universidad del Valle de Guatemala",
        "GT",
        "GT",
        "Guatemala",
    ),
    ("gu", "University of Guam", "GU", "GU", "Guam"),
    (
        "gw",
        "Autoridade Reguladora Nacional (ARN)",
        "GW",
        "GW",
        "Guinea-Bissau",
    ),
    ("gy", "University of Guyana", "GY", "GY", "Guyana"),
    // H
    (
        "hk",
        "Hong Kong Internet Registration Corporation Ltd.",
        "HK",
        "HK",
        "Hong Kong",
    ),
    (
        "hm",
        "HM Domain Registry",
        "HM",
        "HM",
        "Heard Island and McDonald Islands",
    ),
    (
        "hn",
        "Red de Desarrollo Sostenible Honduras",
        "HN",
        "HN",
        "Honduras",
    ),
    (
        "hr",
        "CARNet - Croatian Academic and Research Network",
        "HR",
        "HR",
        "Croatia",
    ),
    ("ht", "Consortium FDS/RDDH", "HT", "HT", "Haiti"),
    (
        "hu",
        "Council of Hungarian Internet Providers (CHIP)",
        "HU",
        "HU",
        "Hungary",
    ),
    // I
    ("id", "PANDI", "ID", "ID", "Indonesia"),
    (
        "il",
        "The Israel Internet Association (RA)",
        "IL",
        "IL",
        "Israel",
    ),
    ("im", "Isle of Man Government", "IM", "IM", "Isle of Man"),
    (
        "iq",
        "Communications and Media Commission (CMC)",
        "IQ",
        "IQ",
        "Iraq",
    ),
    (
        "ir",
        "Institute for Research in Fundamental Sciences",
        "IR",
        "IR",
        "Iran, Islamic Republic of",
    ),
    (
        "is",
        "ISNIC - Internet á Íslandi hf.",
        "IS",
        "IS",
        "Iceland",
    ),
    // J
    ("je", "Island Networks (Jersey) Ltd.", "JE", "JE", "Jersey"),
    ("jm", "University of West Indies", "JM", "JM", "Jamaica"),
    (
        "jo",
        "Ministry of Digital Economy and Entrepreneurship (MoDEE)",
        "JO",
        "JO",
        "Jordan",
    ),
    // K
    (
        "ke",
        "Kenya Network Information Center (KeNIC)",
        "KE",
        "KE",
        "Kenya",
    ),
    (
        "kg",
        "AsiaInfo Telecommunication Enterprise",
        "KG",
        "KG",
        "Kyrgyzstan",
    ),
    (
        "kh",
        "Telecommunication Regulator of Cambodia (TRC)",
        "KH",
        "KH",
        "Cambodia",
    ),
    (
        "ki",
        "Ministry of Information, Communications and Transport (MICT)",
        "KI",
        "KI",
        "Kiribati",
    ),
    ("km", "Comores Telecom", "KM", "KM", "Comoros"),
    (
        "kn",
        "Ministry of Finance, Sustainable Development IT",
        "KN",
        "KN",
        "Saint Kitts and Nevis",
    ),
    (
        "kp",
        "Star Joint Venture Company",
        "KP",
        "KP",
        "Korea, Democratic People's Republic of",
    ),
    (
        "kw",
        "Communications and Information Technology Regulatory Authority",
        "KW",
        "KW",
        "Kuwait",
    ),
    (
        "ky",
        "Utility Regulation and Competition Office (OfReg)",
        "KY",
        "KY",
        "Cayman Islands",
    ),
    (
        "kz",
        "Association of IT Companies of Kazakhstan",
        "KZ",
        "KZ",
        "Kazakhstan",
    ),
    // L
    (
        "la",
        "Lao National Internet Center (LANIC)",
        "LA",
        "LA",
        "Lao People's Democratic Republic",
    ),
    ("lb", "Internet Society Lebanon", "LB", "LB", "Lebanon"),
    ("lc", "University of Puerto Rico", "LC", "LC", "Saint Lucia"),
    ("li", "SWITCH", "LI", "LI", "Liechtenstein"),
    (
        "lk",
        "Council for Information Technology LK Domain Registrar",
        "LK",
        "LK",
        "Sri Lanka",
    ),
    (
        "lr",
        "Data Technology Solutions, Inc.",
        "LR",
        "LR",
        "Liberia",
    ),
    (
        "ls",
        "Lesotho Network Information Centre (LSNIC)",
        "LS",
        "LS",
        "Lesotho",
    ),
    (
        "lt",
        "Kaunas University of Technology",
        "LT",
        "LT",
        "Lithuania",
    ),
    ("lu", "RESTENA", "LU", "LU", "Luxembourg"),
    ("lv", "University of Latvia, NIC", "LV", "LV", "Latvia"),
    (
        "ly",
        "General Authority of Communications and Informatics (GACI)",
        "LY",
        "LY",
        "Libya",
    ),
    // M
    ("ma", "ANRT", "MA", "MA", "Morocco"),
    (
        "mc",
        "Direction des Plateformes et des Ressources Numériques",
        "MC",
        "MC",
        "Monaco",
    ),
    (
        "md",
        "IP Serviciul Tehnologia Informatiei si Securitate Cibernetica",
        "MD",
        "MD",
        "Moldova, Republic of",
    ),
    ("me", "Government of Montenegro", "ME", "ME", "Montenegro"),
    (
        "mg",
        "NIC-MG (Network Information Center Madagascar)",
        "MG",
        "MG",
        "Madagascar",
    ),
    (
        "mh",
        "Office of the Cabinet",
        "MH",
        "MH",
        "Marshall Islands",
    ),
    (
        "mk",
        "Macedonian Academic Research Network Skopje",
        "MK",
        "MK",
        "North Macedonia",
    ),
    (
        "ml",
        "Agence des Technologies de l'Information et de la Communication",
        "ML",
        "ML",
        "Mali",
    ),
    (
        "mm",
        "Ministry of Transport and Communications",
        "MM",
        "MM",
        "Myanmar",
    ),
    ("mn", "Datacom Co., Ltd.", "MN", "MN", "Mongolia"),
    (
        "mo",
        "Macao Post and Telecommunications Bureau (CTT)",
        "MO",
        "MO",
        "Macao",
    ),
    (
        "mp",
        "Saipan Datacom, Inc.",
        "MP",
        "MP",
        "Northern Mariana Islands",
    ),
    ("mq", "CANAL+ TELECOM", "MQ", "MQ", "Martinique"),
    (
        "mr",
        "Université de Nouakchott Al Aasriya",
        "MR",
        "MR",
        "Mauritania",
    ),
    ("ms", "MNI Networks Ltd.", "MS", "MS", "Montserrat"),
    ("mt", "NIC (Malta)", "MT", "MT", "Malta"),
    ("mu", "Internet Direct Ltd", "MU", "MU", "Mauritius"),
    ("mv", "Dhivehi Raajjeyge Gulhun PLC", "MV", "MV", "Maldives"),
    (
        "mw",
        "Malawi Sustainable Development Network Programme",
        "MW",
        "MW",
        "Malawi",
    ),
    ("my", "MYNIC Berhad", "MY", "MY", "Malaysia"),
    (
        "mz",
        "Centro de Informática de Universidade Eduardo Mondlane",
        "MZ",
        "MZ",
        "Mozambique",
    ),
    // N
    (
        "na",
        "Namibian Network Information Center",
        "NA",
        "NA",
        "Namibia",
    ),
    (
        "nc",
        "Office des Postes et Télécommunications",
        "NC",
        "NC",
        "New Caledonia",
    ),
    ("ne", "SONITEL", "NE", "NE", "Niger"),
    (
        "nf",
        "Norfolk Island Data Services",
        "NF",
        "NF",
        "Norfolk Island",
    ),
    (
        "ng",
        "Nigeria Internet Registration Association",
        "NG",
        "NG",
        "Nigeria",
    ),
    (
        "ni",
        "Universidad Nacional del Ingeniería",
        "NI",
        "NI",
        "Nicaragua",
    ),
    (
        "np",
        "Mercantile Communications Pvt. Ltd.",
        "NP",
        "NP",
        "Nepal",
    ),
    ("nr", "CENPAC NET", "NR", "NR", "Nauru"),
    ("nu", "The IUSN Foundation", "NU", "NU", "Niue"),
    // O
    (
        "om",
        "Telecommunications Regulatory Authority (TRA)",
        "OM",
        "OM",
        "Oman",
    ),
    // P
    (
        "pa",
        "Universidad Tecnológica de Panamá",
        "PA",
        "PA",
        "Panama",
    ),
    ("pe", "Red Científica Peruana", "PE", "PE", "Peru"),
    (
        "pf",
        "Gouvernement de la Polynésie française",
        "PF",
        "PF",
        "French Polynesia",
    ),
    (
        "pg",
        "PNG DNS Administration",
        "PG",
        "PG",
        "Papua New Guinea",
    ),
    ("ph", "PH Domain Foundation", "PH", "PH", "Philippines"),
    ("pk", "PKNIC", "PK", "PK", "Pakistan"),
    ("pm", "AFNIC", "FR", "PM", "Saint Pierre and Miquelon"),
    (
        "pn",
        "Pitcairn Island Administration",
        "PN",
        "PN",
        "Pitcairn",
    ),
    (
        "pr",
        "Gauss Research Laboratory Inc.",
        "PR",
        "PR",
        "Puerto Rico",
    ),
    (
        "ps",
        "Ministry of Telecommunications & IT",
        "PS",
        "PS",
        "Palestine, State of",
    ),
    (
        "pw",
        "Micronesia Investment and Development Corporation",
        "PW",
        "PW",
        "Palau",
    ),
    ("py", "NIC-PY", "PY", "PY", "Paraguay"),
    // Q
    (
        "qa",
        "Communications Regulatory Authority",
        "QA",
        "QA",
        "Qatar",
    ),
    // R
    ("re", "AFNIC", "FR", "RE", "Réunion"),
    (
        "ro",
        "National Institute for R&D in Informatics",
        "RO",
        "RO",
        "Romania",
    ),
    (
        "rs",
        "Serbian National Internet Domain Registry (RNIDS)",
        "RS",
        "RS",
        "Serbia",
    ),
    (
        "rw",
        "Rwanda Internet Community and Technology Alliance (RICTA)",
        "RW",
        "RW",
        "Rwanda",
    ),
    // S
    (
        "sa",
        "Communications, Space and Technology Commission",
        "SA",
        "SA",
        "Saudi Arabia",
    ),
    (
        "sb",
        "Solomon Telekom Company Limited",
        "SB",
        "SB",
        "Solomon Islands",
    ),
    ("sc", "VCS Pty Ltd", "SC", "SC", "Seychelles"),
    ("sd", "Sudan Internet Society", "SD", "SD", "Sudan"),
    (
        "sh",
        "Government of St. Helena",
        "SH",
        "SH",
        "Saint Helena, Ascension and Tristan da Cunha",
    ),
    (
        "si",
        "Academic and Research Network of Slovenia (ARNES)",
        "SI",
        "SI",
        "Slovenia",
    ),
    ("sk", "SK-NIC, a.s.", "SK", "SK", "Slovakia"),
    ("sl", "Sierratel", "SL", "SL", "Sierra Leone"),
    (
        "sm",
        "Telecom Italia San Marino S.p.A.",
        "SM",
        "SM",
        "San Marino",
    ),
    ("sn", "Université Cheikh Anta Diop", "SN", "SN", "Senegal"),
    (
        "so",
        "Ministry of Post and Telecommunications",
        "SO",
        "SO",
        "Somalia",
    ),
    ("sr", "Telesur", "SR", "SR", "Suriname"),
    (
        "ss",
        "National Communication Authority (NCA)",
        "SS",
        "SS",
        "South Sudan",
    ),
    ("st", "Tecnisys", "ST", "ST", "Sao Tome and Principe"),
    ("sv", "SVNet", "SV", "SV", "El Salvador"),
    (
        "sx",
        "SX Registry SA B.V.",
        "SX",
        "SX",
        "Sint Maarten (Dutch part)",
    ),
    ("sy", "NAITS", "SY", "SY", "Syrian Arab Republic"),
    ("sz", "University of Eswatini", "SZ", "SZ", "Eswatini"),
    // T
    ("tc", "Melrex TC", "TC", "TC", "Turks and Caicos Islands"),
    ("td", "ADETIC", "TD", "TD", "Chad"),
    ("tf", "AFNIC", "FR", "TF", "French Southern Territories"),
    ("tg", "ARCEP Togo", "TG", "TG", "Togo"),
    (
        "th",
        "Thai Network Information Center Foundation",
        "TH",
        "TH",
        "Thailand",
    ),
    (
        "tj",
        "Information Technology Center",
        "TJ",
        "TJ",
        "Tajikistan",
    ),
    (
        "tk",
        "Telecommunication Tokelau Corporation (Teletok)",
        "TK",
        "TK",
        "Tokelau",
    ),
    (
        "tl",
        "Autoridade Nacional de Comunicações",
        "TL",
        "TL",
        "Timor-Leste",
    ),
    ("tm", "TM Domain Registry Ltd", "TM", "TM", "Turkmenistan"),
    ("tn", "Agence Tunisienne d'Internet", "TN", "TN", "Tunisia"),
    (
        "to",
        "Government of the Kingdom of Tonga",
        "TO",
        "TO",
        "Tonga",
    ),
    (
        "tr",
        "Bilgi Teknolojileri ve İletişim Kurumu (BTK)",
        "TR",
        "TR",
        "Türkiye",
    ),
    (
        "tt",
        "University of the West Indies",
        "TT",
        "TT",
        "Trinidad and Tobago",
    ),
    (
        "tv",
        "Ministry of Transport, Energy, Communications and Innovations",
        "TV",
        "TV",
        "Tuvalu",
    ),
    (
        "tw",
        "Taiwan Network Information Center (TWNIC)",
        "TW",
        "TW",
        "Taiwan, Province of China",
    ),
    (
        "tz",
        "Tanzania Communications Regulatory Authority",
        "TZ",
        "TZ",
        "Tanzania, United Republic of",
    ),
    // U
    ("ua", "Hostmaster Ltd.", "UA", "UA", "Ukraine"),
    ("ug", "Uganda Online Ltd.", "UG", "UG", "Uganda"),
    (
        "uy",
        "SeCIU - Universidad de la República",
        "UY",
        "UY",
        "Uruguay",
    ),
    (
        "uz",
        "Uzbekistan National Internet Council",
        "UZ",
        "UZ",
        "Uzbekistan",
    ),
    // V
    (
        "va",
        "Holy See (Vatican City State)",
        "VA",
        "VA",
        "Holy See",
    ),
    (
        "vc",
        "Government of Saint Vincent and the Grenadines",
        "VC",
        "VC",
        "Saint Vincent and the Grenadines",
    ),
    (
        "ve",
        "CONATEL",
        "VE",
        "VE",
        "Venezuela, Bolivarian Republic of",
    ),
    (
        "vg",
        "Telecommunications Regulatory Commission (TRC)",
        "VG",
        "VG",
        "Virgin Islands, British",
    ),
    (
        "vi",
        "Virgin Islands Public Telecommunications System (VIPITS)",
        "VI",
        "VI",
        "Virgin Islands, U.S.",
    ),
    (
        "vn",
        "Vietnam Internet Network Information Center (VNNIC)",
        "VN",
        "VN",
        "Viet Nam",
    ),
    (
        "vu",
        "Telecommunications, Radiocommunications and Broadcasting Regulator (TRBR)",
        "VU",
        "VU",
        "Vanuatu",
    ),
    // W
    ("wf", "AFNIC", "FR", "WF", "Wallis and Futuna"),
    ("ws", "Government of Samoa", "WS", "WS", "Samoa"),
    // Y
    ("ye", "TeleYemen", "YE", "YE", "Yemen"),
    ("yt", "AFNIC", "FR", "YT", "Mayotte"),
    // Z
    (
        "zm",
        "Zambia Information and Communications Technology Authority (ZICTA)",
        "ZM",
        "ZM",
        "Zambia",
    ),
    (
        "zw",
        "Postal and Telecommunications Regulatory Authority of Zimbabwe (POTRAZ)",
        "ZW",
        "ZW",
        "Zimbabwe",
    ),
];

fn seed_remaining_cctlds(db: &TomeDb) -> Result<()> {
    for &(tld, operator, op_hq, alpha2, country_name) in REMAINING_CCTLDS {
        // Skip if already exists (from initial seed)
        if db.get_tld(tld)?.is_some() {
            continue;
        }

        let desc = format!("Country-code top-level domain for {country_name}");
        let tld_id = insert_tld_with_operator(db, tld, "ccTLD", &desc, operator, Some(op_hq))?;

        db.insert_country_mapping(&CountryMappingRow {
            tld_id,
            iso_3166_alpha2: alpha2.to_string(),
            iso_3166_alpha3: None,
            iso_3166_numeric: None,
            country_name: country_name.to_string(),
            sovereign_state: None,
            local_presence_required: false,
            local_presence_notes: None,
        })?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Popular nTLDs
// ---------------------------------------------------------------------------

/// (tld, operator_name, description)
///
/// Only open or semi-restricted nTLDs that are commonly registered.
/// Operators sourced from IANA Root Zone Database.
const POPULAR_NTLDS: &[(&str, &str, &str)] = &[
    // General purpose
    (
        "site",
        "Radix Technologies Inc.",
        "Websites and online presence",
    ),
    ("tech", "Radix Technologies Inc.", "Technology sector"),
    (
        "store",
        "Radix Technologies Inc.",
        "Online retail and e-commerce",
    ),
    ("fun", "Radix Technologies Inc.", "Entertainment and fun"),
    (
        "host",
        "Radix Technologies Inc.",
        "Hosting and internet services",
    ),
    (
        "space",
        "Radix Technologies Inc.",
        "Creative and open-ended use",
    ),
    ("press", "Radix Technologies Inc.", "News and journalism"),
    ("website", "Radix Technologies Inc.", "Websites"),
    ("shop", "GMO Registry, Inc.", "Shopping and retail"),
    (
        "top",
        "Hong Kong Zhongze International Limited",
        "General purpose",
    ),
    ("icu", "Shortdot SA", "General purpose (I see you)"),
    ("bond", "Shortdot SA", "Finance, community bonds"),
    ("cyou", "Shortdot SA", "Youth-focused general purpose"),
    ("cfd", "Shortdot SA", "Finance and trading"),
    ("club", "Registry Services, LLC", "Clubs and communities"),
    ("vip", "Registry Services, LLC", "Premium / VIP services"),
    (
        "ink",
        "Registry Services, LLC",
        "Tattoos, writing, creative expression",
    ),
    (
        "wiki",
        "Registry Services, LLC",
        "Wikis and knowledge sharing",
    ),
    ("design", "Registry Services, LLC", "Design professionals"),
    ("fashion", "Registry Services, LLC", "Fashion industry"),
    (
        "luxury",
        "Luxury Partners LLC",
        "Luxury brands and services",
    ),
    ("health", "Registry Services, LLC", "Health and wellness"),
    ("law", "Registry Services, LLC", "Legal professionals"),
    ("gay", "Registry Services, LLC", "LGBTQ+ community"),
    ("boston", "Registry Services, LLC", "City of Boston"),
    (
        "blog",
        "Knock Knock WHOIS There, LLC",
        "Blogs and personal publishing",
    ),
    ("cloud", "ARUBA PEC S.p.A.", "Cloud computing and services"),
    ("art", "UK Creative Ideas Limited", "Art and artists"),
    ("eco", "Big Room Inc.", "Ecology and sustainability"),
    (
        "page",
        "Charleston Road Registry Inc.",
        "Web pages (Google)",
    ),
    (
        "how",
        "Charleston Road Registry Inc.",
        "How-to content (Google)",
    ),
    (
        "soy",
        "Charleston Road Registry Inc.",
        "General purpose (Google)",
    ),
    (
        "new",
        "Charleston Road Registry Inc.",
        "New initiatives (Google)",
    ),
    (
        "day",
        "Charleston Road Registry Inc.",
        "Events and daily content (Google)",
    ),
    (
        "ads",
        "Charleston Road Registry Inc.",
        "Advertising (Google)",
    ),
    (
        "map",
        "Charleston Road Registry Inc.",
        "Maps and location (Google)",
    ),
    (
        "earth",
        "Interlink Systems Innovation Institute K.K.",
        "Earth and environmental",
    ),
    (
        "global",
        "Identity Digital Limited",
        "Global businesses and organizations",
    ),
    (
        "llc",
        "Identity Digital Limited",
        "Limited liability companies",
    ),
    (
        "green",
        "Identity Digital Limited",
        "Environmental and green businesses",
    ),
    ("bio", "Identity Digital Limited", "Biology and biographies"),
    ("pro", "Identity Digital Limited", "Professionals"),
    ("lotto", "Identity Digital Limited", "Lottery"),
    // Geographic
    ("london", "Dot London Domains Limited", "City of London"),
    ("berlin", "dotBERLIN GmbH & Co. KG", "City of Berlin"),
    ("amsterdam", "Gemeente Amsterdam", "City of Amsterdam"),
    ("paris", "City of Paris", "City of Paris"),
    ("nyc", "The City of New York", "New York City"),
    ("tokyo", "GMO Registry, Inc.", "City of Tokyo"),
    ("vegas", "Dot Vegas, Inc.", "Las Vegas"),
    ("brussels", "DNS.be vzw", "City of Brussels"),
    ("quebec", "PointQuébec Inc.", "Province of Québec"),
    ("barcelona", "Municipi de Barcelona", "City of Barcelona"),
    (
        "istanbul",
        "Istanbul Metropolitan Municipality",
        "City of Istanbul",
    ),
    ("koeln", "dotKoeln GmbH", "City of Cologne (Köln)"),
    ("cologne", "dotKoeln GmbH", "City of Cologne"),
    (
        "hamburg",
        "Hamburg Top-Level-Domain GmbH",
        "City of Hamburg",
    ),
    ("wien", "punkt.wien GmbH", "City of Vienna (Wien)"),
    ("helsinki", "City of Helsinki", "City of Helsinki"),
    ("africa", "ZA Central Registry NPC", "African continent"),
    ("asia", "DotAsia Organisation Ltd.", "Asia-Pacific region"),
    ("lat", "XYZ.COM LLC", "Latin America and Latino community"),
    // Industry / vertical
    (
        "bank",
        "fTLD Registry Services, LLC",
        "Verified banking institutions",
    ),
    (
        "insurance",
        "fTLD Registry Services, LLC",
        "Verified insurance companies",
    ),
    ("music", "DotMusic Limited", "Music industry"),
    (
        "film",
        "Motion Picture Domain Registry Pty Ltd",
        "Film industry",
    ),
    ("game", "XYZ.COM LLC", "Gaming"),
    ("games", "Dog Beach, LLC", "Games and gaming"),
    ("homes", "XYZ.COM LLC", "Real estate and homes"),
    ("cars", "XYZ.COM LLC", "Automotive"),
    ("auto", "XYZ.COM LLC", "Automotive"),
    ("baby", "XYZ.COM LLC", "Baby products and parenting"),
    ("beauty", "XYZ.COM LLC", "Beauty products and services"),
    ("college", "XYZ.COM LLC", "Higher education"),
    ("lol", "XYZ.COM LLC", "Humor and entertainment"),
    ("mom", "XYZ.COM LLC", "Motherhood and parenting"),
    ("monster", "XYZ.COM LLC", "Entertainment and creative"),
    ("bible", "American Bible Society", "Religious content"),
    (
        "catholic",
        "Pontifical Council for Social Communication",
        "Catholic Church",
    ),
    (
        "charity",
        "Public Interest Registry (PIR)",
        "Charitable organizations",
    ),
    (
        "foundation",
        "Public Interest Registry (PIR)",
        "Foundations and non-profits",
    ),
    (
        "gives",
        "Public Interest Registry (PIR)",
        "Giving and philanthropy",
    ),
    // Business / professional
    ("agency", "Binky Moon, LLC", "Agencies"),
    ("business", "Binky Moon, LLC", "Businesses"),
    ("company", "Binky Moon, LLC", "Companies"),
    ("consulting", "Dog Beach, LLC", "Consulting firms"),
    ("digital", "Binky Moon, LLC", "Digital businesses"),
    ("domains", "Binky Moon, LLC", "Domain industry"),
    ("email", "Binky Moon, LLC", "Email services"),
    ("enterprises", "Binky Moon, LLC", "Enterprises"),
    ("finance", "Binky Moon, LLC", "Finance industry"),
    ("group", "Binky Moon, LLC", "Business groups"),
    ("holdings", "Binky Moon, LLC", "Holding companies"),
    (
        "international",
        "Binky Moon, LLC",
        "International organizations",
    ),
    ("investments", "Binky Moon, LLC", "Investment firms"),
    ("limited", "Binky Moon, LLC", "Limited companies"),
    ("ltd", "Binky Moon, LLC", "Limited companies"),
    ("management", "Binky Moon, LLC", "Management firms"),
    ("marketing", "Binky Moon, LLC", "Marketing agencies"),
    ("media", "Binky Moon, LLC", "Media companies"),
    ("network", "Binky Moon, LLC", "Networks and networking"),
    ("partners", "Binky Moon, LLC", "Business partnerships"),
    ("services", "Binky Moon, LLC", "Service providers"),
    ("solutions", "Binky Moon, LLC", "Solution providers"),
    ("studio", "Dog Beach, LLC", "Studios and creative spaces"),
    ("systems", "Binky Moon, LLC", "Technology systems"),
    ("team", "Binky Moon, LLC", "Teams and collaboration"),
    ("technology", "Binky Moon, LLC", "Technology sector"),
    (
        "ventures",
        "Binky Moon, LLC",
        "Venture capital and startups",
    ),
    ("world", "Binky Moon, LLC", "General purpose global scope"),
    ("zone", "Binky Moon, LLC", "Zones and communities"),
    // Lifestyle / interest
    ("life", "Binky Moon, LLC", "Life and lifestyle"),
    ("live", "Dog Beach, LLC", "Live streaming and events"),
    ("love", "Waterford Limited", "Love and relationships"),
    ("family", "Dog Beach, LLC", "Family-oriented content"),
    ("community", "Binky Moon, LLC", "Community organizations"),
    ("education", "Binky Moon, LLC", "Education sector"),
    ("travel", "Dog Beach, LLC", "Travel industry"),
    ("yoga", "Registry Services, LLC", "Yoga and wellness"),
    ("fitness", "Binky Moon, LLC", "Fitness and health"),
    ("organic", "Identity Digital Limited", "Organic products"),
    // Food / hospitality
    ("restaurant", "Binky Moon, LLC", "Restaurants"),
    ("cafe", "Binky Moon, LLC", "Cafés and coffee"),
    (
        "bar",
        "Punto 2012 Sociedad Anonima Promotora de Inversion de Capital Variable",
        "Bars and nightlife",
    ),
    ("pizza", "Binky Moon, LLC", "Pizza establishments"),
    ("wine", "Binky Moon, LLC", "Wine industry"),
    ("beer", "Registry Services, LLC", "Beer and brewing"),
    // Real estate / property
    ("house", "Binky Moon, LLC", "Houses and real estate"),
    ("apartments", "Binky Moon, LLC", "Apartments and rentals"),
    ("land", "Binky Moon, LLC", "Land and property"),
];

fn seed_popular_ntlds(db: &TomeDb) -> Result<()> {
    for &(tld, operator, purpose) in POPULAR_NTLDS {
        // Skip if already exists (from initial seed)
        if db.get_tld(tld)?.is_some() {
            continue;
        }

        let desc = format!("New generic top-level domain for {purpose}");
        insert_tld_with_operator(db, tld, "nTLD", &desc, operator, None)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Brand / single-registrant nTLDs
// ---------------------------------------------------------------------------

/// (tld, operator_name, description)
///
/// Single-registrant / brand TLDs — not open for general registration.
/// Operators sourced from IANA Root Zone Database.
const BRAND_NTLDS: &[(&str, &str, &str)] = &[
    (
        "aaa",
        "American Automobile Association, Inc.",
        "Brand TLD for AAA",
    ),
    ("aarp", "AARP", "Brand TLD for AARP"),
    ("abb", "ABB Ltd", "Brand TLD for ABB"),
    (
        "abbott",
        "Abbott Laboratories, Inc.",
        "Brand TLD for Abbott",
    ),
    ("abbvie", "AbbVie Inc.", "Brand TLD for AbbVie"),
    (
        "abc",
        "Disney Enterprises, Inc.",
        "Brand TLD for ABC/Disney",
    ),
    ("accenture", "Accenture plc", "Brand TLD for Accenture"),
    (
        "aco",
        "ACO Severin Ahlmann GmbH & Co. KG",
        "Brand TLD for ACO",
    ),
    (
        "aeg",
        "Aktiebolaget Electrolux",
        "Brand TLD for AEG/Electrolux",
    ),
    (
        "aetna",
        "Aetna Life Insurance Company",
        "Brand TLD for Aetna",
    ),
    ("afl", "Australian Football League", "Brand TLD for AFL"),
    (
        "agakhan",
        "Fondation Aga Khan (Aga Khan Foundation)",
        "Brand TLD for Aga Khan Foundation",
    ),
    (
        "aig",
        "American International Group, Inc.",
        "Brand TLD for AIG",
    ),
    ("airbus", "Airbus S.A.S.", "Brand TLD for Airbus"),
    ("airtel", "Bharti Airtel Limited", "Brand TLD for Airtel"),
    (
        "akdn",
        "Fondation Aga Khan (Aga Khan Foundation)",
        "Brand TLD for Aga Khan Development Network",
    ),
    (
        "alibaba",
        "Alibaba Group Holding Limited",
        "Brand TLD for Alibaba",
    ),
    (
        "alipay",
        "Alibaba Group Holding Limited",
        "Brand TLD for Alipay",
    ),
    (
        "allstate",
        "Allstate Fire and Casualty Insurance Company",
        "Brand TLD for Allstate",
    ),
    ("ally", "Ally Financial Inc.", "Brand TLD for Ally"),
    ("alstom", "ALSTOM", "Brand TLD for Alstom"),
    (
        "amazon",
        "Amazon Registry Services, Inc.",
        "Brand TLD for Amazon",
    ),
    (
        "americanexpress",
        "American Express Travel Related Services Company, Inc.",
        "Brand TLD for American Express",
    ),
    (
        "amex",
        "American Express Travel Related Services Company, Inc.",
        "Brand TLD for Amex",
    ),
    (
        "amica",
        "Amica Mutual Insurance Company",
        "Brand TLD for Amica",
    ),
    (
        "android",
        "Charleston Road Registry Inc.",
        "Brand TLD for Android/Google",
    ),
    (
        "anz",
        "Australia and New Zealand Banking Group Limited",
        "Brand TLD for ANZ Bank",
    ),
    ("aol", "Yahoo Inc.", "Brand TLD for AOL"),
    ("apple", "Apple Inc.", "Brand TLD for Apple"),
    ("aramco", "Aramco Services Company", "Brand TLD for Aramco"),
    ("audi", "AUDI Aktiengesellschaft", "Brand TLD for Audi"),
    (
        "audible",
        "Amazon Registry Services, Inc.",
        "Brand TLD for Audible",
    ),
    (
        "auspost",
        "Australian Postal Corporation",
        "Brand TLD for Australia Post",
    ),
    ("aws", "AWS Registry LLC", "Brand TLD for AWS"),
    ("axa", "AXA Group Operations SAS", "Brand TLD for AXA"),
    (
        "azure",
        "Microsoft Corporation",
        "Brand TLD for Microsoft Azure",
    ),
    ("baidu", "Baidu, Inc.", "Brand TLD for Baidu"),
    (
        "barclaycard",
        "Barclays Bank PLC",
        "Brand TLD for Barclaycard",
    ),
    ("barclays", "Barclays Bank PLC", "Brand TLD for Barclays"),
    (
        "bbc",
        "British Broadcasting Corporation",
        "Brand TLD for BBC",
    ),
    (
        "bbva",
        "BANCO BILBAO VIZCAYA ARGENTARIA, S.A.",
        "Brand TLD for BBVA",
    ),
    (
        "bcg",
        "The Boston Consulting Group, Inc.",
        "Brand TLD for BCG",
    ),
    ("bentley", "Bentley Motors Limited", "Brand TLD for Bentley"),
    (
        "bing",
        "Microsoft Corporation",
        "Brand TLD for Microsoft Bing",
    ),
    (
        "bloomberg",
        "Bloomberg IP Holdings LLC",
        "Brand TLD for Bloomberg",
    ),
    (
        "bmw",
        "Bayerische Motoren Werke Aktiengesellschaft",
        "Brand TLD for BMW",
    ),
    ("bnpparibas", "BNP Paribas", "Brand TLD for BNP Paribas"),
    (
        "boehringer",
        "Boehringer Ingelheim International GmbH",
        "Brand TLD for Boehringer Ingelheim",
    ),
    (
        "bofa",
        "Bank of America Corporation",
        "Brand TLD for Bank of America",
    ),
    ("booking", "Booking.com B.V.", "Brand TLD for Booking.com"),
    ("bosch", "Robert Bosch GMBH", "Brand TLD for Bosch"),
    (
        "bridgestone",
        "Bridgestone Corporation",
        "Brand TLD for Bridgestone",
    ),
    (
        "brother",
        "Brother Industries, Ltd.",
        "Brand TLD for Brother",
    ),
    ("canon", "Canon Inc.", "Brand TLD for Canon"),
    (
        "capitalone",
        "Capital One Financial Corporation",
        "Brand TLD for Capital One",
    ),
    (
        "cba",
        "COMMONWEALTH BANK OF AUSTRALIA",
        "Brand TLD for Commonwealth Bank",
    ),
    (
        "cbn",
        "The Christian Broadcasting Network, Inc.",
        "Brand TLD for CBN",
    ),
    ("cbre", "CBRE, Inc.", "Brand TLD for CBRE"),
    (
        "cern",
        "European Organization for Nuclear Research (CERN)",
        "Brand TLD for CERN",
    ),
    (
        "chanel",
        "Chanel International B.V.",
        "Brand TLD for Chanel",
    ),
    (
        "chase",
        "JPMorgan Chase Bank, National Association",
        "Brand TLD for Chase",
    ),
    (
        "chrome",
        "Charleston Road Registry Inc.",
        "Brand TLD for Google Chrome",
    ),
    ("cisco", "Cisco Technology, Inc.", "Brand TLD for Cisco"),
    ("citadel", "Citadel Domain LLC", "Brand TLD for Citadel"),
    ("citi", "Citigroup Inc.", "Brand TLD for Citi"),
    ("citic", "CITIC Group Corporation", "Brand TLD for CITIC"),
    (
        "datsun",
        "NISSAN MOTOR CO., LTD.",
        "Brand TLD for Datsun/Nissan",
    ),
    ("dell", "Dell Inc.", "Brand TLD for Dell"),
    (
        "deloitte",
        "Deloitte Touche Tohmatsu",
        "Brand TLD for Deloitte",
    ),
    (
        "delta",
        "Delta Air Lines, Inc.",
        "Brand TLD for Delta Air Lines",
    ),
    ("dhl", "Deutsche Post AG", "Brand TLD for DHL"),
    (
        "discover",
        "Discover Financial Services",
        "Brand TLD for Discover",
    ),
    ("dnp", "Dai Nippon Printing Co., Ltd.", "Brand TLD for DNP"),
    (
        "dupont",
        "DuPont Specialty Products USA, LLC",
        "Brand TLD for DuPont",
    ),
    (
        "dvag",
        "Deutsche Vermögensberatung Aktiengesellschaft DVAG",
        "Brand TLD for DVAG",
    ),
    ("epson", "Seiko Epson Corporation", "Brand TLD for Epson"),
    (
        "ericsson",
        "Telefonaktiebolaget L M Ericsson",
        "Brand TLD for Ericsson",
    ),
    (
        "fedex",
        "Federal Express Corporation",
        "Brand TLD for FedEx",
    ),
    (
        "ferrari",
        "Fiat Chrysler Automobiles N.V.",
        "Brand TLD for Ferrari",
    ),
    (
        "ferrero",
        "Ferrero Trading Lux S.A.",
        "Brand TLD for Ferrero",
    ),
    (
        "fidelity",
        "Fidelity Brokerage Services LLC",
        "Brand TLD for Fidelity",
    ),
    ("flickr", "Flickr, Inc.", "Brand TLD for Flickr"),
    ("ford", "Ford Motor Company", "Brand TLD for Ford"),
    ("fox", "FOX Registry, LLC", "Brand TLD for Fox"),
    ("fujitsu", "Fujitsu Limited", "Brand TLD for Fujitsu"),
    ("gallo", "Gallo Vineyards, Inc.", "Brand TLD for Gallo"),
    (
        "gmail",
        "Charleston Road Registry Inc.",
        "Brand TLD for Gmail/Google",
    ),
    ("godaddy", "Go Daddy East, LLC", "Brand TLD for GoDaddy"),
    (
        "google",
        "Charleston Road Registry Inc.",
        "Brand TLD for Google",
    ),
    (
        "grainger",
        "Grainger Registry Services, LLC",
        "Brand TLD for Grainger",
    ),
    ("gucci", "Guccio Gucci S.p.a.", "Brand TLD for Gucci"),
    (
        "hdfc",
        "HOUSING DEVELOPMENT FINANCE CORPORATION LIMITED",
        "Brand TLD for HDFC",
    ),
    ("hdfcbank", "HDFC Bank Limited", "Brand TLD for HDFC Bank"),
    ("hermes", "Hermes International", "Brand TLD for Hermès"),
    ("hitachi", "Hitachi, Ltd.", "Brand TLD for Hitachi"),
    ("honda", "Honda Motor Co., Ltd.", "Brand TLD for Honda"),
    (
        "hotmail",
        "Microsoft Corporation",
        "Brand TLD for Hotmail/Microsoft",
    ),
    (
        "hsbc",
        "HSBC Global Services (UK) Limited",
        "Brand TLD for HSBC",
    ),
    ("hyatt", "Hyatt GTLD, L.L.C.", "Brand TLD for Hyatt"),
    ("hyundai", "Hyundai Motor Company", "Brand TLD for Hyundai"),
    (
        "ibm",
        "International Business Machines Corporation",
        "Brand TLD for IBM",
    ),
    (
        "icbc",
        "Industrial and Commercial Bank of China Limited",
        "Brand TLD for ICBC",
    ),
    ("ieee", "IEEE Global LLC", "Brand TLD for IEEE"),
    (
        "infiniti",
        "NISSAN MOTOR CO., LTD.",
        "Brand TLD for Infiniti/Nissan",
    ),
    (
        "intuit",
        "Intuit Administrative Services, Inc.",
        "Brand TLD for Intuit",
    ),
    ("jaguar", "Jaguar Land Rover Ltd", "Brand TLD for Jaguar"),
    ("java", "Oracle Corporation", "Brand TLD for Java/Oracle"),
    ("jcb", "JCB Co., Ltd.", "Brand TLD for JCB"),
    (
        "jpmorgan",
        "JPMorgan Chase Bank, National Association",
        "Brand TLD for JPMorgan",
    ),
    (
        "juniper",
        "JUNIPER NETWORKS, INC.",
        "Brand TLD for Juniper Networks",
    ),
    ("kddi", "KDDI CORPORATION", "Brand TLD for KDDI"),
    ("kia", "KIA MOTORS CORPORATION", "Brand TLD for Kia"),
    ("komatsu", "Komatsu Ltd.", "Brand TLD for Komatsu"),
    (
        "kpmg",
        "KPMG International Cooperative",
        "Brand TLD for KPMG",
    ),
    ("kpn", "Koninklijke KPN N.V.", "Brand TLD for KPN"),
    (
        "lamborghini",
        "Automobili Lamborghini S.p.A.",
        "Brand TLD for Lamborghini",
    ),
    (
        "landrover",
        "Jaguar Land Rover Ltd",
        "Brand TLD for Land Rover",
    ),
    ("lanxess", "LANXESS Corporation", "Brand TLD for LANXESS"),
    ("lego", "LEGO Juris A/S", "Brand TLD for LEGO"),
    (
        "lexus",
        "TOYOTA MOTOR CORPORATION",
        "Brand TLD for Lexus/Toyota",
    ),
    (
        "lidl",
        "Schwarz Domains und Services GmbH & Co. KG",
        "Brand TLD for Lidl",
    ),
    (
        "lincoln",
        "Ford Motor Company",
        "Brand TLD for Lincoln/Ford",
    ),
    ("lilly", "Eli Lilly and Company", "Brand TLD for Eli Lilly"),
    (
        "microsoft",
        "Microsoft Corporation",
        "Brand TLD for Microsoft",
    ),
    ("netflix", "Netflix, Inc.", "Brand TLD for Netflix"),
    ("nike", "NIKE, Inc.", "Brand TLD for Nike"),
    ("nissan", "NISSAN MOTOR CO., LTD.", "Brand TLD for Nissan"),
    ("nokia", "Nokia Corporation", "Brand TLD for Nokia"),
    ("oracle", "Oracle Corporation", "Brand TLD for Oracle"),
    (
        "panasonic",
        "Panasonic Holdings Corporation",
        "Brand TLD for Panasonic",
    ),
    (
        "philips",
        "Koninklijke Philips N.V.",
        "Brand TLD for Philips",
    ),
    ("piaget", "Richemont DNS Inc.", "Brand TLD for Piaget"),
    (
        "playstation",
        "Sony Interactive Entertainment Inc.",
        "Brand TLD for PlayStation",
    ),
    (
        "porsche",
        "Dr. Ing. h.c. F. Porsche AG",
        "Brand TLD for Porsche",
    ),
    ("pwc", "PricewaterhouseCoopers LLP", "Brand TLD for PwC"),
    ("ricoh", "Ricoh Company, Ltd.", "Brand TLD for Ricoh"),
    ("rolex", "Rolex SA", "Brand TLD for Rolex"),
    ("samsung", "SAMSUNG SDS CO., LTD", "Brand TLD for Samsung"),
    ("sap", "SAP AG", "Brand TLD for SAP"),
    (
        "schwarz",
        "Schwarz Domains und Services GmbH & Co. KG",
        "Brand TLD for Schwarz Group",
    ),
    ("sharp", "Sharp Corporation", "Brand TLD for Sharp"),
    (
        "shell",
        "Shell Information Technology International Inc",
        "Brand TLD for Shell",
    ),
    (
        "siemens",
        "Siemens Aktiengesellschaft",
        "Brand TLD for Siemens",
    ),
    ("sky", "Sky International AG", "Brand TLD for Sky"),
    (
        "skype",
        "Microsoft Corporation",
        "Brand TLD for Skype/Microsoft",
    ),
    (
        "smart",
        "Smart Communications, Inc. (PLDT)",
        "Brand TLD for Smart",
    ),
    ("sony", "Sony Corporation", "Brand TLD for Sony"),
    ("stada", "STADA Arzneimittel AG", "Brand TLD for STADA"),
    ("suzuki", "SUZUKI MOTOR CORPORATION", "Brand TLD for Suzuki"),
    ("swatch", "The Swatch Group Ltd", "Brand TLD for Swatch"),
    ("tdk", "TDK Corporation", "Brand TLD for TDK"),
    (
        "teva",
        "Teva Pharmaceutical Industries Limited",
        "Brand TLD for Teva",
    ),
    ("toyota", "TOYOTA MOTOR CORPORATION", "Brand TLD for Toyota"),
    ("toshiba", "TOSHIBA Corporation", "Brand TLD for Toshiba"),
    ("ubs", "UBS AG", "Brand TLD for UBS"),
    ("ups", "UPS Market Driver, Inc.", "Brand TLD for UPS"),
    ("verisign", "VeriSign, Inc.", "Brand TLD for VeriSign"),
    ("visa", "Visa Incorporated", "Brand TLD for Visa"),
    (
        "volkswagen",
        "Volkswagen Group of America Inc.",
        "Brand TLD for Volkswagen",
    ),
    (
        "volvo",
        "Volvo Holding Sverige Aktiebolag",
        "Brand TLD for Volvo",
    ),
    ("walmart", "Wal-Mart Stores, Inc.", "Brand TLD for Walmart"),
    (
        "windows",
        "Microsoft Corporation",
        "Brand TLD for Microsoft Windows",
    ),
    (
        "xbox",
        "Microsoft Corporation",
        "Brand TLD for Xbox/Microsoft",
    ),
    ("xerox", "Xerox PARC Inc.", "Brand TLD for Xerox"),
    ("yahoo", "Yahoo Inc.", "Brand TLD for Yahoo"),
    ("yamaha", "Yamaha Motor Co., Ltd.", "Brand TLD for Yamaha"),
    (
        "youtube",
        "Charleston Road Registry Inc.",
        "Brand TLD for YouTube/Google",
    ),
];

fn seed_brand_ntlds(db: &TomeDb) -> Result<()> {
    for &(tld, operator, purpose) in BRAND_NTLDS {
        if db.get_tld(tld)?.is_some() {
            continue;
        }
        insert_tld_with_operator(db, tld, "nTLD", purpose, operator, None)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::seed;

    #[test]
    fn test_seed_extended_after_seed() {
        let db = TomeDb::open_in_memory().expect("open db");
        seed(&db).expect("initial seed");

        let count_before = db.count_tlds().expect("count");
        seed_extended(&db).expect("extended seed");
        let count_after = db.count_tlds().expect("count");

        // Should have added a substantial number
        let added = count_after - count_before;
        assert!(added >= 350, "expected at least 350 new TLDs, got {added}");

        // Total should be 300+
        assert!(
            count_after >= 300,
            "expected at least 300 total TLDs, got {count_after}"
        );
    }

    #[test]
    fn test_extended_cctlds_have_country_mapping() {
        let db = TomeDb::open_in_memory().expect("open db");
        seed(&db).expect("initial seed");
        seed_extended(&db).expect("extended seed");

        // Spot-check some ccTLDs that were NOT in the initial seed
        for tld in ["gr", "hu", "ro", "th", "tw", "vn", "ke", "gh", "qa"] {
            let ov = db
                .get_tld_overview(tld)
                .expect("overview")
                .unwrap_or_else(|| panic!(".{tld} should exist in overview"));
            assert_eq!(ov.tld_type, "ccTLD", ".{tld} should be ccTLD");
            assert!(
                ov.country_name.is_some(),
                ".{tld} should have a country name"
            );
        }
    }

    #[test]
    fn test_extended_ntlds_present() {
        let db = TomeDb::open_in_memory().expect("open db");
        seed(&db).expect("initial seed");
        seed_extended(&db).expect("extended seed");

        // Spot-check some nTLDs
        for tld in [
            "blog", "cloud", "london", "berlin", "bank", "agency", "studio",
        ] {
            let found = db
                .get_tld(tld)
                .expect("lookup")
                .unwrap_or_else(|| panic!(".{tld} should exist"));
            assert_eq!(found.tld_type, "nTLD", ".{tld} should be nTLD");
        }
    }

    #[test]
    fn test_extended_skips_duplicates() {
        let db = TomeDb::open_in_memory().expect("open db");
        seed(&db).expect("initial seed");

        let count_initial = db.count_tlds().expect("count");
        seed_extended(&db).expect("extended seed first time");
        let count_extended = db.count_tlds().expect("count");

        // Running initial seed entries shouldn't have been duplicated
        // Verify .com is still just one entry
        let coms = db.search_tlds("com").expect("search");
        let com_exact: Vec<_> = coms.iter().filter(|t| t.tld == "com").collect();
        assert_eq!(com_exact.len(), 1, ".com should appear exactly once");

        // The extended count should be strictly greater
        assert!(count_extended > count_initial);
    }

    #[test]
    fn test_total_coverage() {
        let db = TomeDb::open_in_memory().expect("open db");
        seed(&db).expect("initial seed");
        seed_extended(&db).expect("extended seed");

        let all = db.list_tlds().expect("list all");
        let gtlds: Vec<_> = all.iter().filter(|t| t.tld_type == "gTLD").collect();
        let cctlds: Vec<_> = all.iter().filter(|t| t.tld_type == "ccTLD").collect();
        let ntlds: Vec<_> = all.iter().filter(|t| t.tld_type == "nTLD").collect();

        // We should have comprehensive ccTLD coverage
        assert!(
            cctlds.len() >= 200,
            "expected 200+ ccTLD entries (incl SLDs), got {}",
            cctlds.len()
        );

        // And good nTLD coverage
        assert!(
            ntlds.len() >= 200,
            "expected 200+ nTLD entries, got {}",
            ntlds.len()
        );

        // gTLDs from initial seed
        assert!(
            gtlds.len() >= 13,
            "expected 13+ gTLD entries, got {}",
            gtlds.len()
        );
    }
}
