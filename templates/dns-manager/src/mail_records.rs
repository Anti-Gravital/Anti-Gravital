//! SPF/DKIM/DMARC record presets generated through `ag-domains`.

use ag_domains::mail_records::{generate_mail_records, DmarcPolicy, MailDnsRequirements};
use serde_json::{json, Value};

/// Generates the standard mail-authentication records for `MAIL_DOMAIN`.
///
/// No DKIM selector is included because no public key is bundled with the
/// scaffold; add one in a real deployment through `DkimSelector`.
pub fn presets() -> Value {
    let domain = std::env::var("MAIL_DOMAIN").unwrap_or_else(|_| "example.com".into());
    let requirements = MailDnsRequirements {
        domain: domain.clone(),
        dkim_selectors: Vec::new(),
        spf_includes: Vec::new(),
        spf_ips: Vec::new(),
        dmarc_policy: DmarcPolicy::None,
        dmarc_rua: Some("dmarc@example.com".into()),
    };
    let records = generate_mail_records(&requirements);
    json!({ "domain": domain, "records": records })
}
