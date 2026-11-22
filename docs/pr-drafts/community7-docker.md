# Descriptor de PR: community7/docker

## Resumen

Add optional reusable Docker packaging for the five template families with honest verification evidence

## Fase afectada

Ola 7, diseno de despliegue. No cambia el estado de ninguna fase ni de ninguna
casilla de `docs/roadmap/STATUS.md`. No toca `docs/master/`. Docker sigue siendo
un destino opcional: `cargo run` y `ag dev` permanecen como ruta por defecto
(regla 12 y ADR-0009).

## Tipo de cambio

- [x] Empaquetado opcional (`deploy/docker/`)
- [x] Documentacion tecnica (`deploy/docker/README.md`, `docs/INDEX.md`)
- [x] Evidencia de verificacion y script reproducible (`verify.sh`,
      `VERIFICATION.md`)
- [x] CHANGELOG (seccion [Unreleased])
- [ ] Codigo de produccion ni comandos CLI
- [ ] Cambio de arquitectura, DSL, security policy o stack
- [ ] Dependencias de Cargo

## Documentos relacionados

- `deploy/docker/README.md`: guia de adaptacion y uso.
- `deploy/docker/VERIFICATION.md`: evidencia real, incluido el SKIP del build.
- `docs/INDEX.md` y `CHANGELOG.md`: indices y registro de cambios.
- Regla 12 (interoperabilidad) y ADR-0009 (modo nativo por defecto).

## Plan de prueba

Ejecutado en este worktree con `bash deploy/docker/verify.sh`:

1. `docker --version`: Docker version 29.10.0, build fdd4b8e.
2. `docker compose version`: Docker Compose version v5.5.1; Python 3.14.4;
   PyYAML 6.0.3.
3. `docker info` y `docker pull busybox:1.36-musl`: fallan con "Cannot connect
   to the Docker daemon at unix:///home/angel-nereira/.docker/desktop/docker.sock".
   No existe `dockerd` en el entorno. Por tanto, build real = SKIP honesto: no
   se construyo, ejecuto ni publico ninguna imagen.
4. `docker compose -f <file> config -q` para los cinco compose: 5/5 exit 0.
   No requiere demonio.
5. Parseo PyYAML de los cinco compose con asercion de `services` y `volumes`:
   5/5 exit 0.
6. Contrato de compose (interpolacion, volumenes con nombre, seccion build):
   5/5.
7. Lint estatico del Dockerfile (`rust:1.99.1`, `scratch`, `USER 65532:65532`,
   `EXPOSE`, `HEALTHCHECK`, multi-etapa, musl, sin ENV de secreto): 8/8.
8. Resumen del script: `PASS=23 FAIL=0 SKIP=2`.

No se ejecutaron `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit` ni
`cargo deny`: este cambio no toca codigo Rust ni dependencias.

## Criterios de salida que avanza

- Ninguna casilla de `STATUS.md`. Entrega empaquetado opcional y verificable
  para las familias de plantillas sin convertir Docker en requisito.

## Notas para el coordinador

- Solo se anade `deploy/docker/`. No se editan `cli.rs`, `commands/`, las
  carpetas de otras ramas (`design`, `k8s`, `deploy`, `stacks`, `security`,
  `deployqa`) ni `docs/master/`.
- `docker compose config` valida los cinco archivos sin demonio; la unica
  verificacion imposible es el build real y queda marcada como SKIP con el
  error exacto en `VERIFICATION.md`. No se afirma una construccion inexistente.
- Los valores secretos de los compose son variables obligatorias (`${VAR:?}`)
  y `.env.example` usa solo el placeholder `CHANGE_ME`.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la ola y gobernanza correctas.
- [x] Respeta la documentacion y no altera los maestros.
- [x] No anade dependencias ni complejidad innecesaria.
- [x] La evidencia de verificacion es real, reproducible y con SKIP honesto.
- [x] Documentacion actualizada en la misma PR.
- [x] Sin emojis ni evidencia de herramientas de IA.
- [x] Mantiene coherencia con Anti-Gravital v4.1.
