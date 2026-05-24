# Anti-DSL - Implementacion incremental por versiones

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 7.2.

### 7.2 Implementación incremental por versiones del DSL

Probablemente la decisión más importante para que el compilador sea viable es admitir que no se puede entregar el lenguaje completo en la primera versión. La especificación se entrega en fases incrementales, cada una con una gramática estable que no rompe la anterior. Las versiones del DSL son independientes de las versiones del framework y siguen su propio semver.

| Versión DSL | Capacidad gramatical                                                                                              | Hito                       |
|-------------|-------------------------------------------------------------------------------------------------------------------|----------------------------|
| v0.1        | Modelos básicos: campos, tipos primitivos, anotaciones `@primary`, `@unique`, `@auto`                              | Fin Fase 3 (entregado)     |
| v0.2        | Endpoints: método, path, body, response, errors                                                                    | Fin Fase 3 (entregado)     |
| v0.3        | Validaciones: `@min`, `@max`, `@email`, `@regex`, `@length`                                                        | Fin Fase 3 (entregado)     |
| v0.4        | Relaciones entre modelos: `1:1`, `1:N`, `N:M`, cascadas                                                            | Fin Fase 3 (entregado)     |
| v0.5        | Autenticación y autorización: `auth required`, `policy "..."`                                                      | Fin Fase 4 (entregado)     |
| v0.6        | Eventos: declaración de eventos emitidos por endpoint, suscriptores                                                | Fin Fase 4 (entregado)     |
| v0.7        | Mail y dominios declarativos: `mail`, `domain`, `dns`, `tls`                                                       | Fin Fase 4.5               |
| v0.8        | Plugin hooks (lifecycle, decoradores)                                                                              | Fin Fase 9                 |
| v1.0        | Gramática estable, congelada bajo semver. Cualquier extensión posterior será aditiva.                              | Fin Fase 10                |

Esta tabla está realineada por `ADR-0007` (Fase 4.5). Las capacidades de
multi-tenancy y migración de datos previstas para versiones intermedias del
DSL en revisiones anteriores quedan diferidas: se especificarán en RFCs
propios cuando el alcance lo justifique, sin ocupar un slot numerado fijo
hasta entonces. Esto evita prometer features que no tienen tracción
verificada.

## v0.7 — Bloques `mail` y `domain` (Fase 4.5)

Los bloques `mail` y `domain` del DSL v0.7 son la superficie declarativa de los
crates `ag-mail` y `ag-domains` introducidos por `ADR-0007`. El compilador
valida en build-time cuatro invariantes que dejan de ser bugs de runtime y se
convierten en errores de compilación:

1. El `from` de un bloque `mail` referencia un dominio declarado en un bloque
   `domain` del mismo `schema.ag`.
2. El archivo de template referenciado por `mail` existe en disco.
3. Las variables del HTML del template coinciden exactamente con las `vars`
   tipadas declaradas en el bloque `mail`.
4. El dominio tiene SPF/DKIM/DMARC configurados (modo `auto` que delega a
   `ag-domains`) o marcados explícitamente como pendientes.

Ejemplo bilingüe de ambos bloques en `docs/dsl/ejemplo-completo.md` y en los
READMEs de los módulos:

- `docs/modules/ag-mail/README.md`
- `docs/modules/ag-domains/README.md`
