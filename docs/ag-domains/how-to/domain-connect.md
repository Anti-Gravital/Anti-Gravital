# How-to — Domain Connect

[Domain Connect](https://www.domainconnect.org/) standardizes DNS setup between a
service provider and a DNS provider, so the operator authorizes a template
instead of pasting records by hand.

Status: the discovery and template-generation core is implemented in
`ag_domains::domain_connect` (blueprint §12). The HTTP fetch of the provider
settings and the live redirect are the integration boundary; the manual flow
remains the default and works at every provider, including Domain-Connect-capable
ones.

## Flow

1. Discover the provider's `_domainconnect` record for the domain
   (`parse_domainconnect_record`).
2. Fetch and parse the provider settings (`parse_settings`) to learn the
   synchronous-UX base URL.
3. Generate the template variables from the Anti-Gravital records
   (`template_variables`) — apex `ip`/`ip6`, `www`/`cname`, and the `_ag-domain`
   TXT ownership. The result is **MX-safe by construction**: only A/AAAA/CNAME/TXT
   are ever emitted, so existing email is preserved.
4. Redirect the operator to the provider's synchronous authorization flow
   (`build_sync_apply_url`).
5. **Verify independently** — Domain Connect accelerates setup, but the ownership
   token and routing records it applies are exactly those
   `generate_instructions` produces, so `ag domains verify` / `diagnose` confirm
   the result with the same path the manual flow uses.

## Today (manual)

```
ag domains attach <domain> --project <p> --edge-host <edge> [--ip <ip>]...
ag domains instructions <domain> --edge-host <edge> [--ip <ip>]...
ag domains verify <domain>
```

The independent verification step is identical to the one Domain Connect would
run afterwards, so the manual flow reaches the same verified state.
