//! DNS diagnostics: expected vs observed records.
//!
//! Diagnostics compare the records `ag-domains` expects (from the instruction
//! engine) against what is actually observed in DNS, and emit actionable
//! findings (RFC-0011 section 5.4; blueprint section 16.3). The comparison
//! logic is pure; callers supply observed records from a resolver or adapter.

use crate::instructions::DnsInstructions;
use crate::record::RecordType;

/// An observed DNS record (as seen at a resolver/provider).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedRecord {
    /// Record type.
    pub record_type: RecordType,
    /// Fully-qualified record name.
    pub name: String,
    /// Observed value.
    pub value: String,
}

/// Severity of a diagnostic finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Everything matches.
    Ok,
    /// Something is wrong and must be fixed.
    Error,
}

/// A single diagnostic finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Severity.
    pub severity: Severity,
    /// Short description.
    pub message: String,
    /// Suggested action (empty when `Ok`).
    pub action: String,
}

/// The result of diagnosing one attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnosis {
    /// Per-check findings.
    pub findings: Vec<Finding>,
}

impl Diagnosis {
    /// `true` when no finding has `Error` severity.
    pub fn is_healthy(&self) -> bool {
        self.findings.iter().all(|f| f.severity == Severity::Ok)
    }
}

/// Compares expected instructions against observed records.
///
/// For each expected record, looks for a matching observed record by type and
/// name; reports a mismatch (wrong value) or absence. Detects an apex
/// CNAME/A coexistence conflict.
pub fn diagnose(instructions: &DnsInstructions, observed: &[ObservedRecord]) -> Diagnosis {
    let mut findings = Vec::new();

    for expected in instructions.all() {
        let same_name_type: Vec<&ObservedRecord> = observed
            .iter()
            .filter(|o| {
                o.record_type == expected.record_type && names_match(&o.name, &expected.name)
            })
            .collect();

        if same_name_type.is_empty() {
            findings.push(Finding {
                severity: Severity::Error,
                message: format!(
                    "missing {} record for {}",
                    expected.record_type, expected.name
                ),
                action: format!(
                    "add {} {} -> {}",
                    expected.record_type, expected.name, expected.value
                ),
            });
        } else if same_name_type
            .iter()
            .any(|o| values_match(&o.value, &expected.value))
        {
            findings.push(Finding {
                severity: Severity::Ok,
                message: format!(
                    "{} record for {} points correctly",
                    expected.record_type, expected.name
                ),
                action: String::new(),
            });
        } else {
            let observed_values: Vec<String> =
                same_name_type.iter().map(|o| o.value.clone()).collect();
            findings.push(Finding {
                severity: Severity::Error,
                message: format!(
                    "{} record for {} has wrong value (observed: {})",
                    expected.record_type,
                    expected.name,
                    observed_values.join(", ")
                ),
                action: format!("replace the value with {}", expected.value),
            });
        }
    }

    // Apex CNAME/A coexistence conflict.
    for o in observed {
        if o.record_type == RecordType::Cname {
            let conflict = observed.iter().any(|x| {
                names_match(&x.name, &o.name)
                    && matches!(x.record_type, RecordType::A | RecordType::Aaaa)
            });
            if conflict {
                findings.push(Finding {
                    severity: Severity::Error,
                    message: format!("CNAME at {} coexists with A/AAAA", o.name),
                    action: "remove either the CNAME or the A/AAAA records at this name".to_owned(),
                });
                break;
            }
        }
    }

    Diagnosis { findings }
}

fn names_match(a: &str, b: &str) -> bool {
    a.trim_end_matches('.')
        .eq_ignore_ascii_case(b.trim_end_matches('.'))
}

fn values_match(a: &str, b: &str) -> bool {
    a.trim_end_matches('.')
        .eq_ignore_ascii_case(b.trim_end_matches('.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hostname::Hostname;
    use crate::instructions::{generate_instructions, EdgeTargets};

    fn subdomain_instructions() -> DnsInstructions {
        let h = Hostname::parse("api.example.com").unwrap();
        let t = EdgeTargets::new("edge.example-cloud.net");
        generate_instructions(&h, &t, "ag-verification=tok")
    }

    #[test]
    fn healthy_when_all_match() {
        let ins = subdomain_instructions();
        let observed = vec![
            ObservedRecord {
                record_type: RecordType::Cname,
                name: "api.example.com".to_owned(),
                value: "edge.example-cloud.net.".to_owned(),
            },
            ObservedRecord {
                record_type: RecordType::Txt,
                name: "_ag-domain.example.com".to_owned(),
                value: "ag-verification=tok".to_owned(),
            },
        ];
        let d = diagnose(&ins, &observed);
        assert!(d.is_healthy(), "findings: {:?}", d.findings);
    }

    #[test]
    fn detects_wrong_cname_target() {
        let ins = subdomain_instructions();
        let observed = vec![
            ObservedRecord {
                record_type: RecordType::Cname,
                name: "api.example.com".to_owned(),
                value: "old-hosting.example.net".to_owned(),
            },
            ObservedRecord {
                record_type: RecordType::Txt,
                name: "_ag-domain.example.com".to_owned(),
                value: "ag-verification=tok".to_owned(),
            },
        ];
        let d = diagnose(&ins, &observed);
        assert!(!d.is_healthy());
        assert!(d.findings.iter().any(|f| f.message.contains("wrong value")));
    }

    #[test]
    fn detects_missing_ownership() {
        let ins = subdomain_instructions();
        let observed = vec![ObservedRecord {
            record_type: RecordType::Cname,
            name: "api.example.com".to_owned(),
            value: "edge.example-cloud.net".to_owned(),
        }];
        let d = diagnose(&ins, &observed);
        assert!(d.findings.iter().any(|f| f.message.contains("missing TXT")));
    }

    #[test]
    fn detects_cname_a_conflict() {
        let ins = subdomain_instructions();
        let observed = vec![
            ObservedRecord {
                record_type: RecordType::Cname,
                name: "api.example.com".to_owned(),
                value: "edge.example-cloud.net".to_owned(),
            },
            ObservedRecord {
                record_type: RecordType::A,
                name: "api.example.com".to_owned(),
                value: "203.0.113.1".to_owned(),
            },
            ObservedRecord {
                record_type: RecordType::Txt,
                name: "_ag-domain.example.com".to_owned(),
                value: "ag-verification=tok".to_owned(),
            },
        ];
        let d = diagnose(&ins, &observed);
        assert!(d.findings.iter().any(|f| f.message.contains("coexists")));
    }
}
