# Autenticación de Usuarios

## Purpose

Sistema de autenticación con sesiones por token, soporte para OAuth 2.0 y refresh tokens rotativos.

## Requirements

#### Requirement: Registro de usuario

El sistema debe permitir crear cuentas con email y contraseña. La contraseña se almacena con bcrypt (costo 12) y nunca en texto plano.

##### Scenario: Registro exitoso

**Given** un email válido y una contraseña de al menos 8 caracteres
**When** el usuario envía el formulario de registro
**Then** se crea la cuenta y se devuelve un token de sesión

##### Scenario: Email duplicado

**Given** una cuenta existente con el email `a@b.com`
**When** un usuario intenta registrarse con ese email
**Then** el sistema responde `409 Conflict` con el código `EMAIL_TAKEN`

#### Requirement: Inicio de sesión

El sistema debe validar credenciales y emitir un access token de 15 minutos y un refresh token de 30 días.

##### Scenario: Login exitoso

**Given** credenciales válidas
**When** el usuario inicia sesión
**Then** se emiten access token y refresh token

##### Scenario: Bloqueo por intentos fallidos

**Given** 5 intentos de login fallidos en la misma cuenta
**When** el usuario intenta iniciar sesión nuevamente
**Then** la cuenta queda bloqueada por 15 minutos

#### Requirement: Integración OAuth 2.0

El sistema debe soportar autenticación con proveedores OAuth 2.0 (Google y GitHub) mediante flujo de code exchange.

##### Scenario: Login con Google

**Given** el usuario elige "Continuar con Google"
**When** Google devuelve el código de autorización
**Then** el sistema intercambia el código, crea/vincula la cuenta y emite tokens

## Known Limitations

- El bloqueo por intentos fallidos es por cuenta, no por IP.
- El refresh token se invalida por completo si el usuario cambia su contraseña (no se emite re-validación).
- No hay soporte para 2FA (TOTP) en la primera versión.

## Open Questions

- ¿Debe el refresh token expirar a los 30 días o al 90?
- ¿Se requiere soporte para 2FA (TOTP) en la primera versión?
- ¿Los tokens de sesión se revocan al cambiar la contraseña?
