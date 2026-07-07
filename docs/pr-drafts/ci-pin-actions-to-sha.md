# Descriptor de PR

## Resumen

Fijar las GitHub Actions de terceros a SHA de commit completo (con el tag en comentario) y documentar el bump

## Fase afectada

Endurecimiento de cadena de suministro de CI. No toca codigo de los crates ni
avanza una fase de la Hoja de Ruta.

## Tipo de cambio

- [ ] Correccion de bug
- [x] CI / seguridad (pin de acciones de terceros a SHA)
- [x] Documentacion (nota de mantenimiento en CONTRIBUTING.md)
- [ ] Nueva feature
- [ ] Cambio que rompe compatibilidad

## Contexto

`.github/workflows/*` referenciaban acciones de terceros por tag movil
(`actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, etc.). Un tag movil
permite que cambios no revisados entren a la ruta de build. Esta PR las fija a
un SHA de commit completo, con el tag en un comentario para legibilidad
(CLAUDE.md regla 16: minimizar superficie de ataque; regla 36: reproducibilidad).

Este trabajo se rescato del esfuerzo paralelo de `issues/session-2026-06-13`
(PR #162), cuyo CI ya valido estos SHA en verde; el resto de esa rama es
redundante con lo ya fusionado en `main` (#146/#147/#148/#150).

## Cambios

- `ci.yml`, `docs.yml`, `pr-autofill.yml`, `quality.yml`: cada `uses:` de
  tercero fijado a SHA completo con el tag en comentario.
- `CONTRIBUTING.md`: nota de mantenimiento (revisar y bumpear pins por PR
  dedicada, conservando el tag de referencia).

## Plan de prueba

```sh
# Ninguna accion de tercero sin pinear:
grep -rEn "uses: [^.]" .github/workflows/ | grep -vE "@[0-9a-f]{40}"   # vacio
# YAML valido:
for f in .github/workflows/*.yml; do python3 -c "import yaml;yaml.safe_load(open('$f'))"; done
```

Validacion real: los workflows corren en verde con los SHA fijados (ya
verificado por el CI de la PR #162, de donde provienen los pins).

## Criterios de salida que avanza

- Acciones de terceros reproducibles y revisadas por SHA, no por tag movil.

## Cierre de issues

Closes #140

## Checklist final

- [x] Pertenece a la fase correcta y respeta la documentacion.
- [x] No rompe arquitectura ni anade dependencias.
- [x] Todos los workflows con YAML valido; sin acciones sin pinear.
- [x] Nota de mantenimiento de pins documentada (CONTRIBUTING.md).
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
