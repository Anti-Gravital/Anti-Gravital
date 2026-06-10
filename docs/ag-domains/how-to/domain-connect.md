# How-to — Domain Connect

[Domain Connect](https://www.domainconnect.org/) standardizes DNS setup between a
service provider and a DNS provider, so the operator authorizes a template
instead of pasting records by hand.

Status: Domain Connect automation is a later phase (RFC-0011 phase E; see
`docs/DEBT.md`, DEBT-025). Until then, use the manual flow — it works at every
provider, including Domain-Connect-capable ones.

## Intended flow (phase E)

1. Discover the provider's `_domainconnect` record for the domain.
2. Generate a Domain Connect template for the Anti-Gravital records (apex/www/
   subdomain attach, `_ag-domain` TXT ownership, MX-safe so existing email is
   preserved, wildcard where supported).
3. Redirect the operator to the DNS provider's authorization flow.
4. Return to Anti-Gravital after confirmation.
5. **Verify independently** — Domain Connect accelerates setup, but Anti-Gravital
   still confirms the resulting records with `ag domains verify` / `diagnose`.

## Today (manual)

```
ag domains attach <domain> --project <p> --edge-host <edge> [--ip <ip>]...
ag domains instructions <domain> --edge-host <edge> [--ip <ip>]...
ag domains verify <domain>
```

The independent verification step is identical to the one Domain Connect would
run afterwards, so the manual flow reaches the same verified state.
