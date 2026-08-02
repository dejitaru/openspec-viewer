# Lapses

## Purpose

Registro de lapsus temporales en el desarrollo (pausas, contexto perdido, bloqueos) para auditar productividad y detectar interrupciones recurrentes.

## Requirements

#### Requirement: Registro manual de lapse

El sistema debe permitir al usuario registrar manualmente un lapse indicando motivo y duración estimada.

##### Scenario: Registrar lapse

**Given** el usuario está trabajando en una tarea
**When** marca "iniciar lapse"
**Then** se crea un registro con timestamp de inicio

##### Scenario: Cerrar lapse

**Given** existe un lapse abierto
**When** el usuario lo cierra
**Then** el registro guarda duración real y motivo

#### Requirement: Reporte semanal

El sistema debe generar un resumen semanal de lapses agregados por motivo.

##### Scenario: Generar reporte

**Given** hay lapses registrados en la semana
**When** el usuario solicita el reporte
**Then** se muestran conteos agregados por motivo

## Known Limitations

- Los lapses se registran solo de forma manual; no hay detección automática de inactividad.

## Open Questions

- ¿Se debe integrar con la detección automática de inactividad del teclado?
- ¿El reporte semanal se envía por email o solo se ve en la app?
