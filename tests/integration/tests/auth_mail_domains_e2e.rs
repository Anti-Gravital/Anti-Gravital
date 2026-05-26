//! Cross-module E2E: ag-domains generates email-authorizing DNS records, and
//! ag-mail enqueues a verification email through the in-memory queue.
//! Uses NullSender so no real SMTP server is needed; runs in CI.

use std::sync::Arc;

use ag_domains::mail_records::{
    DkimSelector, DmarcPolicy, MailDnsRequirements, generate_mail_records,
};
use ag_mail::message::{Address, EmailBuilder};
use ag_mail::queue::{InMemoryQueue, MailQueue, RetryPolicy};
use ag_mail::sender::test_utils::NullSender;

#[tokio::test]
async fn auth_sends_verification_via_mail_with_domain_records() {
    // 1. ag-domains produces the email-authorizing DNS records for the sender domain.
    let req = MailDnsRequirements {
        domain: "example.com".to_owned(),
        dkim_selectors: vec![DkimSelector {
            name: "ag1".to_owned(),
            public_key_record: "v=DKIM1; k=rsa; p=MIIBIjAN".to_owned(),
        }],
        spf_includes: vec![],
        spf_ips: vec![],
        dmarc_policy: DmarcPolicy::Quarantine,
        dmarc_rua: None,
    };
    let records = generate_mail_records(&req);
    assert!(
        records.iter().any(|r| r.content.contains("v=spf1")),
        "must include SPF record"
    );
    assert!(
        records.iter().any(|r| r.content.contains("DMARC")),
        "must include DMARC record"
    );
    assert!(
        records.iter().any(|r| r.name.contains("_domainkey")),
        "must include DKIM record"
    );

    // 2. ag-mail enqueues a verification email through the in-memory queue.
    let sender = Arc::new(NullSender::new());
    let queue = InMemoryQueue::new(sender.clone(), RetryPolicy::default(), 32);
    let email = EmailBuilder::new()
        .from(Address::new("noreply@example.com"))
        .to(Address::new("user@example.com"))
        .subject("Verify your account")
        .text_body("Click the link to verify your account.")
        .build()
        .expect("valid email must build");
    queue.enqueue(email).await.expect("enqueue must succeed");

    // 3. Allow the worker to flush, then assert the email was captured.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_eq!(sender.emails_sent(), 1, "verification email must be sent once");
}
