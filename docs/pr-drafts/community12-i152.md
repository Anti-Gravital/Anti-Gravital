# Resumen

Document the ag-mail managed API architecture and resource model, with explicit implementation gates (design only, Mail A gate open).

- Fase afectada: Fase 4.6, pista de correo, etapas 4.6-C/D (RFC-0009). No avanza ninguna fase; Mail C/D siguen diferidos.
- Tipo de cambio: [x] documentacion y arquitectura  [ ] CI/workflow  [ ] codigo de crate  [ ] DSL  [ ] API publica.
- Documentos relacionados: `docs/architecture/ag-mail-managed-api-design.md` (nuevo),
  `docs/modules/ag-mail/managed-api-dx-design.md`, `docs/rfc/RFC-0009-ag-mail-native-mta.md`
  (secciones 4.5/4.6 y 5), `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (4.6.3),
  `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`, `docs/DEBT.md` DEBT-021,
  `CLAUDE.md` reglas 0, 1, 7, 8, 15, 22, 26, 28, ADR-0008/0009/0010/0011.

## Cambios

- `docs/architecture/ag-mail-managed-api-design.md`: anexo de arquitectura con
  el modelo de recursos (mensajes, plantillas, listas de supresion, campanas y
  consentimiento) mapeado a los recursos de RFC-0009; limite outbound-only de
  v1; esquemas de endpoints con forma OpenAPI coherentes con el contrato DX;
  composicion con `ag-auth` (solo consumidor, sin dependencia inversa) y
  `ag-workers` (productor de jobs `mail.delivery`); y la seccion de compuertas
  G1-G5 que deben pasar antes de escribir codigo.
- El anexo referencia el kit de verificacion DKIM independiente de #178 sin
  copiarlo ni presentarlo como evidencia de entrega real. No se anade codigo,
  feature, comando CLI ni DSL nuevo.
- Navegacion: `docs/architecture/README.md` (seccion de anexos, no verbatim)
  y `docs/modules/ag-mail/README.md` (enlace al anexo tecnico).
- `CHANGELOG.md`: entrada bajo `[Unreleased]` que deja #152 abierta con `Refs`.

## Plan de prueba

- `git diff --check`
- Verificacion de enlaces relativos de los archivos nuevos y editados.
- Escaneos de contenido de `.github/workflows/docs.yml`: evidencia de
  herramientas IA, prefijos prohibidos en descriptores, emojis, marcas
  comerciales retiradas (ADR-0011) y palabra clave de cierre del descriptor.
- Sin comandos Cargo: el cambio es solo documental, no compila ni ejecuta
  pruebas; no se afirma resultado de test alguno.

## Criterios de salida que avanza

- Hace ejecutable el diseno previo a la implementacion de #152 sin cerrar la
  issue: deja registradas las cinco precondiciones (G1-G5) y la condicion de
  salida de Hoja de Ruta 4.6.3.
- Respeta la regla 28: ningun comando CLI nuevo. Respeta la regla 8: no se
  adelanta la fase 4.6-C/D mientras Mail A carezca de evidencia real.

## Cierre de issues

Refs #152
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (4.6-C/D, diseno, ola 12).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares: explicita la
      direccion `ag-auth -> ag-mail` y `ag-mail -> ag-workers`.
- [x] No anade complejidad innecesaria: reutiliza el contrato DX y el kit DKIM
      de #178 por referencia.
- [x] Compila / no aplica (sin codigo Rust).
- [x] Pasa `git diff --check` y los escaneos de contenido.
- [x] Tiene documentacion: anexo, navegacion, descriptor y CHANGELOG.
- [x] No inventa evidencia de entrega ni cierra la puerta Mail A.
