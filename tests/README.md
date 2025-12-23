# Sistema de Validación Multi-Agente - Apolo Billing

Sistema automatizado de testing con 6 agentes especializados para validación integral del sistema de billing.

## 📋 Índice

- [Arquitectura de Testing](#arquitectura-de-testing)
- [Instalación](#instalación)
- [Uso Rápido](#uso-rápido)
- [Agentes de Testing](#agentes-de-testing)
- [Comandos Disponibles](#comandos-disponibles)
- [Estructura de Archivos](#estructura-de-archivos)
- [Reportes](#reportes)
- [Troubleshooting](#troubleshooting)

## 🏗️ Arquitectura de Testing

El sistema utiliza **6 agentes especializados**, cada uno responsable de validar aspectos específicos:

```
┌─────────────────────────────────────────────────┐
│         COORDINADOR MAESTRO                     │
│     (run_all_agents.py)                        │
└────────────┬────────────────────────────────────┘
             │
             ├─── 🤖 Agente 1: Navegación y Enlaces
             │    └─ Valida endpoints, rutas, links
             │
             ├─── 🤖 Agente 2: CRUD Funcionalidades
             │    └─ Valida Create, Read, Update, Delete
             │
             ├─── 🤖 Agente 3: Integraciones
             │    └─ Valida FreeSWITCH, CUCM, WebSocket
             │
             ├─── 🤖 Agente 4: Seguridad
             │    └─ Valida auth, RBAC, vulnerabilidades
             │
             ├─── 🤖 Agente 5: Performance
             │    └─ Valida carga, latencia, concurrencia
             │
             └─── 🤖 Agente 6: UI/UX
                  └─ Valida responsive, accesibilidad, UX
```

## 🚀 Instalación

### 1. Instalar dependencias de testing

```bash
cd /home/user/webapp
pip install -r requirements-test.txt
```

### 2. Verificar instalación

```bash
pytest --version
python tests/run_all_agents.py --help
```

## ⚡ Uso Rápido

### Ejecutar todos los tests

```bash
# Ejecutar validación completa (todos los agentes)
python tests/run_all_agents.py

# O usando pytest directamente
pytest tests/ -v
```

### Ejecutar tests rápidos (solo críticos)

```bash
python tests/run_all_agents.py --quick
```

### Ejecutar un agente específico

```bash
# Agente 1: Navegación
python tests/run_all_agents.py --agent 1

# Agente 2: CRUD
python tests/run_all_agents.py --agent 2

# O usando pytest con markers
pytest -m agent1 -v
pytest -m agent2 -v
```

### Ejecutar solo tests críticos

```bash
pytest -m critical -v
```

## 🤖 Agentes de Testing

### Agente 1: Navegación y Enlaces 🔗

**Responsabilidad**: Validar que todos los endpoints y enlaces funcionen.

**Tests incluidos**:
- ✅ Endpoints públicos accesibles sin auth
- ✅ Endpoints protegidos requieren auth
- ✅ APIs retornan JSON válido
- ✅ Logout funciona correctamente
- ✅ Balance check endpoints existen

**Ejecutar**:
```bash
pytest tests/test_agent_1_*.py -v
```

### Agente 2: CRUD Funcionalidades 📝

**Responsabilidad**: Validar operaciones Create, Read, Update, Delete.

**Tests incluidos**:
- ✅ Crear entidades (Anexos, Zonas, Tarifas)
- ✅ Leer listas de entidades
- ✅ Validar duplicados
- ✅ Validar campos obligatorios
- ✅ Validar restricciones (ej: costos negativos)
- ✅ Búsqueda y filtros

**Ejecutar**:
```bash
pytest tests/test_agent_2_*.py -v
```

### Agente 3: Integraciones 🔌

**Responsabilidad**: Validar integraciones externas.

**Tests incluidos**:
- 🚧 FreeSWITCH ESL (en desarrollo)
- 🚧 Cisco CUCM AXL (en desarrollo)
- 🚧 WebSocket real-time (en desarrollo)
- 🚧 Redis cache (en desarrollo)

**Status**: En desarrollo

### Agente 4: Seguridad 🔒

**Responsabilidad**: Validar seguridad del sistema.

**Tests incluidos**:
- 🚧 Autenticación y autorización
- 🚧 RBAC (roles y permisos)
- 🚧 SQL injection protection
- 🚧 XSS protection
- 🚧 CSRF protection
- 🚧 Rate limiting

**Status**: En desarrollo

### Agente 5: Performance ⚡

**Responsabilidad**: Validar rendimiento y escalabilidad.

**Tests incluidos**:
- 🚧 Tests de carga (Locust)
- 🚧 Latencia de endpoints
- 🚧 Concurrencia
- 🚧 Memory leaks
- 🚧 Database query performance

**Status**: En desarrollo

### Agente 6: UI/UX 🎨

**Responsabilidad**: Validar experiencia de usuario.

**Tests incluidos**:
- 🚧 Responsive design
- 🚧 Accesibilidad (WCAG 2.1)
- 🚧 Navegación con teclado
- 🚧 Mensajes de error claros
- 🚧 Usabilidad de formularios

**Status**: En desarrollo

## 📂 Estructura de Archivos

```
tests/
├── __init__.py                    # Package marker
├── conftest.py                    # Configuración pytest y fixtures
├── run_all_agents.py              # Coordinador maestro (ejecutable)
│
├── test_agent_1_navigation.py    # Tests Agente 1
├── test_agent_2_crud.py           # Tests Agente 2
├── test_agent_3_integrations.py  # Tests Agente 3 (en desarrollo)
├── test_agent_4_security.py      # Tests Agente 4 (en desarrollo)
├── test_agent_5_performance.py   # Tests Agente 5 (en desarrollo)
├── test_agent_6_uiux.py           # Tests Agente 6 (en desarrollo)
│
├── fixtures/                      # Fixtures de datos de prueba
│   └── sample_data.json
│
├── data/                          # Archivos de datos para tests
│   ├── anexos_test.xlsx
│   └── recargas_test.xlsx
│
└── reports/                       # Reportes generados
    ├── full_report_*.json
    └── summary_*.txt
```

## 📊 Reportes

Después de ejecutar los tests, se generan automáticamente:

### Reporte JSON
```
tests/reports/full_report_20251223_153045.json
```

Contiene:
- Timestamp de ejecución
- Duración total
- Resultados por agente
- Output completo de pytest

### Reporte de Texto
```
tests/reports/summary_20251223_153045.txt
```

Resumen legible con:
- Total de tests ejecutados
- Tests pasados/fallidos
- Detalle por agente

### Reporte en Consola

Durante la ejecución, se muestra:
- Header colorido por agente
- Output de pytest en tiempo real
- Resumen final con estadísticas

## 🛠️ Comandos Disponibles

### Pytest directo

```bash
# Ejecutar todos los tests
pytest tests/ -v

# Ejecutar tests de un agente específico
pytest -m agent1 -v
pytest -m agent2 -v

# Ejecutar solo tests críticos
pytest -m critical -v

# Ejecutar solo tests rápidos (excluir slow)
pytest -m "not slow" -v

# Ejecutar con coverage
pytest tests/ --cov=. --cov-report=html

# Ejecutar con output detallado
pytest tests/ -v --tb=long

# Parar al primer fallo
pytest tests/ -x

# Ejecutar en paralelo (requiere pytest-xdist)
pytest tests/ -n auto
```

### Coordinador maestro

```bash
# Validación completa
python tests/run_all_agents.py

# Modo rápido (solo críticos)
python tests/run_all_agents.py --quick

# Agente específico
python tests/run_all_agents.py --agent 1
python tests/run_all_agents.py --agent 2

# Con ayuda
python tests/run_all_agents.py --help
```

## 🧪 Agregar Nuevos Tests

### 1. Crear archivo de test

```python
# tests/test_mi_funcionalidad.py

import pytest
from conftest import BASE_URL

pytestmark = pytest.mark.agent2  # Asignar a un agente

class TestMiFuncionalidad:
    
    def test_algo(self, auth_session):
        """Test de algo"""
        response = auth_session.get(f"{BASE_URL}/mi-endpoint")
        assert response.status_code == 200
```

### 2. Usar fixtures disponibles

```python
def test_con_fixtures(self, auth_session, test_anexo_data, cleanup_test_data):
    """Test que usa fixtures"""
    # auth_session: sesión autenticada
    # test_anexo_data: datos de anexo de prueba
    # cleanup_test_data: limpieza automática
    pass
```

### 3. Marcar tests especiales

```python
@pytest.mark.slow
def test_lento():
    """Test que tarda mucho"""
    pass

@pytest.mark.critical
def test_critico():
    """Test crítico que no debe fallar"""
    pass
```

## 🔧 Configuración

### Modificar configuración de pytest

Editar `tests/conftest.py`:

```python
BASE_URL = "http://localhost:8000"  # Cambiar URL si es necesario
DEFAULT_ADMIN_USER = "admin"
DEFAULT_ADMIN_PASS = "admin123"
```

### Agregar nuevos fixtures

```python
# En tests/conftest.py

@pytest.fixture
def mi_fixture():
    """Mi fixture personalizado"""
    # Setup
    yield dato
    # Teardown
```

## 🐛 Troubleshooting

### Error: "No module named 'pytest'"

```bash
pip install pytest
```

### Error: "Connection refused"

Asegúrate de que la aplicación esté corriendo:
```bash
cd /home/user/webapp
uvicorn main:app --host 0.0.0.0 --port 8000
```

### Error: "No se pudo autenticar"

Verifica que las credenciales en `conftest.py` sean correctas:
```python
DEFAULT_ADMIN_USER = "admin"
DEFAULT_ADMIN_PASS = "admin123"
```

### Tests fallan por timeout

Aumentar timeout en `run_all_agents.py`:
```python
timeout=600  # 10 minutos
```

### Ver output completo de tests fallidos

```bash
pytest tests/ -v --tb=long
```

## 📈 Métricas y Coverage

### Generar reporte de coverage

```bash
pytest tests/ --cov=. --cov-report=html
open htmlcov/index.html  # Ver reporte HTML
```

### Ver coverage en terminal

```bash
pytest tests/ --cov=. --cov-report=term-missing
```

## 🔄 CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/tests.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Python
        uses: actions/setup-python@v2
        with:
          python-version: '3.9'
      - name: Install dependencies
        run: |
          pip install -r requirements.txt
          pip install -r requirements-test.txt
      - name: Run tests
        run: python tests/run_all_agents.py
```

## 📚 Documentación Adicional

- [PRD Completo](../PRD_COMPLETO_APOLO_BILLING.md)
- [Plan de Validación](../PLAN_VALIDACION_MULTI_AGENTE.md)
- [Documentación de pytest](https://docs.pytest.org/)

## 🤝 Contribuir

Para agregar nuevos tests:

1. Crear archivo `test_agent_X_*.py`
2. Usar fixtures de `conftest.py`
3. Marcar con `@pytest.mark.agentX`
4. Documentar con docstrings claros
5. Ejecutar localmente antes de commit

## 📞 Soporte

Para problemas o dudas:
- Revisar esta documentación
- Consultar PRD y Plan de Validación
- Contactar al equipo QA

---

**Última actualización**: 2025-12-23  
**Versión**: 1.0  
**Mantenido por**: Equipo QA Apolo Billing
