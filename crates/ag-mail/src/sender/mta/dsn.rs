//! Asynchronous bounce intake for the native MTA (Phase 4.6-B / DEBT-020).
//!
//! Some failures arrive later as their own email: a **DSN** (Delivery Status
//! Notification, RFC 3464) for asynchronous bounces, or an **ARF** feedback
//! report (RFC 5965) for spam complaints. This module parses those messages and
//! feeds the [`SuppressionList`]: a permanent DSN suppresses the recipient as a
//! hard bounce, a complaint suppresses it as a complaint.
//!
//! The field parsers are pure and unit-tested; [`process_dsn`] / [`process_arf`]
//! use `mail-parser` to locate the `message/delivery-status` and
//! `message/feedback-report` parts and apply the result.

use mail_parser::{MessageParser, MimeHeaders};

use super::suppress::{SuppressionList, SuppressionReason};

/// The `Action` field of a DSN per-recipient block (RFC 3464).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsnAction {
    /// Permanent failure: the recipient should be suppressed.
    Failed,
    /// Temporary delay; no suppression.
    Delayed,
    /// Delivered successfully.
    Delivered,
    /// Relayed to a system that does not issue DSNs.
    Relayed,
    /// Expanded into a mailing list.
    Expanded,
    /// Any other / unrecognized action.
    Other,
}

impl DsnAction {
    fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "failed" => DsnAction::Failed,
            "delayed" => DsnAction::Delayed,
            "delivered" => DsnAction::Delivered,
            "relayed" => DsnAction::Relayed,
            "expanded" => DsnAction::Expanded,
            _ => DsnAction::Other,
        }
    }
}

/// One per-recipient record from a DSN delivery-status part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DsnRecord {
    /// The recipient address (the part after `rfc822;`).
    pub recipient: String,
    /// The reported action.
    pub action: DsnAction,
    /// The enhanced status code (`X.Y.Z`), if present.
    pub status: Option<String>,
}

/// Splits a header-style block into `(name_lowercased, value)` pairs, joining
/// folded continuation lines (those starting with whitespace).
fn parse_fields(block: &str) -> Vec<(String, String)> {
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in block.lines() {
        if line.is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t']) {
            if let Some(last) = fields.last_mut() {
                last.1.push(' ');
                last.1.push_str(line.trim());
            }
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            fields.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }
    fields
}

/// Extracts the address from a DSN/ARF address value such as
/// `rfc822; user@example.com` or `<user@example.com>`.
fn extract_address(value: &str) -> String {
    let v = value
        .rsplit_once(';')
        .map(|(_, addr)| addr)
        .unwrap_or(value)
        .trim();
    v.trim_start_matches('<')
        .trim_end_matches('>')
        .trim()
        .to_owned()
}

/// Parses the body of a `message/delivery-status` part into per-recipient
/// records. Per RFC 3464 the body is a per-message field block followed by one
/// blank-line-separated block per recipient.
pub fn parse_delivery_status(body: &str) -> Vec<DsnRecord> {
    let mut records = Vec::new();
    // Blocks are separated by a blank line (handle CRLF and LF).
    for block in body.replace("\r\n", "\n").split("\n\n") {
        let fields = parse_fields(block);
        let mut recipient = None;
        let mut action = None;
        let mut status = None;
        for (name, value) in &fields {
            match name.as_str() {
                "final-recipient" | "original-recipient" if recipient.is_none() => {
                    recipient = Some(extract_address(value));
                }
                "action" => action = Some(DsnAction::parse(value)),
                "status" => status = Some(value.clone()),
                _ => {}
            }
        }
        if let (Some(recipient), Some(action)) = (recipient, action) {
            if !recipient.is_empty() {
                records.push(DsnRecord {
                    recipient,
                    action,
                    status,
                });
            }
        }
    }
    records
}

/// Parses an ARF `message/feedback-report` body, returning the complained
/// recipient addresses (RFC 5965 `Original-Rcpt-To`).
pub fn parse_feedback_report(body: &str) -> Vec<String> {
    let mut recipients = Vec::new();
    for (name, value) in parse_fields(&body.replace("\r\n", "\n")) {
        if name == "original-rcpt-to" || name == "removal-recipient" {
            let addr = extract_address(&value);
            if !addr.is_empty() {
                recipients.push(addr);
            }
        }
    }
    recipients
}

/// Returns the text of the first MIME part whose content-type subtype matches.
fn report_part(raw: &[u8], subtype: &str) -> Option<String> {
    let message = MessageParser::default().parse(raw)?;
    for part in &message.parts {
        if part.content_type().and_then(|c| c.subtype()) == Some(subtype) {
            if let Some(text) = part.text_contents() {
                return Some(text.to_owned());
            }
        }
    }
    None
}

/// Parses a DSN message and suppresses every permanently-failed recipient.
/// Returns the number of recipients suppressed.
pub fn process_dsn(raw: &[u8], list: &SuppressionList) -> usize {
    let Some(body) = report_part(raw, "delivery-status") else {
        return 0;
    };
    let mut suppressed = 0;
    for record in parse_delivery_status(&body) {
        if record.action == DsnAction::Failed {
            list.suppress(&record.recipient, SuppressionReason::HardBounce);
            suppressed += 1;
        }
    }
    suppressed
}

