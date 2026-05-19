# ag-auth

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`.
> README del crate: `crates/ag-auth/README.md`.
> Criticidad: Estandar.
> Fase de implementacion: Fase 4.

## Dominio

Autenticacion y autorizacion: WebAuthn, JWT Ed25519, OAuth2, RBAC declarativo, rate limiting.

## Dependencias internas permitidas

Depende de ag-core. Puede depender de ag-data para persistencia de sesiones.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate `crates/ag-auth/` esta declarado en el workspace con
`src/lib.rs` vacio y `README.md` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
