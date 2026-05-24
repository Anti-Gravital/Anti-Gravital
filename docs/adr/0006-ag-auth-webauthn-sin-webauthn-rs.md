# ADR-0006: ag-auth — WebAuthn implementado sin webauthn-rs

**Estado:** Aceptado
**Fecha:** 2026-05-23
**Crate afectado:** `ag-auth`

---

## Contexto

El documento maestro `ANTI-GRAVITAL-Arquitectura-Tecnica.md` (seccion 8.1) especifica
`webauthn-rs` como la libreria para implementar WebAuthn/FIDO2 en `ag-auth`.

Durante la implementacion de Fase 4 se verifico que `webauthn-rs` usa licencia
**MPL-2.0** (Mozilla Public License 2.0). Esta licencia tiene clausulas de copyleft
de archivo que son incompatibles con la licencia Apache-2.0 del proyecto Anti-Gravital
cuando el codigo de `webauthn-rs` se vincula estaticamente en un binario distribuido.

La regla 15 de `CLAUDE.md` exige justificar cada dependencia por: madurez, mantenimiento,
seguridad, performance, estabilidad y **necesidad real**. La incompatibilidad de licencia
es una razon suficiente para rechazar una dependencia.

---

## Decision

Implementar las ceremonias WebAuthn directamente sobre los primitivos criptograficos,
sin depender de `webauthn-rs`. Las librerias usadas son:

| Rol | Libreria | Licencia |
|---|---|---|
| CBOR (codificacion/decodificacion) | `ciborium` | Apache-2.0 |
| COSE ES256 (verificacion) | `p256` | Apache-2.0 |
| COSE EdDSA (verificacion) | `ed25519-dalek` | Apache-2.0 |
| Encoding base64url | `base64ct` | Apache-2.0 |

Todas estas librerias son Apache-2.0, compatibles con la licencia del proyecto.

---

## Consecuencias

**Positivas:**

- Compatibilidad total de licencias. El binario Anti-Gravital puede distribuirse
  sin restricciones MPL-2.0.
- Menor superficie de dependencias: las mismas librerias ya usadas para JWT y API keys.
- Control total sobre el subset de CBOR/COSE implementado.

**Negativas:**

- El subset WebAuthn implementado cubre los casos de uso principales (ES256, EdDSA)
  pero no todos los algoritmos COSE opcionales que soporta `webauthn-rs`.
- Actualizaciones futuras del estandar WebAuthn requieren trabajo manual en lugar
  de actualizar una dependencia.

**Neutrales:**

- La interfaz publica de `WebAuthnRp` es la misma que habria sido con `webauthn-rs`.
  Un futuro reemplazo de la implementacion interna no cambia el API del crate.

---

## Alternativas consideradas

**Alternativa A: Mantener webauthn-rs con dual licensing.**
`webauthn-rs` no ofrece dual licensing. Descartada.

**Alternativa B: Solicitar RFC para cambiar la licencia del proyecto a MPL-2.0.**
Cambia la estrategia de licencia del proyecto. Impacto demasiado amplio para un
solo modulo. Descartada.

**Alternativa C (elegida): Implementar sobre primitivos Apache-2.0.**
Coste acotado, compatibilidad total, sin impacto en el API publico.

---

## Referencias

- `crates/ag-auth/src/webauthn.rs` — implementacion.
- `docs/architecture/08-modulos-batteries-included.md` seccion 8.1 — spec original.
- `CLAUDE.md` regla 15 (politica de dependencias).
- `CLAUDE.md` regla 22 (toda decision grande requiere RFC o ADR).
