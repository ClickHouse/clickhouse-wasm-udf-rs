use std::collections::HashMap;

use x509_parser::parse_x509_certificate;
use x509_parser::pem::parse_x509_pem;

use clickhouse_wasm_udf::clickhouse_udf;

#[clickhouse_udf]
pub fn parse_pem(data: String) -> anyhow::Result<HashMap<String, String>> {
    let (_, pem) = parse_x509_pem(data.as_bytes())?;
    let (_, cert) = parse_x509_certificate(&pem.contents)?;
    let tbs = &cert.tbs_certificate;

    Ok(HashMap::from([
        ("version".into(), tbs.version.0.to_string()),
        ("issuer".into(), tbs.issuer.to_string()),
        ("subject".into(), tbs.subject.to_string()),
        ("not_before".into(), tbs.validity.not_before.timestamp().to_string()),
        ("not_after".into(), tbs.validity.not_after.timestamp().to_string()),
    ]))
}