/// Parses an ARF complaint message and suppresses the complained recipients.
/// Returns the number of recipients suppressed.
pub fn process_arf(raw: &[u8], list: &SuppressionList) -> usize {
    let Some(body) = report_part(raw, "feedback-report") else {
        return 0;
    };
    let mut suppressed = 0;
    for recipient in parse_feedback_report(&body) {
        list.suppress(&recipient, SuppressionReason::Complaint);
        suppressed += 1;
    }
    suppressed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crlf(lines: &[&str]) -> Vec<u8> {
        lines.join("\r\n").into_bytes()
    }

    #[test]
    fn parse_fields_folds_continuations() {
        let fields = parse_fields("Action: failed\nDiagnostic-Code: smtp; 550\n very bad");
        assert_eq!(fields[0], ("action".to_owned(), "failed".to_owned()));
        assert_eq!(fields[1].1, "smtp; 550 very bad");
    }

    #[test]
    fn extract_address_variants() {
        assert_eq!(
            extract_address("rfc822; user@example.com"),
            "user@example.com"
        );
        assert_eq!(extract_address("<user@example.com>"), "user@example.com");
        assert_eq!(extract_address("user@example.com"), "user@example.com");
    }

    #[test]
    fn parse_delivery_status_extracts_failed_recipient() {
        let body = "Reporting-MTA: dns; mx.example.com\n\n\
                    Final-Recipient: rfc822; bad@dest.example\n\
                    Action: failed\n\
                    Status: 5.1.1\n\
                    Diagnostic-Code: smtp; 550 5.1.1 user unknown\n";
        let records = parse_delivery_status(body);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].recipient, "bad@dest.example");
        assert_eq!(records[0].action, DsnAction::Failed);
        assert_eq!(records[0].status.as_deref(), Some("5.1.1"));
    }

    #[test]
    fn parse_delivery_status_ignores_delayed() {
        let body = "Final-Recipient: rfc822; slow@dest.example\nAction: delayed\nStatus: 4.4.1\n";
        let records = parse_delivery_status(body);
        assert_eq!(records[0].action, DsnAction::Delayed);
    }

    #[test]
    fn process_dsn_suppresses_hard_bounce() {
        let raw = crlf(&[
            "From: MAILER-DAEMON@mx.example.com",
            "To: sender@send.example",
            "Subject: Delivery Status Notification (Failure)",
            "MIME-Version: 1.0",
            "Content-Type: multipart/report; report-type=delivery-status; boundary=\"b\"",
            "",
            "--b",
            "Content-Type: text/plain",
            "",
            "Delivery to the following recipient failed permanently.",
            "",
            "--b",
            "Content-Type: message/delivery-status",
            "",
            "Reporting-MTA: dns; mx.example.com",
            "",
            "Final-Recipient: rfc822; bad@dest.example",
            "Action: failed",
            "Status: 5.1.1",
            "Diagnostic-Code: smtp; 550 5.1.1 user unknown",
            "",
            "--b",
            "Content-Type: message/rfc822",
            "",
            "From: sender@send.example",
            "To: bad@dest.example",
            "Subject: hi",
            "",
            "--b--",
            "",
        ]);
        let list = SuppressionList::new();
        let n = process_dsn(&raw, &list);
        assert_eq!(n, 1);
        assert!(list.is_suppressed("bad@dest.example"));
        assert_eq!(
            list.reason("bad@dest.example"),
            Some(SuppressionReason::HardBounce)
        );
    }

    #[test]
    fn process_arf_suppresses_complaint() {
        let raw = crlf(&[
            "From: complaints@isp.example",
            "To: fbl@send.example",
            "Subject: complaint",
            "MIME-Version: 1.0",
            "Content-Type: multipart/report; report-type=feedback-report; boundary=\"c\"",
            "",
            "--c",
            "Content-Type: text/plain",
            "",
            "This is an email abuse report.",
            "",
            "--c",
            "Content-Type: message/feedback-report",
            "",
            "Feedback-Type: abuse",
            "User-Agent: SomeFBL/1.0",
            "Version: 1",
            "Original-Rcpt-To: <victim@send.example>",
            "Original-Mail-From: <sender@send.example>",
            "",
            "--c",
            "Content-Type: message/rfc822",
            "",
            "From: sender@send.example",
            "To: victim@send.example",
            "Subject: spam?",
            "",
            "--c--",
            "",
        ]);
        let list = SuppressionList::new();
        let n = process_arf(&raw, &list);
        assert_eq!(n, 1);
        assert!(list.is_suppressed("victim@send.example"));
        assert_eq!(
            list.reason("victim@send.example"),
            Some(SuppressionReason::Complaint)
        );
    }

    #[test]
    fn process_dsn_on_non_dsn_is_noop() {
        let raw = crlf(&[
            "From: a@b.com",
            "To: c@d.com",
            "Subject: regular",
            "Content-Type: text/plain",
            "",
            "just a normal message",
            "",
        ]);
        let list = SuppressionList::new();
        assert_eq!(process_dsn(&raw, &list), 0);
        assert!(list.is_empty());
    }
}
