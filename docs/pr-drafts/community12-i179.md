# Resumen

Compile spike real de la migracion de instant-acme 0.7 a 0.8 para #179,sin tocar codigo de produccion: evidencia de compilacion, correcciones a lareferencia y plan de parche listo para cuando #87 aporte la evidencia destaging.

- Fase afectada: mantenimiento de `ag-domains` (ola 12); no altera fases delroadmap ni alcance de RFC-0011.
- Tipo de cambio: [x] documentacion y auditoria  [ ] CI/workflow  [ ] codigo de crate  [ ] DSL  [ ] API publica.
- Documentos relacionados: `docs/audits/instant-acme-08-compile-spike.md`,
  `docs/ag-domains/reference/instant-acme-0.8-migration.md`,
  `docs/ag-domains/README.md`, `docs/ag-domains/how-to/acme-staging-test.md`,
  `docs/ag-domains/acme-mock-evidence.md`, RFC-0011, CLAUDE.md reglas 5, 19, 22, 26 y 29.

## Cambios

- `docs/audits/instant-acme-08-compile-spike.md`: documento nuevo con elentorno real (base 72b97ad, rustc 1.99.1, 0.8.0 y 0.8.5), los ocho erroresde compilacion verbatim, la verificacion de los nueve puntos de migracion,las tres correcciones a la referencia, el diff completo del parche ya probadoy la secuencia exacta de cierre de #179 tras #87.
- `docs/ag-domains/reference/instant-acme-0.8-migration.md`: el bloque dediagnosticos se etiqueta como historico y se anotan las lineas reales delarbol actual (67, 89, 255, 264, 285, 287, 295, 327); el punto 7 pasa a 287;el punto 9 se aclara como item de verificacion de comportamiento, no comoerror de compilador; enlace al spike.
- `docs/ag-domains/README.md`: enlace al spike desde la entrada de la referenciade migracion bloqueada.
- Sin cambios en `crates/ag-domains/**`, `Cargo.toml`, `Cargo.lock` ni en ningunotro archivo de produccion. La dependencia sigue en 0.7.

## Plan de prueba

- Copia desechable fuera del repo; `instant-acme = "0.8"` solo en la copia.
- `cargo check -p ag-domains --features acme --message-format=short --offline`con 0.8.5 y con `=0.8.0`: 8 errores identicos, exit 101.
- Aplicar el parche del spike a la copia; `cargo check` exit 0,`rustfmt --check` exit 0, `cargo test -p ag-domains --all-features` exit 0(171 + 8 + 2 tests; 3 ignorados de Postgres y 5 doc-tests ignorados).
- `git diff --check` y escaneos de contenido (emojis, evidencia de herramientasIA, marcas retiradas ADR-0011, palabras clave de cierre del descriptor).
- No se contacto ningun directorio ACME; el mock local no sustituye al staging.

## Criterios de salida que avanza

- Deja verificados y corregidos los nueve puntos de la referencia de migracion.
- Deja un parche compilado y con tests en verde, listo para aplicarse cuando #87entregue `staging-before.log` y `staging-after.log` reales.
- #179 permanece ABIERTO con `blocked`; no se cierra ni se afirma que la puerta paso.

## Cierre de issues

Refs #179
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (ola 12, una issue por agente).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: solo documentacion de evidencia.
- [x] No aplica compilacion en el repo: ningun archivo Rust cambio aqui; elparche se compilo en copia desechable.
- [x] Pasa `git diff --check` y los escaneos de contenido.
- [x] Tiene documentacion (auditoria, referencia corregida e indice).
- [x] No inventa resultados de staging ni cierra #179.
