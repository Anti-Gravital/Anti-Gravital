# Capacidades especializadas: tecnología, finanzas y mercadeo

Responsable: `capacidades-negocio` | Fecha de corte: 2026-11-21

> Regla de oro: toda recomendación lleva **supuestos explícitos, métrica base, fórmula y forma de verificación**. Los números de ejemplo son ilustrativos, nunca evidencia (K-10).

## 1. Tecnología

- **Arquitectura**: límites de módulo, contratos versionados, ADR por decisión estructural, bajo acoplamiento. Anti-patrón: acoplar dominio a infraestructura.
- **APIs e integraciones**: contratos antes de código, idempotencia, versionado, errores tipados, timeouts y reintentos con backoff; nunca exponer secretos.
- **Confiabilidad**: SLO/SLI, error budget, DORA (frecuencia de despliegue, lead time, tasa de fallo, MTTR), benchmarks y postmortem sin culpables.
- **Seguridad**: defensa en capas, mínimo privilegio, fallo cerrado, OWASP ASVS/Top 10, CIS v8.1, NIST CSF 2.0, ISO 27001:2022.
- **Datos**: modelado, índices, migraciones reversibles, calidad de datos, linaje, privacidad por diseño (L-12).
- **IA aplicada**: evaluaciones reproducibles, guardrails, fallback determinista, medición de latencia y coste por inferencia; prompts versionados como archivos.
- **FinOps**: etiquetado de coste, presupuesto por entorno, apagado de recursos ociosos, coste por transacción.

**Checklist de propuesta técnica:** ¿contrato? ¿prueba? ¿rollback? ¿métrica? ¿coste? ¿riesgo? ¿dueño?

## 2. Finanzas

- **Estados financieros (IFRS/NIIF)**: estado de situación, resultado, flujo de efectivo y notas. IFRS 15 (reconocimiento de ingresos), IFRS 9 (instrumentos), IFRS 16 (arrendamientos). Verificar adopción local.
- **Liquidez**: `runway_meses = efectivo / burn_rate_mensual`; `burn_rate = gastos - ingresos` en periodo. Señal de alerta: runway < 6 meses sin financiamiento comprometido.
- **Economía unitaria**: `margen_contribucion = precio - coste_variable`; `LTV = ARPU × margen bruto / churn`; `ratio LTV:CAC` referencia ≈ 3:1; `payback_meses = CAC / margen_contribucion_mensual`.
- **Capital**: NPV = Σ(FCF_t/(1+r)^t) − inversión; TIR (IRR) con cautela en flujos no convencionales; CAPEX vs OPEX.
- **Precios**: coste + margen, valor percibido, competencia, segmentación; probar elasticidad antes de subir.
- **Panamá**: moneda balboa (PAB) en paridad 1:1 con USD; ITBMS general 7% (verificar tasas especiales y vigencia en DGI); precios de transferencia y reportes según norma aplicable.
- **Riesgo**: FX, crédito, liquidez, concentración de clientes; escenarios base/estrés.

**Regla:** ningún modelo financiero se entrega sin supuestos, sensibilidad y fuente de los datos.

## 3. Mercadeo y negocio

- **Embudo (AARRR)**: adquisición → activación → retención → referido → ingresos. Medir cada etapa, no solo "visitas".
- **Segmentación y posicionamiento**: ICP, personas, jobs-to-be-done, propuesta de valor verificable con entrevistas.
- **Canales**: orgánico, pago, socios, contenido, email/eventos; elegir por CAC y calidad, no por moda.
- **Atribución**: first-touch, last-touch, multi-touch; ninguna es verdad absoluta; combinar con experimentos incrementales.
- **Experimentos**: hipótesis, métrica primaria, muestra mínima, significancia estadística, duración fija, decisión pre-registrada. No mirar resultados a diario y decidir a ojo.
- **Retención**: cohortes, churn, RFM, encuestas; retener suele ser más barato que adquirir.
- **Marketing responsable**: consentimiento y privacidad (Ley 81/2019 si aplica), mensajes no engañosos, exclusiones y preferencias del usuario, construcción de marcas sin spam.
- **KPIs**: CAC, LTV, ROAS, CTR, CVR, ARPU, churn, NPS; cada uno con definición y fuente.

**Regla:** una campaña no se lanza sin hipótesis, presupuesto, KPI y plan de parada.

## 4. Formato de recomendación del agente

```
Tema:
Contexto y datos (fuente + fecha):
Supuestos:
Opciones (mínimo 2):
Métrica y baseline:
Coste / riesgo:
Implementación (pasos, dueño, rollback):
Verificación (cómo se sabrá que funcionó):
Incertidumbre y escalado:
```

## 5. Límites

Este agente propone y analiza; **no** decide inversiones, precios finales, campañas con gasto real ni interpretaciones legales sin humano. Las cifras son marcos de trabajo, no pronósticos.
