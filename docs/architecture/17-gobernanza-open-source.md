# Capitulo 17. Modelo de gobernanza Open Source

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 17
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [16-rendimiento-y-validacion.md](./16-rendimiento-y-validacion.md)
> Siguiente: [18-riesgos-y-mitigaciones.md](./18-riesgos-y-mitigaciones.md)

## 17. Modelo de gobernanza Open Source

### 17.1 Licencia y promesa

La licencia es Apache 2.0 para todo el ecosistema. No existe ni existirá una versión Enterprise cerrada con features reservadas para clientes pagos. El compromiso es explícito y se documenta en el README. Cualquier cambio de licencia futuro requeriría la aprobación de toda la comunidad de mantenedores, y el ecosistema sigue siendo forkable.

### 17.2 Modelo de mantenimiento

El proyecto adopta un modelo BDFL inicial con plan de transición a meritocracia explícita. En la fase inicial (versiones 0.x), Ángel Nereira es el mantenedor principal. A partir de la versión 1.0, se establece un comité técnico de cinco personas elegidas entre los contribuidores con mayor historial. El comité aprueba RFCs (Request For Comments) para cambios mayores.

### 17.3 RFCs

Cualquier cambio que afecte la API pública, el DSL, la arquitectura de plugins o el modelo de seguridad requiere un RFC. El proceso es: el proponente abre un issue en `anti-gravital-rfcs/`, la comunidad debate por al menos dos semanas, el comité técnico vota. Una vez aprobado, el RFC se mueve a estado "Accepted" y se implementa en una versión específica.

### 17.4 Compatibilidad

Después de la versión 1.0, el proyecto se compromete a semver estricto en la API pública. Breaking changes solo en mayores. Las versiones LTS se anuncian con un calendario público, con al menos 18 meses de soporte de seguridad.

### 17.5 Sostenibilidad económica

El proyecto se sostiene en tres patas. La primera es Gravital Labs (Nereira Technology and Business Solutions), que financia el desarrollo inicial como inversión estratégica. La segunda es servicios profesionales: consultoría de adopción, training y soporte premium para empresas que quieran SLA, sin que esto cierre features del producto. La tercera, a futuro, son sponsors corporativos (GitHub Sponsors, Open Collective) de empresas que dependen del proyecto.

---

