# 🔍 VERIFICACIÓN DE ESQUEMA DE BASE DE DATOS

**Problema:** El backend intenta usar campos que NO existen en la base de datos real.

---

## 📊 VERIFICAR ESQUEMA ACTUAL

Ejecuta estos comandos en tu servidor para ver el esquema real:

```bash
sudo -u postgres psql -d apolo_billing << 'SQL'

-- Ver esquema de tabla cdrs
\d cdrs

-- Ver esquema de tabla accounts
\d accounts

-- Ver esquema de tabla rate_cards
\d rate_cards

SQL
```

---

## 🎯 ESQUEMAS ESPERADOS

### **Tabla `cdrs` del Motor Rust:**

Según el código Rust, la tabla debería tener:
- `id` (BIGINT)
- `call_uuid` (VARCHAR) ← Motor Rust usa este nombre
- `account_id` (INTEGER)
- `caller_number` (VARCHAR) ← Motor Rust usa este nombre
- `called_number` (VARCHAR) ← Motor Rust usa este nombre
- `start_time` (TIMESTAMP)
- `answer_time` (TIMESTAMP)
- `end_time` (TIMESTAMP)
- `duration` (INTEGER)
- `billsec` (INTEGER)
- `hangup_cause` (VARCHAR)
- `rate_id` (INTEGER)
- `cost` (NUMERIC)
- `direction` (VARCHAR)
- `freeswitch_server_id` (VARCHAR)
- `created_at` (TIMESTAMP)

### **Tabla `accounts`:**

Campos básicos (sin `customer_phone`, `credit_limit`, `currency`):
- `id` (INTEGER)
- `account_number` (VARCHAR)
- `account_name` (VARCHAR)
- `balance` (NUMERIC)
- `account_type` (VARCHAR)
- `status` (VARCHAR)
- `max_concurrent_calls` (INTEGER)
- `created_at` (TIMESTAMP)
- `updated_at` (TIMESTAMP)

---

## ✅ SOLUCIÓN

**El backend debe adaptarse al esquema real de la base de datos, NO al revés.**

Por favor comparte la salida de `\d cdrs` y `\d accounts` para que pueda ajustar los modelos del backend correctamente.

