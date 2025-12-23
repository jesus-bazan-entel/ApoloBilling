# Product Requirements Document (PRD) - Apolo Billing System
## Versión 2.0 - Análisis Completo y Puntos de Mejora

---

## 📋 Tabla de Contenidos
1. [Resumen Ejecutivo](#resumen-ejecutivo)
2. [Arquitectura del Sistema](#arquitectura-del-sistema)
3. [Componentes Principales](#componentes-principales)
4. [Funcionalidades Detalladas](#funcionalidades-detalladas)
5. [Modelos de Datos](#modelos-de-datos)
6. [API Endpoints](#api-endpoints)
7. [Interfaz de Usuario](#interfaz-de-usuario)
8. [Puntos de Mejora Identificados](#puntos-de-mejora-identificados)
9. [Roadmap de Implementación](#roadmap-de-implementación)

---

## 1. Resumen Ejecutivo

### 1.1 Descripción del Sistema
**Apolo Billing** es un sistema completo de tarificación y facturación telefónica en tiempo real, diseñado para integrarse con FreeSWITCH y Cisco CUCM. Combina un motor de billing en Rust de alto rendimiento con una interfaz web administrativa en Python/FastAPI.

### 1.2 Propósito
- **Tarificación en tiempo real** de llamadas telefónicas
- **Gestión de saldos** prepago y postpago
- **Control de anexos** y extensiones telefónicas
- **Generación de CDRs** (Call Detail Records)
- **Reportería financiera** y operativa
- **Integración CUCM** para sincronización de líneas

### 1.3 Usuarios Objetivo
- **Administradores del sistema**: Gestión completa de configuración
- **Personal de facturación**: Consulta de CDRs y reportes financieros
- **Operadores telefónicos**: Monitoreo de llamadas activas
- **Contadores/Finanzas**: Análisis de consumo y auditoría

### 1.4 Tecnologías Principales
- **Backend Principal**: FastAPI (Python 3.x)
- **Motor de Billing**: Rust + Tokio (asíncrono)
- **Base de Datos**: PostgreSQL 13+
- **Cache**: Redis (opcional para motor Rust)
- **Frontend**: Bootstrap 5, jQuery, DataTables
- **Integraciones**: FreeSWITCH ESL, Cisco CUCM AXL

---

## 2. Arquitectura del Sistema

### 2.1 Diagrama de Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│                         USUARIOS                                 │
│  (Navegador Web - Dashboards)                                   │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP/WebSocket
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    FastAPI Application                           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ • Autenticación (fastapi-login)                           │  │
│  │ • Gestión de Anexos, Zonas, Tarifas                      │  │
│  │ • Dashboards (Saldo, CDR, Finanzas, Monitoreo)           │  │
│  │ • WebSocket (Llamadas en tiempo real)                    │  │
│  │ • Exportación (PDF, Excel)                               │  │
│  │ • CUCM Integration                                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────┬───────────────────────┬────────────────────────┘
                 │                       │
                 ▼                       ▼
    ┌───────────────────────┐   ┌──────────────────────┐
    │   PostgreSQL DB       │   │   Rust Billing       │
    │   (apolobilling)      │   │   Engine             │
    │                       │   │  (Optional)          │
    │ • Accounts            │   │                      │
    │ • CDRs                │   │ • Real-time Rating   │
    │ • Rates/Zones         │   │ • Balance Reserve    │
    │ • Active Calls        │   │ • FreeSWITCH ESL     │
    │ • Anexos              │   └──────────┬───────────┘
    │ • FAC Codes           │              │
    │ • Audit Logs          │              │ Redis
    └───────────┬───────────┘              ▼ (Cache)
                │                   ┌─────────────┐
                │                   │   Redis     │
                │                   │  (Sessions) │
                └───────────────────┤             │
                                    └─────────────┘
                                           
┌─────────────────────────────────────────────────────────────────┐
│                    INTEGRACIONES                                 │
├─────────────────────────────────────────────────────────────────┤
│  FreeSWITCH (ESL)          Cisco CUCM (AXL/SOAP)               │
│  • CHANNEL_CREATE          • Sync Lines                         │
│  • CHANNEL_ANSWER          • Sync Extensions                    │
│  • CHANNEL_HANGUP          • Device Configuration               │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Capas del Sistema

#### 2.2.1 Capa de Presentación (Frontend)
- **Templates Jinja2**: 33 plantillas HTML
- **CSS Framework**: Bootstrap 5.3.2
- **JavaScript**: jQuery, DataTables, WebSocket API
- **Iconografía**: Bootstrap Icons, Font Awesome

#### 2.2.2 Capa de Aplicación (Backend Python)
- **Archivo principal**: `main.py` (6,777 líneas)
- **Total de endpoints**: 80+ rutas REST
- **WebSocket**: Tiempo real para llamadas activas
- **Autenticación**: Cookie-based con fastapi-login

#### 2.2.3 Capa de Motor de Billing (Rust)
- **Ubicación**: `/rust-billing-engine/`
- **Características**:
  - Procesamiento asíncrono con Tokio
  - Autorización de llamadas
  - Reserva de saldo en tiempo real
  - Generación de CDRs
- **Base de datos**: PostgreSQL con SQLx
- **Cache**: Redis para estado de sesiones

#### 2.2.4 Capa de Datos
- **PostgreSQL**: Base de datos principal
- **Tablas principales**: 20+ tablas
- **SQLAlchemy ORM**: Para operaciones Python
- **Migraciones**: Scripts de inicialización

---

## 3. Componentes Principales

### 3.1 Gestión de Anexos (Extensions)

**Descripción**: Módulo para administrar extensiones telefónicas internas.

**Funcionalidades**:
- ✅ CRUD completo de anexos
- ✅ Búsqueda y filtrado
- ✅ Carga masiva desde CSV/Excel
- ✅ Generación de PINes automáticos
- ✅ Configuración de longitud de PIN
- ✅ Asociación con áreas/departamentos

**Endpoints**:
- `GET /dashboard/anexos` - Listar anexos
- `POST /anexo` - Crear anexo
- `PUT /anexo/{id}` - Actualizar anexo
- `DELETE /anexo/{id}` - Eliminar anexo
- `POST /dashboard/anexos/carga_masiva` - Importar masivo
- `POST /anexos/generar_pines` - Generar PINes

**Modelo de Datos**:
```python
class Anexo(Base):
    __tablename__ = "anexos"
    id = Column(Integer, primary_key=True)
    numero = Column(String(50), unique=True)
    area = Column(String(100))
    nombre = Column(String(200))
    pin = Column(String(20))
    saldo_inicial = Column(Numeric(10, 2), default=0)
    fecha_creacion = Column(DateTime, default=datetime.now)
```

### 3.2 Gestión de Saldos y Recargas

**Descripción**: Control de crédito prepago para anexos y cuentas.

**Funcionalidades**:
- ✅ Consulta de saldo individual
- ✅ Recarga manual (individual)
- ✅ Recarga masiva desde Excel
- ✅ Histórico de recargas
- ✅ Auditoría de operaciones

**Endpoints**:
- `GET /dashboard/saldo` - Dashboard de saldos
- `POST /recargar/{calling_number}/{amount}` - Recarga manual
- `GET /dashboard/recargas` - Histórico
- `POST /dashboard/recarga_masiva` - Recarga masiva

**Modelo de Datos**:
```python
class SaldoAnexo(Base):
    __tablename__ = "saldo_anexos"
    id = Column(Integer, primary_key=True)
    anexo_id = Column(Integer, ForeignKey('anexos.id'))
    saldo_actual = Column(Numeric(10, 2))
    ultima_actualizacion = Column(DateTime)

class Recarga(Base):
    __tablename__ = "recargas"
    id = Column(Integer, primary_key=True)
    anexo_id = Column(Integer, ForeignKey('anexos.id'))
    monto = Column(Numeric(10, 2))
    fecha = Column(DateTime, default=datetime.now)
    usuario = Column(String(100))
    metodo = Column(String(50))  # manual, masiva
```

### 3.3 Tarificación (Rating Engine)

**Descripción**: Sistema de tarifas por destino y zona horaria.

**Funcionalidades**:
- ✅ Gestión de zonas geográficas
- ✅ Gestión de prefijos telefónicos
- ✅ Tarifas por zona y franja horaria
- ✅ Validación de tarifas duplicadas
- ✅ Rate cards con vigencia temporal

**Endpoints**:
- `GET /dashboard/zonas` - Gestión de zonas
- `GET /dashboard/prefijos` - Gestión de prefijos
- `GET /dashboard/tarifas` - Gestión de tarifas
- `POST /api/zonas` - Crear zona
- `POST /api/prefijos` - Crear prefijo
- `POST /api/tarifas` - Crear tarifa

**Modelos de Datos**:
```python
class Zona(Base):
    __tablename__ = "zonas"
    id = Column(Integer, primary_key=True)
    nombre = Column(String(100), unique=True)
    descripcion = Column(String(500))

class Prefijo(Base):
    __tablename__ = "prefijos"
    id = Column(Integer, primary_key=True)
    prefijo = Column(String(20), unique=True)
    zona_id = Column(Integer, ForeignKey('zonas.id'))
    descripcion = Column(String(200))

class Tarifa(Base):
    __tablename__ = "tarifas"
    id = Column(Integer, primary_key=True)
    zona_id = Column(Integer, ForeignKey('zonas.id'))
    costo_minuto = Column(Numeric(10, 4))
    costo_conexion = Column(Numeric(10, 4))
    franja_horaria = Column(String(50))  # normal, nocturna, feriado

class RateCard(Base):
    __tablename__ = "rate_cards"
    id = Column(Integer, primary_key=True)
    prefix = Column(String(20))
    destination = Column(String(200))
    rate = Column(Numeric(10, 4))
    effective_start = Column(DateTime)
    effective_end = Column(DateTime, nullable=True)
```

### 3.4 CDR (Call Detail Records)

**Descripción**: Registro detallado de todas las llamadas procesadas.

**Funcionalidades**:
- ✅ Captura automática de CDRs desde FreeSWITCH
- ✅ Dashboard de consulta con filtros avanzados
- ✅ Exportación a PDF y Excel
- ✅ CDRs rechazados (llamadas no autorizadas)
- ✅ Estadísticas por zona y anexo

**Endpoints**:
- `GET /dashboard/cdr` - Dashboard CDR
- `POST /cdr` - Recibir CDR desde FreeSWITCH
- `POST /cdr/rejected` - CDR rechazado
- `GET /export/cdr/pdf` - Exportar PDF
- `GET /export/cdr/excel` - Exportar Excel

**Modelo de Datos**:
```python
class CDR(Base):
    __tablename__ = "cdrs"
    id = Column(Integer, primary_key=True)
    call_id = Column(String(100))
    calling_number = Column(String(50))
    called_number = Column(String(50))
    direction = Column(String(20))  # inbound, outbound, internal
    start_time = Column(DateTime)
    answer_time = Column(DateTime)
    end_time = Column(DateTime)
    duration = Column(Integer)  # segundos totales
    billsec = Column(Integer)  # segundos tarificados
    hangup_cause = Column(String(50))
    cost = Column(Numeric(10, 4))
    zone = Column(String(100))
    prefix_matched = Column(String(20))
```

### 3.5 Llamadas Activas (Real-time Monitoring)

**Descripción**: Monitoreo en tiempo real de llamadas en curso.

**Funcionalidades**:
- ✅ Visualización en tiempo real vía WebSocket
- ✅ Duración y costo actualizado cada segundo
- ✅ Filtrado por dirección (entrante/saliente/interna)
- ✅ Terminación manual de llamadas (en desarrollo)
- ✅ Estadísticas de conexiones WebSocket

**Endpoints**:
- `GET /dashboard/monitoreo` - Dashboard de monitoreo
- `GET /api/active-calls` - Lista de llamadas activas (REST)
- `WebSocket /ws` - Stream de actualizaciones
- `POST /api/active-calls` - Simular llamada activa
- `DELETE /api/active-calls/{call_id}` - Terminar llamada

**Modelo de Datos**:
```sql
CREATE TABLE active_calls (
    id SERIAL PRIMARY KEY,
    call_id VARCHAR(100) UNIQUE,
    calling_number VARCHAR(50),
    called_number VARCHAR(50),
    direction VARCHAR(20),
    start_time TIMESTAMP,
    current_duration INTEGER DEFAULT 0,
    current_cost NUMERIC(10, 4) DEFAULT 0,
    zone VARCHAR(100),
    connection_id VARCHAR(100)
);
```

### 3.6 Integración CUCM (Cisco Unified Communications Manager)

**Descripción**: Sincronización bidireccional con Cisco CUCM vía AXL API.

**Funcionalidades**:
- ✅ Configuración de credenciales CUCM
- ✅ Test de conexión SOAP/AXL
- ✅ Sincronización de líneas y extensiones
- ✅ Control de servicio (start/stop/status)
- ✅ Logs del listener Java
- ✅ Dashboard de estado

**Endpoints**:
- `GET /dashboard/cucm` - Dashboard CUCM
- `GET /api/cucm/config` - Obtener configuración
- `POST /api/cucm/config` - Guardar configuración
- `POST /api/cucm/test_connection` - Probar conexión
- `GET /api/cucm/status` - Estado del servicio
- `POST /api/cucm/service/{action}` - start/stop/restart
- `GET /api/cucm/logs` - Ver logs

**Modelo de Datos**:
```python
class CucmConfig(Base):
    __tablename__ = "cucm_config"
    id = Column(Integer, primary_key=True)
    host = Column(String(200))
    username = Column(String(100))
    password = Column(String(200))  # Encriptado
    version = Column(String(20))
    enabled = Column(Boolean, default=False)
    last_sync = Column(DateTime)
```

**Componente Java**:
- **Ubicación**: `/java_listener/`
- **Función**: Listener de eventos CUCM
- **Tecnología**: Java + AXL SOAP API

### 3.7 FAC (Forced Authorization Codes)

**Descripción**: Códigos de autorización para control de llamadas.

**Funcionalidades**:
- ✅ Gestión de códigos FAC
- ✅ Sincronización con CUCM
- ✅ Dashboard de PINes
- ✅ Histórico de sincronizaciones

**Endpoints**:
- `GET /dashboard/pines` - Dashboard PINes/FAC
- `GET /dashboard/fac` - Dashboard FAC
- `GET /dashboard/fac_historial` - Histórico
- `GET /dashboard/fac_sync` - Estado de sincronización

**Modelo de Datos**:
```python
class FacCode(Base):
    __tablename__ = "fac_codes"
    id = Column(Integer, primary_key=True)
    code = Column(String(20), unique=True)
    anexo_id = Column(Integer, ForeignKey('anexos.id'))
    enabled = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.now)
    synced_to_cucm = Column(Boolean, default=False)
```

### 3.8 Auditoría y Seguridad

**Descripción**: Registro de todas las operaciones administrativas.

**Funcionalidades**:
- ✅ Log de todas las modificaciones
- ✅ Dashboard de auditoría
- ✅ Filtros por usuario, acción, fecha
- ✅ Trazabilidad completa

**Endpoints**:
- `GET /dashboard/auditoria` - Dashboard de auditoría

**Modelo de Datos**:
```python
class Auditoria(Base):
    __tablename__ = "auditoria"
    id = Column(Integer, primary_key=True)
    usuario = Column(String(100))
    accion = Column(String(50))  # crear, actualizar, eliminar
    tabla = Column(String(50))
    registro_id = Column(Integer)
    detalles = Column(String(1000))
    timestamp = Column(DateTime, default=datetime.now)
```

### 3.9 Reportes Financieros

**Descripción**: Análisis financiero y estadístico del sistema.

**Funcionalidades**:
- ✅ Dashboard de finanzas
- ✅ Ranking de consumo por anexo
- ✅ Estadísticas por zona
- ✅ Consumo por área/departamento
- ✅ Exportación a PDF y Excel

**Endpoints**:
- `GET /dashboard/finanzas` - Dashboard financiero
- `GET /dashboard/ranking_consumo` - Ranking de anexos
- `GET /dashboard/estadisticas_zona` - Estadísticas por zona
- `GET /export/consumo_zona/pdf` - Exportar estadísticas

### 3.10 Autenticación y Gestión de Usuarios

**Descripción**: Sistema de login y control de acceso.

**Funcionalidades**:
- ✅ Login con usuario/contraseña
- ✅ Cookies de sesión
- ✅ Hash de contraseñas (bcrypt)
- ✅ Logout seguro
- ✅ Protección de rutas

**Endpoints**:
- `GET /login` - Página de login
- `POST /auth/login` - Autenticar usuario
- `GET /logout` - Cerrar sesión
- `GET /` - Redirect a dashboard

**Modelo de Datos**:
```python
class Usuario(Base):
    __tablename__ = "usuarios"
    id = Column(Integer, primary_key=True)
    username = Column(String(100), unique=True)
    password_hash = Column(String(200))
    email = Column(String(200))
    role = Column(String(50))  # admin, operator, viewer
    created_at = Column(DateTime, default=datetime.now)
    last_login = Column(DateTime)
```

---

## 4. Funcionalidades Detalladas

### 4.1 Matriz de Funcionalidades

| Módulo | Funcionalidad | Estado | Prioridad | Complejidad |
|--------|--------------|--------|-----------|-------------|
| **Anexos** | CRUD básico | ✅ Completo | Alta | Baja |
| | Búsqueda avanzada | ✅ Completo | Media | Baja |
| | Carga masiva | ✅ Completo | Alta | Media |
| | Generación PINes | ✅ Completo | Alta | Media |
| | Exportación | ⚠️ Parcial | Media | Baja |
| **Saldos** | Consulta individual | ✅ Completo | Alta | Baja |
| | Recarga manual | ✅ Completo | Alta | Baja |
| | Recarga masiva | ✅ Completo | Alta | Media |
| | Alertas de saldo bajo | ❌ Pendiente | Media | Media |
| | Dashboard gráfico | ⚠️ Parcial | Media | Media |
| **Tarifas** | Gestión zonas | ✅ Completo | Alta | Baja |
| | Gestión prefijos | ✅ Completo | Alta | Media |
| | Tarifas por franja | ✅ Completo | Alta | Alta |
| | Validación duplicados | ✅ Completo | Alta | Media |
| | Rate cards temporales | ✅ Completo | Media | Alta |
| | Importación masiva | ❌ Pendiente | Media | Media |
| **CDR** | Captura automática | ✅ Completo | Alta | Alta |
| | Dashboard consulta | ✅ Completo | Alta | Media |
| | Filtros avanzados | ✅ Completo | Alta | Media |
| | Exportación PDF | ✅ Completo | Media | Baja |
| | Exportación Excel | ✅ Completo | Media | Baja |
| | CDRs rechazados | ✅ Completo | Media | Media |
| | Estadísticas | ✅ Completo | Media | Media |
| **Llamadas Activas** | WebSocket real-time | ✅ Completo | Alta | Alta |
| | Dashboard monitoreo | ✅ Completo | Alta | Media |
| | Terminación manual | ⚠️ En desarrollo | Media | Alta |
| | Alertas duración | ❌ Pendiente | Baja | Media |
| **CUCM** | Configuración | ✅ Completo | Alta | Media |
| | Test conexión | ✅ Completo | Alta | Baja |
| | Sincronización | ✅ Completo | Alta | Alta |
| | Control servicio | ✅ Completo | Media | Media |
| | Logs viewer | ✅ Completo | Baja | Baja |
| **FAC** | Gestión códigos | ✅ Completo | Media | Baja |
| | Sincronización CUCM | ✅ Completo | Media | Media |
| | Dashboard | ✅ Completo | Baja | Baja |
| **Finanzas** | Dashboard general | ✅ Completo | Alta | Media |
| | Ranking consumo | ✅ Completo | Media | Baja |
| | Estadísticas zona | ✅ Completo | Media | Media |
| | Gráficos avanzados | ⚠️ Parcial | Media | Alta |
| | Proyecciones | ❌ Pendiente | Baja | Alta |
| **Auditoría** | Log automático | ✅ Completo | Alta | Media |
| | Dashboard consulta | ✅ Completo | Media | Baja |
| | Filtros | ✅ Completo | Media | Baja |
| | Alertas | ❌ Pendiente | Baja | Media |
| **Autenticación** | Login/Logout | ✅ Completo | Alta | Baja |
| | Roles/Permisos | ⚠️ Básico | Alta | Alta |
| | 2FA | ❌ Pendiente | Media | Alta |
| | Recuperación password | ❌ Pendiente | Media | Media |

**Leyenda**:
- ✅ Completo: Funcionalidad implementada y operativa
- ⚠️ Parcial: Funcionalidad implementada parcialmente
- ❌ Pendiente: Funcionalidad no implementada

---

## 5. Modelos de Datos

### 5.1 Diagrama Entidad-Relación

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Usuario    │         │    Anexo     │         │     Zona     │
├──────────────┤         ├──────────────┤         ├──────────────┤
│ id (PK)      │         │ id (PK)      │         │ id (PK)      │
│ username     │         │ numero       │         │ nombre       │
│ password_hash│         │ area         │         │ descripcion  │
│ email        │         │ nombre       │         └──────┬───────┘
│ role         │         │ pin          │                │
└──────────────┘         │ saldo_inicial│                │
                         └──────┬───────┘                │
                                │                        │
                                │ 1:1                    │ 1:N
                                ▼                        ▼
                         ┌──────────────┐         ┌──────────────┐
                         │ SaldoAnexo   │         │   Prefijo    │
                         ├──────────────┤         ├──────────────┤
                         │ id (PK)      │         │ id (PK)      │
                         │ anexo_id (FK)│         │ prefijo      │
                         │ saldo_actual │         │ zona_id (FK) │
                         └──────────────┘         │ descripcion  │
                                                  └──────┬───────┘
                                                         │
                         ┌──────────────┐                │
                         │   Recarga    │                │
                         ├──────────────┤                │ 1:N
                         │ id (PK)      │                ▼
                         │ anexo_id (FK)│         ┌──────────────┐
                         │ monto        │         │   Tarifa     │
                         │ fecha        │         ├──────────────┤
                         │ usuario      │         │ id (PK)      │
                         └──────────────┘         │ zona_id (FK) │
                                                  │ costo_minuto │
                                                  │ costo_conexion│
┌──────────────┐                                 │ franja_horaria│
│     CDR      │                                 └──────────────┘
├──────────────┤
│ id (PK)      │                                 ┌──────────────┐
│ call_id      │                                 │  RateCard    │
│ calling_num  │                                 ├──────────────┤
│ called_num   │                                 │ id (PK)      │
│ direction    │                                 │ prefix       │
│ start_time   │                                 │ destination  │
│ duration     │                                 │ rate         │
│ cost         │                                 │ effective_*  │
│ zone         │                                 └──────────────┘
└──────────────┘

┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ ActiveCall   │         │  CucmConfig  │         │   FacCode    │
├──────────────┤         ├──────────────┤         ├──────────────┤
│ id (PK)      │         │ id (PK)      │         │ id (PK)      │
│ call_id      │         │ host         │         │ code         │
│ calling_num  │         │ username     │         │ anexo_id (FK)│
│ called_num   │         │ password     │         │ enabled      │
│ direction    │         │ version      │         │ synced_cucm  │
│ start_time   │         │ enabled      │         └──────────────┘
│ current_dur  │         └──────────────┘
│ current_cost │
│ zone         │         ┌──────────────┐
└──────────────┘         │  Auditoria   │
                         ├──────────────┤
                         │ id (PK)      │
                         │ usuario      │
                         │ accion       │
                         │ tabla        │
                         │ registro_id  │
                         │ detalles     │
                         │ timestamp    │
                         └──────────────┘
```

### 5.2 Descripción de Tablas

| Tabla | Descripción | Registros Típicos | Índices Clave |
|-------|-------------|-------------------|---------------|
| `usuarios` | Cuentas de acceso al sistema | 5-50 | username |
| `anexos` | Extensiones telefónicas | 100-10,000 | numero, area |
| `saldo_anexos` | Saldo actual de cada anexo | 100-10,000 | anexo_id |
| `recargas` | Histórico de recargas | 1,000-1,000,000 | anexo_id, fecha |
| `zonas` | Zonas geográficas de tarificación | 10-200 | nombre |
| `prefijos` | Prefijos telefónicos | 100-50,000 | prefijo, zona_id |
| `tarifas` | Tarifas por zona y franja | 100-1,000 | zona_id, franja |
| `rate_cards` | Tarifas detalladas por destino | 1,000-100,000 | prefix |
| `cdrs` | Registros de llamadas | 10,000-10,000,000+ | call_id, fecha, anexo |
| `active_calls` | Llamadas en curso | 0-1,000 | call_id |
| `cucm_config` | Configuración CUCM | 1 | - |
| `fac_codes` | Códigos de autorización | 100-10,000 | code |
| `auditoria` | Log de auditoría | 10,000-1,000,000+ | usuario, fecha |

---

## 6. API Endpoints

### 6.1 Resumen de Endpoints

**Total de endpoints**: 80+

**Distribución por categoría**:
- Autenticación: 3 endpoints
- Anexos: 8 endpoints
- Saldos y Recargas: 5 endpoints
- Tarifas (Zonas/Prefijos): 10 endpoints
- CDR: 6 endpoints
- Llamadas Activas: 6 endpoints
- CUCM: 7 endpoints
- FAC/PINes: 4 endpoints
- Finanzas: 5 endpoints
- Auditoría: 2 endpoints
- Configuración: 3 endpoints
- Dashboards: 15 endpoints
- Exportación: 6 endpoints

### 6.2 Endpoints Críticos

#### 6.2.1 Autenticación
```
POST /auth/login
GET  /login
GET  /logout
```

#### 6.2.2 Balance Check (para FreeSWITCH)
```
GET  /check_balance/{calling_number}/{called_number}
GET  /check_balance_for_call/{calling_number}/{called_number}
POST /recargar/{calling_number}/{amount}
```

#### 6.2.3 CDR Reception (desde FreeSWITCH)
```
POST /cdr
POST /cdr/rejected
```

#### 6.2.4 Active Calls
```
GET       /api/active-calls
GET       /api/active-calls-list
WebSocket /ws
POST      /api/active-calls
DELETE    /api/active-calls/{call_id}
```

#### 6.2.5 CUCM Integration
```
GET  /api/cucm/config
POST /api/cucm/config
POST /api/cucm/test_connection
GET  /api/cucm/status
POST /api/cucm/service/{action}  # start, stop, restart
GET  /api/cucm/logs
```

### 6.3 Seguridad de Endpoints

**Endpoints públicos** (sin autenticación):
- `GET /login`
- `POST /auth/login`
- `POST /cdr` (desde FreeSWITCH)
- `GET /check_balance/{calling_number}/{called_number}` (FreeSWITCH)

**Endpoints protegidos** (requieren autenticación):
- Todos los dashboards (`/dashboard/*`)
- Todas las operaciones CRUD
- Todas las exportaciones

**Mecanismo de protección**:
```python
@manager.user_loader()
def load_user(username: str):
    # Cargar usuario desde BD
    pass

# Proteger rutas
@app.get("/dashboard/saldo")
async def dashboard_saldo(user=Depends(manager)):
    # Solo usuarios autenticados
    pass
```

---

## 7. Interfaz de Usuario

### 7.1 Estructura de Templates

**Total de templates**: 33 archivos HTML

**Template base**: `base.html`
- Menú lateral (sidebar) con navegación
- Header con información de usuario
- Footer con copyright
- Carga de CSS/JS común

### 7.2 Dashboards Principales

| Dashboard | Ruta | Descripción | Componentes Clave |
|-----------|------|-------------|-------------------|
| **Saldo** | `/dashboard/saldo` | Vista general de saldos | Tabla de anexos, búsqueda, filtros |
| **CDR** | `/dashboard/cdr` | Consulta de llamadas | Filtros fecha/anexo, paginación |
| **Finanzas** | `/dashboard/finanzas` | Resumen financiero | Gráficos, totales, ranking |
| **Anexos** | `/dashboard/anexos` | Gestión de extensiones | CRUD completo, carga masiva |
| **Monitoreo** | `/dashboard/monitoreo` | Llamadas en tiempo real | WebSocket, lista dinámica |
| **Tarifas** | `/dashboard/tarifas` | Configuración de tarifas | Tablas de zonas/prefijos/tarifas |
| **CUCM** | `/dashboard/cucm` | Integración Cisco | Estado, configuración, logs |
| **Auditoría** | `/dashboard/auditoria` | Log de operaciones | Filtros, búsqueda, timeline |
| **Recargas** | `/dashboard/recargas` | Histórico de recargas | Tabla filtrable, exportación |

### 7.3 Navegación del Sistema

**Menú Sidebar**:
```
🏠 Dashboard Principal
   └─ Saldo
   
📊 Operaciones
   ├─ CDRs
   ├─ Llamadas Activas
   ├─ Finanzas
   └─ Ranking de Consumo
   
📞 Configuración
   ├─ Anexos
   ├─ Zonas
   ├─ Prefijos
   ├─ Tarifas
   └─ PINes/FAC
   
💰 Recargas
   ├─ Recarga Manual
   └─ Recarga Masiva
   
🔧 Integraciones
   └─ Cisco CUCM
   
📋 Auditoría y Logs
   └─ Auditoría

🚪 Cerrar Sesión
```

### 7.4 Experiencia de Usuario

**Características de UX**:
- ✅ Diseño responsivo (Bootstrap 5)
- ✅ Tablas interactivas (DataTables)
- ✅ Búsqueda en tiempo real
- ✅ Modales para formularios
- ✅ Notificaciones toast/alert
- ✅ WebSocket para updates en tiempo real
- ✅ Exportación a PDF/Excel
- ⚠️ Gráficos limitados (Chart.js no implementado completamente)
- ❌ Sin modo oscuro
- ❌ Sin internacionalización (i18n)

### 7.5 Componentes Reutilizables

**Modales**:
- Modal de nuevo anexo
- Modal de configuración PIN
- Modal de recarga manual
- Modal de confirmación eliminación

**Tablas**:
- Tabla de anexos (búsqueda, paginación, ordenamiento)
- Tabla de CDRs (filtros avanzados, exportación)
- Tabla de recargas (filtros de fecha)
- Tabla de auditoría (filtros por usuario)

**Formularios**:
- Login form
- Formulario de anexo (crear/editar)
- Formulario de zona/prefijo/tarifa
- Formulario de recarga
- Formulario de configuración CUCM

---

## 8. Puntos de Mejora Identificados

### 8.1 🔴 CRÍTICOS (Alta Prioridad)

#### 8.1.1 Seguridad y Autenticación
**Problema**: Sistema de roles y permisos básico, sin control granular.

**Impacto**: Riesgo de acceso no autorizado a funciones críticas.

**Recomendaciones**:
1. Implementar RBAC (Role-Based Access Control) completo
   - Roles: `super_admin`, `admin`, `operator`, `viewer`, `accountant`
   - Permisos granulares por endpoint
2. Agregar autenticación de dos factores (2FA)
3. Implementar recuperación de contraseña
4. Agregar límite de intentos de login (rate limiting)
5. Logs de acceso y sesiones
6. Tokens JWT con expiración configurable

**Estimación**: 2-3 semanas

#### 8.1.2 Validación y Sanitización de Datos
**Problema**: Validación insuficiente en varios endpoints, uso de SQL directo en algunos casos.

**Impacto**: Riesgo de inyección SQL, XSS, y corrupción de datos.

**Recomendaciones**:
1. Reemplazar todos los `text()` con consultas ORM
2. Validar todos los inputs con Pydantic models
3. Sanitizar datos antes de almacenar
4. Implementar CSRF protection
5. Validar tipos de archivo en cargas masivas
6. Agregar límites de tamaño de archivo

**Estimación**: 2 semanas

#### 8.1.3 Manejo de Errores y Excepciones
**Problema**: Manejo inconsistente de errores, algunos endpoints retornan 200 con error en JSON.

**Impacto**: Dificulta debugging, mala experiencia de usuario.

**Recomendaciones**:
1. Estandarizar respuestas de error con códigos HTTP apropiados
2. Implementar exception handlers globales
3. Crear middleware de logging estructurado
4. Agregar IDs de correlación para trazabilidad
5. Implementar retry logic para operaciones críticas

**Estimación**: 1 semana

#### 8.1.4 Performance y Escalabilidad
**Problema**: Consultas sin índices, falta de paginación en algunas vistas, WebSocket sin autenticación.

**Impacto**: Rendimiento degradado con alta carga, posible saturación de recursos.

**Recomendaciones**:
1. Agregar índices a tablas principales:
   ```sql
   CREATE INDEX idx_cdrs_calling_num ON cdrs(calling_number);
   CREATE INDEX idx_cdrs_called_num ON cdrs(called_number);
   CREATE INDEX idx_cdrs_start_time ON cdrs(start_time);
   CREATE INDEX idx_prefijos_prefijo ON prefijos(prefijo);
   ```
2. Implementar paginación server-side en todas las listas
3. Agregar caché Redis para:
   - Tarifas frecuentes
   - Configuración del sistema
   - Datos de referencia (zonas, prefijos)
4. Implementar rate limiting por usuario
5. Optimizar consultas con JOINs pesados

**Estimación**: 2 semanas

#### 8.1.5 Concurrencia en Llamadas Activas
**Problema**: Posibles race conditions en actualización de saldo durante llamadas simultáneas.

**Impacto**: Inconsistencia de datos, sobregiro de saldo.

**Recomendaciones**:
1. Implementar locks de base de datos para actualizaciones de saldo
2. Usar transacciones ACID para operaciones críticas
3. Implementar versionado optimista (row versioning)
4. Agregar validación de saldo con FOR UPDATE
5. Considerar usar el motor Rust para operaciones críticas

**Estimación**: 1-2 semanas

### 8.2 🟡 IMPORTANTES (Media Prioridad)

#### 8.2.1 Testing y Calidad de Código
**Problema**: Ausencia de tests automatizados, código monolítico en `main.py` (6,777 líneas).

**Impacto**: Dificulta mantenimiento y refactorización, alto riesgo de regresiones.

**Recomendaciones**:
1. Refactorizar `main.py` en módulos:
   ```
   app/
   ├── routers/
   │   ├── auth.py
   │   ├── anexos.py
   │   ├── cdr.py
   │   ├── tarifas.py
   │   └── cucm.py
   ├── models/
   ├── schemas/
   ├── services/
   └── utils/
   ```
2. Implementar tests unitarios (pytest)
3. Implementar tests de integración
4. Agregar CI/CD pipeline
5. Code coverage mínimo del 70%
6. Linting (flake8, black, mypy)

**Estimación**: 3-4 semanas

#### 8.2.2 Logging y Monitoreo
**Problema**: Logging inconsistente, falta de métricas de sistema.

**Impacto**: Dificulta troubleshooting y análisis de incidentes.

**Recomendaciones**:
1. Implementar logging estructurado (JSON logs)
2. Niveles de log apropiados (DEBUG, INFO, WARNING, ERROR)
3. Integrar con herramienta de monitoreo (Prometheus, Grafana)
4. Métricas de negocio:
   - Llamadas por segundo
   - Tasa de éxito/fallo
   - Latencia de tarificación
   - Saldo promedio
5. Alertas automáticas para:
   - Errores críticos
   - Saldo bajo
   - Servicios caídos

**Estimación**: 2 semanas

#### 8.2.3 Documentación
**Problema**: Documentación técnica limitada, falta de documentación de API.

**Impacto**: Curva de aprendizaje alta para nuevos desarrolladores.

**Recomendaciones**:
1. Generar documentación OpenAPI/Swagger automática
2. Documentar arquitectura con diagramas actualizados
3. Crear guía de deployment
4. Manual de usuario (screenshots, videos)
5. Documentar integraciones (FreeSWITCH, CUCM)
6. Changelog con versiones

**Estimación**: 1-2 semanas

#### 8.2.4 UI/UX Improvements
**Problema**: Gráficos limitados, falta de dashboards visuales, sin modo oscuro.

**Impacto**: Experiencia de usuario subóptima, análisis visual difícil.

**Recomendaciones**:
1. Implementar gráficos interactivos (Chart.js o D3.js):
   - Consumo por hora del día
   - Tendencias de llamadas
   - Top destinos
   - Evolución de saldo
2. Dashboard ejecutivo con KPIs
3. Modo oscuro/claro
4. Notificaciones push (Web Push API)
5. Exportación de dashboards a PDF
6. Mejoras de accesibilidad (WCAG 2.1)

**Estimación**: 3 semanas

#### 8.2.5 Configuración Externalizada
**Problema**: Configuración hardcoded en varios lugares, archivo `.env` no utilizado completamente.

**Impacto**: Dificulta deployment en múltiples ambientes.

**Recomendaciones**:
1. Centralizar configuración en Pydantic Settings
2. Variables de entorno para todos los parámetros críticos
3. Configuración por ambiente (dev, staging, prod)
4. Secrets management (HashiCorp Vault o AWS Secrets Manager)
5. Feature flags para funcionalidades experimentales

**Estimación**: 1 semana

### 8.3 🟢 MEJORAS OPCIONALES (Baja Prioridad)

#### 8.3.1 Funcionalidades Nuevas
1. **Alertas y Notificaciones**:
   - Email alerts para saldo bajo
   - SMS notifications
   - Webhook notifications
   
2. **Reportes Avanzados**:
   - Reportes personalizables
   - Scheduled reports (cron)
   - Report builder UI
   
3. **API Pública**:
   - REST API documentada para terceros
   - API Keys management
   - Rate limiting por API key
   
4. **Mobile App**:
   - App móvil para consulta de saldo
   - Push notifications
   - QR codes para PINes

5. **Machine Learning**:
   - Detección de fraude
   - Predicción de consumo
   - Optimización de tarifas

**Estimación**: Variable (2-12 semanas por funcionalidad)

#### 8.3.2 Internacionalización (i18n)
**Problema**: Sistema solo en español.

**Recomendaciones**:
1. Implementar i18n con gettext o Flask-Babel
2. Soporte para inglés, español, portugués
3. Formato de fechas localizado
4. Monedas múltiples

**Estimación**: 2 semanas

#### 8.3.3 Backup y Disaster Recovery
**Problema**: No hay estrategia documentada de backup.

**Recomendaciones**:
1. Backups automáticos de PostgreSQL
2. Replicación de base de datos
3. Plan de disaster recovery documentado
4. Pruebas periódicas de restore

**Estimación**: 1 semana

### 8.4 🔧 DEUDA TÉCNICA

#### 8.4.1 Código Duplicado
**Ubicaciones identificadas**:
- Lógica de búsqueda de tarifas repetida
- Validaciones de saldo duplicadas
- Manejo de sesión de BD redundante

**Recomendación**: Extraer a funciones/clases reutilizables

#### 8.4.2 Dependencias Obsoletas
**Verificar actualizaciones**:
```bash
cd /home/user/webapp && pip list --outdated
```

**Recomendación**: Actualizar dependencias regularmente, mantener requirements.txt limpio

#### 8.4.3 Código Comentado
**Problema**: Código comentado en varios archivos.

**Recomendación**: Eliminar código muerto, usar control de versiones

---

## 9. Roadmap de Implementación

### 9.1 Fase 1: Estabilización y Seguridad (4-6 semanas)

**Objetivos**:
- Resolver todos los puntos críticos
- Garantizar seguridad del sistema
- Mejorar estabilidad

**Tareas**:
1. ✅ Implementar RBAC completo (Semana 1-2)
2. ✅ Validación y sanitización de datos (Semana 2-3)
3. ✅ Manejo de errores estandarizado (Semana 3)
4. ✅ Optimización de performance (Semana 4-5)
5. ✅ Fix de concurrencia (Semana 5-6)

**Entregables**:
- Sistema con seguridad reforzada
- Tests de seguridad aprobados
- Documentación de seguridad

### 9.2 Fase 2: Refactorización y Testing (3-4 semanas)

**Objetivos**:
- Mejorar mantenibilidad del código
- Implementar suite de tests
- Automatizar CI/CD

**Tareas**:
1. ✅ Refactorizar main.py en módulos (Semana 1-2)
2. ✅ Implementar tests unitarios (Semana 2-3)
3. ✅ Implementar tests de integración (Semana 3)
4. ✅ Setup CI/CD (Semana 3-4)
5. ✅ Code coverage > 70% (Semana 4)

**Entregables**:
- Código modular y mantenible
- Suite de tests completa
- Pipeline CI/CD operativo

### 9.3 Fase 3: Mejoras de UX y Monitoreo (2-3 semanas)

**Objetivos**:
- Mejorar experiencia de usuario
- Implementar observabilidad

**Tareas**:
1. ✅ Implementar gráficos interactivos (Semana 1)
2. ✅ Dashboard ejecutivo (Semana 1)
3. ✅ Logging estructurado (Semana 2)
4. ✅ Métricas y alertas (Semana 2-3)
5. ✅ Modo oscuro (Semana 3)

**Entregables**:
- Dashboards visuales atractivos
- Sistema de monitoreo operativo
- Alertas configuradas

### 9.4 Fase 4: Documentación y Nuevas Funcionalidades (2-4 semanas)

**Objetivos**:
- Documentación completa
- Funcionalidades adicionales

**Tareas**:
1. ✅ Documentación OpenAPI (Semana 1)
2. ✅ Manual de usuario (Semana 1-2)
3. ✅ Alertas de saldo bajo (Semana 2)
4. ✅ Reportes avanzados (Semana 3-4)
5. ✅ API pública (Semana 4)

**Entregables**:
- Documentación completa y actualizada
- Funcionalidades nuevas operativas

### 9.5 Estimación Total de Esfuerzo

| Fase | Duración | Recursos | Prioridad |
|------|----------|----------|-----------|
| Fase 1: Estabilización | 4-6 semanas | 2 devs | 🔴 Crítica |
| Fase 2: Refactorización | 3-4 semanas | 2 devs | 🟡 Alta |
| Fase 3: UX y Monitoreo | 2-3 semanas | 1-2 devs | 🟡 Alta |
| Fase 4: Documentación | 2-4 semanas | 1 dev | 🟢 Media |
| **TOTAL** | **11-17 semanas** | **1-2 devs** | - |

---

## 10. Conclusiones y Recomendaciones

### 10.1 Fortalezas del Sistema Actual

1. ✅ **Arquitectura sólida**: Separación clara entre motor Rust y aplicación Python
2. ✅ **Funcionalidades completas**: Sistema cubre todo el ciclo de billing
3. ✅ **Integraciones robustas**: FreeSWITCH y CUCM bien implementados
4. ✅ **UI funcional**: Bootstrap 5 con componentes modernos
5. ✅ **WebSocket**: Monitoreo en tiempo real implementado
6. ✅ **Auditoría**: Trazabilidad de operaciones administrativas

### 10.2 Áreas Críticas de Mejora

1. 🔴 **Seguridad**: RBAC, 2FA, validación de inputs
2. 🔴 **Performance**: Índices, caché, optimización de queries
3. 🔴 **Calidad de código**: Refactorización, tests, CI/CD
4. 🟡 **Observabilidad**: Logging, métricas, alertas
5. 🟡 **Documentación**: API docs, manual de usuario

### 10.3 Recomendación Final

**Priorizar Fase 1** (Estabilización y Seguridad) antes de agregar nuevas funcionalidades. Un sistema seguro y estable es fundamental para un billing system que maneja transacciones financieras.

**Enfoque iterativo**: Implementar mejoras en sprints cortos (2 semanas) con entregas continuas.

**Métricas de éxito**:
- ✅ 0 vulnerabilidades críticas
- ✅ 99.9% uptime
- ✅ < 100ms latencia promedio en tarificación
- ✅ 70%+ code coverage
- ✅ < 5 minutos para CI/CD pipeline

---

## Anexos

### A. Stack Tecnológico Completo

**Backend**:
- FastAPI 0.x
- SQLAlchemy 1.x/2.x
- Pydantic 2.x
- Uvicorn
- passlib + bcrypt

**Motor Rust**:
- Tokio (async runtime)
- SQLx (PostgreSQL)
- Redis client
- ESL library

**Frontend**:
- Bootstrap 5.3.2
- jQuery 3.6.0
- DataTables 1.11.5
- Bootstrap Icons

**Base de Datos**:
- PostgreSQL 13+

**Integraciones**:
- FreeSWITCH ESL
- Cisco CUCM AXL (SOAP)

### B. Comandos Útiles

```bash
# Iniciar aplicación
cd /home/user/webapp && uvicorn main:app --reload --host 0.0.0.0 --port 8000

# Iniciar motor Rust
cd /home/user/webapp/rust-billing-engine && cargo run --release

# Conectar a PostgreSQL
psql -U apolo -d apolobilling

# Ver logs
tail -f /home/user/webapp/logs/app.log

# Backup de base de datos
pg_dump -U apolo apolobilling > backup_$(date +%Y%m%d).sql
```

### C. Enlaces de Referencia

- **FastAPI Docs**: https://fastapi.tiangolo.com/
- **SQLAlchemy Docs**: https://docs.sqlalchemy.org/
- **Rust Tokio**: https://tokio.rs/
- **FreeSWITCH ESL**: https://freeswitch.org/confluence/display/FREESWITCH/Event+Socket+Library

---

**Documento generado**: 2025-12-23  
**Versión**: 2.0  
**Autor**: Análisis automatizado del sistema Apolo Billing  
**Próxima revisión**: Después de Fase 1
