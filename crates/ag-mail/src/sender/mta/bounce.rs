//! Bounce classification for the native MTA, per RFC 3463 (enhanced status
//! codes) and RFC 5321 (basic reply codes).
//!
//! A synchronous SMTP failure (or an asynchronous DSN diagnostic code) is
//! reduced to a single decision: retry later (transient) or stop and suppress
//! the recipient (permanent). This is the minimal, pure-logic core that the
//! delivery loop and, later, the suppression list and the scheduled queue
//! build on. It performs no I/O and depends on nothing external, so it is
//! exhaustively unit-tested.

/// What the delivery loop should do with a failed recipient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BounceClass {
    /// 4xx — temporary failure. The message should be retried with backoff.
    Transient,
    /// 5xx — permanent failure. The recipient should be suppressed.
    Permanent,
}

/// An RFC 3463 enhanced status code (`class.subject.detail`, e.g. `5.1.1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnhancedStatus {
    /// Class digit: 2 (success), 4 (transient), 5 (permanent).
    pub class: u8,
    /// Subject sub-code (1 address, 2 mailbox, 3 system, 4 network, ...).
    pub subject: u16,
    /// Detail sub-code.
    pub detail: u16,
}

/// The outcome of classifying an SMTP failure response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BounceVerdict {
    /// Retry-or-suppress decision.
    pub class: BounceClass,
    /// The basic 3-digit reply code (e.g. `550`).
    pub basic_code: u16,
    /// The enhanced status code, when present in the reply text.
    pub enhanced: Option<EnhancedStatus>,
}

/// Classifies a basic SMTP reply code.
///
/// Returns `None` for non-failure codes (2xx/3xx) and for codes outside the
/// valid 400-599 failure range: the caller only classifies failures.
pub fn classify_code(code: u16) -> Option<BounceClass> {
    match code {
        400..=499 => Some(BounceClass::Transient),
        500..=599 => Some(BounceClass::Permanent),
        _ => None,
    }
}

/// Parses an enhanced status code out of an SMTP reply, if one is present.
///
/// Accepts the leading token form `"5.1.1 ..."` and tolerates the basic code
/// prefix `"550 5.1.1 ..."`.
pub fn parse_enhanced(text: &str) -> Option<EnhancedStatus> {
    for token in text.split_whitespace() {
        let mut parts = token.split('.');
        let (Some(c), Some(s), Some(d), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let (Ok(class), Ok(subject), Ok(detail)) =
            (c.parse::<u8>(), s.parse::<u16>(), d.parse::<u16>())
        else {
            continue;
        };
        if matches!(class, 2 | 4 | 5) {
            return Some(EnhancedStatus {
                class,
                subject,
                detail,
            });
        }
    }
    None
}

/// Classifies a full SMTP reply line such as `"550 5.1.1 user unknown"`.
///
/// The decision uses the enhanced class when it disagrees with the basic code
/// (the enhanced code is more specific), otherwise the basic code. Returns
/// `None` when the line carries no failure code.
pub fn classify_reply(reply: &str) -> Option<BounceVerdict> {
    let trimmed = reply.trim_start();
    let basic_code = trimmed
        .split_whitespace()
        .next()
        .and_then(|t| t.parse::<u16>().ok())?;

    let enhanced = parse_enhanced(trimmed);

    // Prefer the enhanced class when present; it is the authoritative signal.
    let class = match enhanced.map(|e| e.class) {
        Some(4) => BounceClass::Transient,
        Some(5) => BounceClass::Permanent,
        _ => classify_code(basic_code)?,
    };

    Some(BounceVerdict {
        class,
        basic_code,
        enhanced,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_ranges() {
        assert_eq!(classify_code(250), None);
        assert_eq!(classify_code(354), None);
        assert_eq!(classify_code(421), Some(BounceClass::Transient));
        assert_eq!(classify_code(450), Some(BounceClass::Transient));
        assert_eq!(classify_code(550), Some(BounceClass::Permanent));
        assert_eq!(classify_code(554), Some(BounceClass::Permanent));
        assert_eq!(classify_code(600), None);
    }

    #[test]
    fn parse_enhanced_basic() {
        assert_eq!(
            parse_enhanced("5.1.1 user unknown"),
            Some(EnhancedStatus {
                class: 5,
                subject: 1,
                detail: 1
            })
        );
        assert_eq!(
            parse_enhanced("550 5.7.1 blocked"),
            Some(EnhancedStatus {
                class: 5,
                subject: 7,
                detail: 1
            })
        );
    }

    #[test]
    fn parse_enhanced_none_when_absent() {
        assert_eq!(parse_enhanced("user unknown"), None);
        // Version-looking token but class digit not 2/4/5.
        assert_eq!(parse_enhanced("1.2.3 nope"), None);
    }

    #[test]
    fn permanent_hard_bounce() {
        let v = classify_reply("550 5.1.1 <x@y.com>: user unknown").unwrap();
        assert_eq!(v.class, BounceClass::Permanent);
        assert_eq!(v.basic_code, 550);
        assert_eq!(v.enhanced.unwrap().subject, 1);
    }

    #[test]
    fn transient_soft_bounce() {
        let v = classify_reply("451 4.7.0 greylisted, try again").unwrap();
        assert_eq!(v.class, BounceClass::Transient);
        assert_eq!(v.basic_code, 451);
    }

    #[test]
    fn enhanced_class_overrides_when_present() {
        // Basic 500-range but enhanced says transient (rare, but enhanced wins).
        let v = classify_reply("500 4.3.2 system busy").unwrap();
        assert_eq!(v.class, BounceClass::Transient);
    }

    #[test]
    fn basic_only_no_enhanced() {
        let v = classify_reply("452 too many recipients").unwrap();
        assert_eq!(v.class, BounceClass::Transient);
        assert!(v.enhanced.is_none());
    }

    #[test]
    fn non_failure_returns_none() {
        assert!(classify_reply("250 2.0.0 OK").is_none());
        assert!(classify_reply("220 mx.example.com ESMTP").is_none());
        assert!(classify_reply("not a reply").is_none());
    }
}
