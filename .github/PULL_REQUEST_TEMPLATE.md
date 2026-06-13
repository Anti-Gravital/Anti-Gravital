<!--
  Esta plantilla se reemplaza automaticamente al abrir la pull request.

  El workflow `.github/workflows/pr-autofill.yml` busca el descriptor
  pre-rellenado en `docs/pr-drafts/<rama-aplanada>.md` (las barras de
  la rama se convierten en guiones) y, si existe, sustituye el cuerpo
  de esta PR por su contenido completo.

  Si abre la PR y este texto sigue visible despues de unos segundos:

    1. Asegurese de que la rama tiene su descriptor en
       `docs/pr-drafts/`. Sin descriptor, la PR no avanza.
    2. Pushee el descriptor a la rama.
    3. Cierre y reabra esta PR (o marquela como ready-for-review).

  Vease `docs/pr-drafts/README.md` para la convencion.

  El titulo de la pull request es la linea `Resumen` del descriptor,
  con un maximo de 256 caracteres.

  El descriptor DEBE incluir una seccion `## Cierre de issues` con
  `Closes #NNN` por cada issue que la PR resuelve (o `Closes: none`).
  GitHub solo auto-cierra issues al fusionar si el cuerpo de la PR usa
  esas palabras clave; una referencia suelta `#NNN` no cierra.
-->
