## Resumen
Anadir tests/run.sh con selftests deterministicos del framework para desbloquear el job kaizen-gates/framework.

## Componente afectado
Framework Kaizen local / compuerta de tests (KZ_TEST_CMD).

## Tipo de cambio
- [x] enhancement
- [ ] bug
- [ ] tech-debt
- [ ] documentation
- [ ] refactor

## Documentos relacionados
- CHANGELOG.md (entrada bajo [Unreleased])
- .kaizen/config.env (KZ_TEST_CMD ya apuntaba a tests/run.sh)

## Plan de prueba
- `bash tests/run.sh`
- `scripts/kaizen check --only tests`
- `git diff --check`

## Impacto en calidad (ratchet)
- Cobertura: igual (no se toca codigo de produccion)
- Advertencias: igual (KZ_LINT_CMD ya incluia tests/run.sh en bash -n)
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- `bash tests/run.sh` termina con exit 0 y 0 fallos.
- `scripts/kaizen check --only tests` en verde.
- Ningun check se omite silenciosamente: se reporta SKIP cuando aplica.

## Aprendizaje (opcional)
El selftest heredado tests/hooks/test-hooks.sh tiene dos fallos dependientes del entorno (core.hooksPath local y deriva de hash en docs/master). El runner usa un subconjunto determinista equivalente que no depende de esos estados ni de red.

## Cierre de issues
Closes: none

## Checklist
- [x] Compila (bash -n incluido en el propio runner)
- [x] Pasa tests
- [x] `scripts/kaizen check --only tests` en verde
- [x] Documentacion actualizada (CHANGELOG)
- [x] Linea base no empeora
