# ADR-0016 - Open interfaces between Anti-Gravital and Gravital Fabric

- Status: accepted
- Date: 2026-11-21
- Decider: BDFL (Angel Nereira)

## Context

Anti-Gravital is the open-source technology layer: framework, components,
protocols, CLI, SDK, security, runtimes and development tools. Gravital Fabric
is a separate infrastructure platform that provisions, connects and administers
physical and cloud resources. A commercial catalog of services is built on both.

The two products must be usable independently. A Fabric installation must not
require Anti-Gravital, and Anti-Gravital components must be usable in
infrastructures that do not run Fabric. A compile-time dependency in either
direction would make that impossible and would turn an integration into a
lock-in.

## Decision

1. The contracts between the layers are published as versioned data: Protobuf
   for service messages, OpenAPI for HTTP APIs and JSON Schema for declarative
   manifests. Each product consumes generated types from those schemas.
2. No crate whose name starts with `ag-` depends on a `fabric-` crate, and no
   `fabric-` crate depends on an `ag-` crate. Implementations may satisfy a
   contract, but they may not import each other.
3. Anti-Gravital may implement a Fabric component as an optional adapter behind
   a Cargo feature. Fabric may use an Anti-Gravital implementation for a
   component when the operator selects it.
4. Every contract has an export path: a user can take their manifests, images,
   volumes, configuration and snapshots out in documented formats without an
   account on either platform.

## Consequences

- Both products can be installed, upgraded and replaced independently.
- A change to a contract is a versioned change, not an implicit code change;
  it needs an RFC in the product that owns the contract.
- Generated clients add a build step, but keep the dependency graph acyclic
  and auditable.
- Adapter labels must follow ADR-0011: a third-party name may label an adapter
  for that third party, never a component of our own.

## Alternatives

- **Shared Rust crate with the types:** rejected, it creates a mandatory
  dependency and couples release cycles.
- **Direct crate dependency in one direction:** rejected, it makes one product
  a hostage of the other's release schedule.
- **No contract, only documentation:** rejected, it cannot be tested and drifts
  immediately.

## Status

Accepted. This ADR is the architectural counterpart of the scope amendment in
RFC-0025 of the Gravital Fabric repository.
