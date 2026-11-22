# Política de seguridad

## Cómo reportar una vulnerabilidad
No abras un issue público. Usa "Report a vulnerability" en la pestaña Security del repositorio (reporte privado) o escribe a contact@angelnereira.com.
Respuesta inicial en 3 días hábiles. Divulgación coordinada tras publicar la corrección.

Los hallazgos que encuentren los agentes se registran con `scripts/kaizen-security report` y **nunca** incluyen el valor de un secreto ni detalles explotables en un issue público (S-12/S-13). Si el repo es público, el reporte se convierte en un aviso mínimo y el detalle va al canal privado.

## Versiones con soporte

| Versión | Soporte |
| --- | --- |
| main | Sí |

## Configuración que debe hacer un humano
1. Secret scanning y push protection activados.
2. Dependabot (alertas) y code scanning (CodeQL).
3. Private vulnerability reporting activado.
4. Protección de `main`: exigir PR, checks `kaizen-gates`, `kaizen-security` y revisión; sin push directo.
5. CODEOWNERS con responsables humanos para los rieles (ya incluido).
6. Workflows de solo lectura por defecto; acciones de terceros fijadas por SHA; sin `pull_request_target` con código del PR.
7. GitHub App o PAT de granularidad fina con caducidad y permisos mínimos; sin credenciales de producción en el entorno del agente.
8. 2FA para todos los mantenedores.
