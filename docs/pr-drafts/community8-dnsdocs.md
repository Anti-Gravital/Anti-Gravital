# Resumen

Add bilingual domain-to-deploy loop guide and navigation links (wave 8).

## Fase afectada

Fase 5 (dominios y despliegue). Documentacion de apoyo a la ola 8: el circulo
dominio-despliegue. No cambia el estado alpha ni puertas de fase.

## Tipo de cambio

- [ ] Codigo
- [x] Documentacion
- [ ] Tests
- [ ] CI / build
- [ ] RFC / ADR
- [ ] Cambio de dependencias

## Documentos relacionados

- `docs/dns/README.md` (nuevo): guia bilingue del circulo dominio-despliegue.
- `docs/INDEX.md`, `README.md` (EN y ES), `docs/manual/04-instalacion-y-onboarding.md`:
  enlaces y apendice breve.
- `CHANGELOG.md`: entrada bajo `[Unreleased]`.
- RFC-0011 / ADR-0012 (plano de control y edge), RFC-0007 (alcance inicial).
- RFC-0033 y `docs/dns/DOMAIN_LOOP.md`: diseno de la ola 8 en curso; se
  referencian como planificados sin enlace duro hasta su fusion.

## Plan de prueba

1. `git diff --check` sin errores.
2. Verificar que los enlaces relativos nuevos resuelven:
   `docs/dns/README.md` desde `docs/INDEX.md`, `README.md` (EN y ES) y
   `docs/manual/04-instalacion-y-onboarding.md`.
3. Comprobar paridad EN/ES: mismas secciones y tablas en ambos idiomas, con
   la seccion inglesa canonica primero y ancla `English | Espanol`.
4. Escanear emojis, evidencia de herramientas IA y caracteres no ASCII.
5. Confirma que no se modifica el texto de estado alpha del README.

## Criterios de salida que avanza

Ninguna casilla de `docs/roadmap/STATUS.md` se marca completada: esta
entrega es solo documentacion y no cambia el estado de una fase.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta.
- [x] Respeta la documentacion.
- [x] No rompe arquitectura.
- [x] No anade complejidad innecesaria.
- [x] No crea dependencias circulares.
- [x] Compila (no aplica: solo documentacion).
- [x] Pasa tests (no aplica: verificacion documental).
- [x] Pasa fmt (no aplica: sin codigo).
- [x] Pasa clippy (no aplica: sin codigo).
- [x] Pasa audit (no aplica: sin dependencias nuevas).
- [x] Tiene documentacion (esta entrega es documentacion).
