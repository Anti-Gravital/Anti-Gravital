# RFC-0014: Validacion @regex ejecutable en Rust

## Motivacion

El DSL acepta @regex, pero el generador Rust solo emitia un comentario. Esto hacia que el contrato declarado por el schema no se aplicara durante la ejecucion.

## Problema

La validacion necesita un motor de expresiones regulares, debe rechazar patrones invalidos antes de generar codigo y no debe recompilar el patron en cada llamada a validate().

## Diseno

- Se incorpora regex como dependencia compartida del workspace y dependencia directa de ag-dsl.
- El analisis semantico compila cada patron y produce un error si es invalido.
- El codigo Rust generado usa std::sync::OnceLock<regex::Regex> por campo para compilar cada patron una sola vez.
- Todo proyecto generado que use @regex debe declarar regex = "1" en su Cargo.toml.
- El fixture Rust de CI incluye un campo @regex y pruebas de aceptacion y rechazo.

## Alternativas

Generar validacion manual no cubre la sintaxis completa del DSL. Compilar una Regex en cada llamada es correcto pero introduce trabajo evitable en cada request. Eliminar @regex romperia schemas existentes.

## Riesgos

regex aumenta el grafo de dependencias del compilador y de los consumidores que utilicen la anotacion. OnceLock mantiene una expresion compilada por campo durante la vida del proceso.

## Impacto y compatibilidad

Los schemas validos mantienen su sintaxis. Los patrones malformados, antes aceptados sin ejecucion, pasan a ser errores semanticos. Los proyectos que usen @regex deben anadir la dependencia explicita.

## Rollback

Revertir la dependencia, el chequeo semantico y la emision ejecutable restaura el comportamiento anterior basado en comentarios.
