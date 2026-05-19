# Modelo de seguridad

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 15.

## 15. Modelo de seguridad

La seguridad es una preocupación transversal, no un módulo. Esta sección documenta las garantías y las prácticas del proyecto.

### 15.1 Garantías por construcción

Rust elimina por construcción cuatro categorías de bugs que históricamente representan más del 70% de las vulnerabilidades críticas en software de sistemas: use-after-free, buffer overflows, data races, y null pointer dereferences. Estas garantías son a nivel de compilador, no de runtime; no requieren GC ni runtime checks.

Anti-Gravital prohíbe el uso de `unsafe` en todo el código del framework salvo en bloques explícitamente justificados, documentados, y revisados por al menos dos mantenedores. Cada bloque `unsafe` viene acompañado de un comentario que explica por qué es necesario y qué invariantes preserva.

### 15.2 Prácticas de criptografía

Las primitivas criptográficas se importan del crate `ring`, mantenido por miembros del equipo BoringSSL de Google. No se rueda criptografía propia. Los algoritmos por defecto son Ed25519 para firmas, ChaCha20-Poly1305 para AEAD, Argon2id para hashing de passwords, y TLS 1.3 para transporte. Algoritmos heredados (RSA, AES-CBC, SHA-1) están disponibles solo para interoperabilidad explícita.

### 15.3 Política de divulgación responsable

El repositorio mantiene un archivo `SECURITY.md` con direcciones de contacto (primario `anti@gravitalcloud.com`, respaldo `angelnereira@gravitalcloud.com`) y una política clara: las vulnerabilidades se reportan privadamente, el equipo confirma recepción en 48 horas, publica un parche en 30 días para vulnerabilidades críticas, y un CVE con crédito al reportero.

### 15.4 Auditorías

Antes de la versión 1.0 estable, el componente Shield del framework se somete a una auditoría externa por una empresa especializada en seguridad de sistemas Rust (Trail of Bits, NCC Group o equivalente). El reporte de auditoría se publica con el lanzamiento.

### 15.5 Fuzzing continuo

El parser del DSL y el parser HTTP se someten a fuzzing continuo con `cargo-fuzz`. La CI ejecuta corpus de fuzzing en cada PR; antes del 1.0, se completan al menos 72 horas de fuzzing sin crashes en cada parser.

---

