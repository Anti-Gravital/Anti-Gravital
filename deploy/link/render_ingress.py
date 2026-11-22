#!/usr/bin/env python3
"""Render a Kustomize overlay that binds a generated Ingress to a domain.

The domain manifest follows the normative environment/visibility contract in
`docs/dns/ENVIRONMENT_RULES.md` (rules ER-01..ER-11): `schema_version: 1`,
environments `dev`, `staging` and `prod`, a strict key set and explicit
visibility. This tool never contacts a DNS provider, a cluster or a registry:
it writes a local Kustomization plus a JSON6902 patch. Kubernetes, Docker and
DNS remain optional integration targets (CLAUDE.md rule 12, ADR-0009 rule 2).

Rules enforced here (the parts that affect deploy artifacts):

- ER-01 strict keys; ER-02a no `local` manifest entry; ER-03 private defaults;
- ER-04 missing visibility resolves to private, unknown values are rejected;
- ER-05/ER-07 public and vpn-only preconditions, including source CIDRs;
- ER-08 a private service never gets an Ingress rule, even if it has a host;
- ER-09/ER-10 host allowlist and wildcard rules; ER-TLS bound name requires TLS.

Manifest shape (see docs/dns/examples/domain-manifest.yaml):

    schema_version: 1
    project: my-project
    default_visibility: private
    environments:
      prod:
        visibility_default: private
        zone: example.invalid
        allowed_hosts: [api.example.invalid]
        allow_wildcard: false
        tls: {required: true, termination: edge, min_version: "1.2", hsts: true}
        ingress: true
        service_type: ClusterIP
        services:
          - name: api
            host: api.example.invalid
            visibility: public
            port: 8080
            allow_sources: ["0.0.0.0/0", "::/0"]

Usage:

    python3 deploy/link/render_ingress.py \
      --manifest examples/domain-link/domain.yaml \
      --env prod --service api \
      --out /tmp/ag-domain-link/prod
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - environment without PyYAML
    print("ERROR: PyYAML is required to parse the domain manifest", file=sys.stderr)
    raise SystemExit(2)


DOMAIN_RE = re.compile(
    r"^(?=.{1,253}$)"
    r"[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?"
    r"(?:\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)+$"
)
CIDR_RE = re.compile(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}/[0-9]{1,2}$|^[0-9a-fA-F:]+/[0-9]{1,3}$")
INTERNET_CIDRS = {"0.0.0.0/0", "::/0"}

TOP_KEYS = {"schema_version", "project", "default_visibility", "environments"}
ENV_KEYS = {
    "visibility_default", "zone", "allowed_hosts", "allow_wildcard",
    "tls", "ingress", "service_type", "services",
}
TLS_KEYS = {"required", "termination", "min_version", "hsts"}
SERVICE_KEYS = {"name", "host", "visibility", "port", "allow_sources"}
VISIBILITIES = {"private", "vpn-only", "public"}
TERMINATIONS = {"none", "internal", "edge"}
DEPLOY_ENVS = {"dev", "staging", "prod"}
BACKEND_SERVICE = "template"


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(2)


def strict_keys(mapping: dict, allowed: set[str], where: str) -> None:
    unknown = set(mapping) - allowed
    if unknown:
        fail(f"{where}: unknown key(s) {sorted(unknown)} (ER-01 strict schema)")


def load_manifest(path: Path) -> dict:
    if not path.is_file():
        fail(f"manifest not found: {path}")
    try:
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
    except yaml.YAMLError as exc:
        fail(f"invalid YAML in {path}: {exc}")
    if not isinstance(data, dict):
        fail("manifest root must be a mapping")
    strict_keys(data, TOP_KEYS, "manifest")
    if data.get("schema_version") != 1:
        fail("unsupported schema_version (expected 1, ER-01)")
    if not isinstance(data.get("project"), str) or not data["project"].strip():
        fail("project must be a non-empty string")
    if data.get("default_visibility") != "private":
        fail("default_visibility must be private (ER-03)")
    envs = data.get("environments")
    if not isinstance(envs, dict) or not envs:
        fail("environments must be a non-empty mapping")
    if "local" in envs:
        fail("environment 'local' has no domain manifest entry (ER-02a)")
    for name in envs:
        if name not in DEPLOY_ENVS:
            fail(f"unknown environment '{name}'; allowed: {sorted(DEPLOY_ENVS)} (ER-02b)")
    return data


def validate_environment(env_name: str, env: dict) -> None:
    strict_keys(env, ENV_KEYS, f"environments.{env_name}")
    if env.get("visibility_default") != "private":
        fail(f"{env_name}: visibility_default must be private (ER-03)")
    allowed = env.get("allowed_hosts")
    if not isinstance(allowed, list) or not allowed or not all(isinstance(h, str) for h in allowed):
        fail(f"{env_name}: allowed_hosts must be a non-empty list of strings (ER-09)")
    if not isinstance(env.get("zone"), str) or not env["zone"]:
        fail(f"{env_name}: zone must be a non-empty string")
    if not isinstance(env.get("allow_wildcard"), bool):
        fail(f"{env_name}: allow_wildcard must be a boolean")
    if env_name == "prod" and not env["allow_wildcard"]:
        for entry in allowed:
            if entry.startswith("*."):
                fail(f"{env_name}: wildcard host needs allow_wildcard and review (ER-09)")
    tls = env.get("tls")
    if not isinstance(tls, dict):
        fail(f"{env_name}: tls must be a mapping")
    strict_keys(tls, TLS_KEYS, f"{env_name}.tls")
    if not isinstance(tls.get("required"), bool):
        fail(f"{env_name}: tls.required must be a boolean")
    if tls.get("termination") not in TERMINATIONS:
        fail(f"{env_name}: tls.termination must be one of {sorted(TERMINATIONS)}")
    if not isinstance(tls.get("hsts"), bool):
        fail(f"{env_name}: tls.hsts must be a boolean")
    if not isinstance(env.get("ingress"), bool):
        fail(f"{env_name}: ingress must be a boolean")
    if not isinstance(env.get("service_type"), str) or not env["service_type"]:
        fail(f"{env_name}: service_type must be a non-empty string")
    services = env.get("services")
    if not isinstance(services, list) or not services:
        fail(f"{env_name}: services must be a non-empty list")
    names: set[str] = set()
    for service in services:
        if not isinstance(service, dict):
            fail(f"{env_name}: each service must be a mapping")
        strict_keys(service, SERVICE_KEYS, f"{env_name}.services[]")
        name = service.get("name")
        if not isinstance(name, str) or not name:
            fail(f"{env_name}: service.name must be a non-empty string")
        if name in names:
            fail(f"{env_name}: duplicate service name '{name}' (ER-10)")
        names.add(name)
        if not isinstance(service.get("port"), int) or service["port"] <= 0:
            fail(f"{env_name}.{name}: port must be a positive integer")
        sources = service.get("allow_sources", [])
        if not isinstance(sources, list) or not all(isinstance(c, str) for c in sources):
            fail(f"{env_name}.{name}: allow_sources must be a list of CIDR strings")
        for cidr in sources:
            if not CIDR_RE.match(cidr):
                fail(f"{env_name}.{name}: invalid CIDR '{cidr}'")


def host_allowed(host: str, env: dict) -> bool:
    """ER-10: exact match or one-label wildcard when explicitly allowed."""
    for entry in env["allowed_hosts"]:
        if host == entry:
            return True
        if env["allow_wildcard"] and entry.startswith("*."):
            zone = entry[2:]
            if host.endswith("." + zone) and host.count(".") == zone.count(".") + 1:
                return True
    return False


def select_service(manifest: dict, env_name: str, service_name: str, tls_secret: str | None) -> dict:
    if env_name == "local":
        return {
            "visibility": "private", "host": None, "port": 8080,
            "allow_sources": [], "tls": False, "ingress": False,
        }
    if env_name not in manifest["environments"]:
        fail(f"environment '{env_name}' not in manifest; have: {', '.join(sorted(manifest['environments']))}")
    env = manifest["environments"][env_name]
    validate_environment(env_name, env)
    service = next((s for s in env["services"] if s.get("name") == service_name), None)
    if service is None:
        available = ", ".join(sorted(s["name"] for s in env["services"]))
        fail(f"service '{service_name}' not in {env_name}; have: {available}")

    raw_visibility = service.get("visibility")
    if raw_visibility is None:
        visibility = env["visibility_default"]  # ER-04: fail closed to private
    elif raw_visibility in VISIBILITIES:
        visibility = raw_visibility
    else:
        fail(f"{env_name}.{service_name}: unknown visibility '{raw_visibility}' (ER-04 hard error)")

    host = service.get("host")
    if host is not None:
        if not isinstance(host, str) or not DOMAIN_RE.match(host):
            fail(f"{env_name}.{service_name}: host must be a lowercase DNS name with a dot")
        if not host_allowed(host, env):
            fail(f"{env_name}.{service_name}: host '{host}' not in allowed_hosts (ER-10)")
        if not env["tls"]["required"]:
            fail(f"{env_name}: a bound host requires tls.required true (ER-TLS)")

    sources = service.get("allow_sources", [])
    has_internet = any(cidr in INTERNET_CIDRS for cidr in sources)

    if visibility == "private":
        # ER-08: a host never turns a private service into a public one.
        if sources:
            fail(f"{env_name}.{service_name}: private service must not declare allow_sources (ER-08)")
        exposed = False
    else:
        if not host:
            fail(f"{env_name}.{service_name}: {visibility} requires a host (ER-05)")
        if not env["ingress"]:
            fail(f"{env_name}.{service_name}: {visibility} requires ingress: true (ER-05)")
        if visibility == "public":
            if not env["tls"]["required"] or env["tls"]["termination"] == "none":
                fail(f"{env_name}.{service_name}: public requires TLS with termination != none (ER-05)")
            if not has_internet:
                fail(f"{env_name}.{service_name}: public needs an internet CIDR in allow_sources (ER-07)")
        else:  # vpn-only
            if not sources or has_internet:
                fail(f"{env_name}.{service_name}: vpn-only needs non-empty sources without internet CIDR (ER-07)")
        exposed = True

    use_tls = exposed and env["tls"]["required"] and env["tls"]["termination"] != "none"
    secret = tls_secret or f"ag-tls-{manifest['project']}-{env_name}"
    return {
        "visibility": visibility,
        "host": host if exposed else None,
        "port": service["port"],
        "allow_sources": sources,
        "tls": use_tls,
        "tls_secret": secret,
        "ingress": exposed,
        "termination": env["tls"]["termination"],
    }


def build_kustomization(k8s_rel: str, env: str, service: dict, patch_ops: list[dict]) -> str:
    header = (
        "# Generated by deploy/link/render_ingress.py. Do not edit by hand.\n"
        "# Re-run the generator after changing the domain manifest.\n"
        f"# Environment: {env} | Visibility: {service['visibility']} | "
        f"Ingress: {str(service['ingress']).lower()}\n"
    )
    body = (
        "apiVersion: kustomize.config.k8s.io/v1beta1\n"
        "kind: Kustomization\n\n"
        "namespace: anti-gravital\n\n"
        "resources:\n"
        f"  - {k8s_rel}/overlays/api\n"
    )
    if not service["ingress"]:
        # ER-02a/ER-08: no Ingress component and no patch for local/private.
        return header + body

    patch_text = "\n".join(
        "          " + line for line in json.dumps(patch_ops, indent=2).splitlines()
    )
    body += (
        "\ncomponents:\n"
        f"  - {k8s_rel}/optional/ingress\n"
        "\n# JSON6902 avoids depending on Ingress strategic-merge list keys.\n"
        "patches:\n"
        "  - target:\n"
        "      group: networking.k8s.io\n"
        "      version: v1\n"
        "      kind: Ingress\n"
        "      name: template\n"
        "      namespace: anti-gravital\n"
        "    patch: |-\n"
        f"{patch_text}\n"
    )
    return header + body


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path, help="domain manifest (YAML)")
    parser.add_argument(
        "--env", required=True,
        help="environment key: dev, staging, prod, or local (loopback only)",
    )
    parser.add_argument("--service", default="api", help="service name in the environment")
    parser.add_argument("--out", required=True, type=Path, help="output overlay directory")
    parser.add_argument("--tls-secret", default=None, help="override the derived TLS Secret name")
    parser.add_argument("--k8s-root", type=Path, default=None, help="path to deploy/k8s")
    args = parser.parse_args()

    manifest = load_manifest(args.manifest)
    service = select_service(manifest, args.env, args.service, args.tls_secret)

    repo_root = Path(__file__).resolve().parents[2]
    k8s_root = (args.k8s_root or (repo_root / "deploy" / "k8s")).resolve()
    if not (k8s_root / "overlays" / "api" / "kustomization.yaml").is_file():
        fail(f"k8s root is missing overlays/api: {k8s_root}")

    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    k8s_rel = os.path.relpath(k8s_root, out).replace(os.sep, "/")

    patch_ops: list[dict] = []
    if service["ingress"]:
        host = service["host"]
        annotations = {
            "anti-gravital.io/environment": args.env,
            "anti-gravital.io/visibility": service["visibility"],
            "anti-gravital.io/project": manifest["project"],
        }
        if service["visibility"] == "vpn-only":
            # Metadata for downstream policy tools; not enforcement by itself.
            annotations["anti-gravital.io/allow-sources"] = ",".join(service["allow_sources"])
        patch_ops = [
            {"op": "replace", "path": "/spec/rules/0/host", "value": host},
            {"op": "add", "path": "/metadata/annotations", "value": annotations},
        ]
        if service["tls"]:
            patch_ops.append(
                {
                    "op": "add",
                    "path": "/spec/tls",
                    "value": [{"hosts": [host], "secretName": service["tls_secret"]}],
                }
            )

    (out / "kustomization.yaml").write_text(
        build_kustomization(k8s_rel, args.env, service, patch_ops),
        encoding="utf-8",
    )

    if args.env == "local":
        url = "http://127.0.0.1:8080"
    elif service["ingress"]:
        scheme = "https" if service["tls"] else "http"
        url = f"{scheme}://{service['host']}"
    else:
        url = ""  # private: no DNS surface (ER-08)

    print(f"PROJECT={manifest['project']}")
    print(f"ENV={args.env}")
    print(f"SERVICE={args.service}")
    print(f"HOST={service['host'] or ''}")
    print(f"VISIBILITY={service['visibility']}")
    print(f"TLS={'true' if service['tls'] else 'false'}")
    print(f"INGRESS={'true' if service['ingress'] else 'false'}")
    print(f"URL={url}")
    print(f"OVERLAY={out}")
    print(f"K8S_ROOT={k8s_root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
