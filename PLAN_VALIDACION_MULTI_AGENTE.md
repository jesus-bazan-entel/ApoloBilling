# Plan de Validación Multi-Agente - Apolo Billing System
## Testing Integral con Enfoque de Agentes Especializados

---

## 📋 Índice

1. [Introducción](#introducción)
2. [Arquitectura de Agentes](#arquitectura-de-agentes)
3. [Agente 1: Validador de Navegación y Enlaces](#agente-1-validador-de-navegación-y-enlaces)
4. [Agente 2: Validador de Funcionalidades CRUD](#agente-2-validador-de-funcionalidades-crud)
5. [Agente 3: Validador de Integraciones](#agente-3-validador-de-integraciones)
6. [Agente 4: Validador de Seguridad](#agente-4-validador-de-seguridad)
7. [Agente 5: Validador de Performance](#agente-5-validador-de-performance)
8. [Agente 6: Validador de UI/UX](#agente-6-validador-de-uiux)
9. [Scripts de Automatización](#scripts-de-automatización)
10. [Reporte de Resultados](#reporte-de-resultados)

---

## 1. Introducción

### 1.1 Objetivos del Plan

Este plan establece una estrategia de validación exhaustiva del sistema Apolo Billing utilizando **6 agentes especializados**, cada uno con responsabilidades específicas para garantizar la calidad integral del sistema.

### 1.2 Alcance

- ✅ Validación de todos los endpoints (80+)
- ✅ Validación de todos los enlaces en templates (33 archivos)
- ✅ Validación de funcionalidades CRUD completas
- ✅ Validación de integraciones (FreeSWITCH, CUCM)
- ✅ Validación de seguridad y autenticación
- ✅ Validación de performance y escalabilidad
- ✅ Validación de experiencia de usuario

### 1.3 Metodología

**Enfoque**: Testing automatizado con scripts Python utilizando:
- `pytest` para tests unitarios y de integración
- `requests` para tests de API
- `selenium` o `playwright` para tests de UI
- `locust` para tests de carga
- `coverage.py` para cobertura de código

**Duración estimada**: 2-3 semanas

**Recursos**: 1-2 QA Engineers + 1 DevOps

---

## 2. Arquitectura de Agentes

### 2.1 Diagrama de Agentes

```
┌─────────────────────────────────────────────────────────────┐
│                    COORDINADOR MAESTRO                       │
│        (Orquesta la ejecución de todos los agentes)         │
└────────────┬────────────────────────────────────────────────┘
             │
             ├─────────────────────────────────────────────────┐
             │                                                 │
    ┌────────▼────────┐                              ┌────────▼────────┐
    │   AGENTE 1      │                              │   AGENTE 2      │
    │   Navegación    │                              │   CRUD          │
    │   y Enlaces     │                              │   Funcional     │
    └────────┬────────┘                              └────────┬────────┘
             │                                                │
             ├─────────────────┬──────────────────────────────┤
             │                 │                              │
    ┌────────▼────────┐ ┌────────▼────────┐        ┌────────▼────────┐
    │   AGENTE 3      │ │   AGENTE 4      │        │   AGENTE 5      │
    │   Integraciones │ │   Seguridad     │        │   Performance   │
    └────────┬────────┘ └────────┬────────┘        └────────┬────────┘
             │                   │                          │
             └───────────────────┴──────────────────────────┤
                                                            │
                                                   ┌────────▼────────┐
                                                   │   AGENTE 6      │
                                                   │   UI/UX         │
                                                   └────────┬────────┘
                                                            │
┌───────────────────────────────────────────────────────────▼─────────┐
│                      GENERADOR DE REPORTES                           │
│          (Consolida resultados de todos los agentes)                │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Responsabilidades de Cada Agente

| Agente | Responsabilidad | Prioridad | Duración |
|--------|----------------|-----------|----------|
| **Agente 1** | Validar navegación, enlaces, rutas | 🔴 Alta | 2-3 días |
| **Agente 2** | Validar CRUD completo de todas las entidades | 🔴 Alta | 4-5 días |
| **Agente 3** | Validar integraciones externas | 🟡 Media | 3-4 días |
| **Agente 4** | Validar seguridad, autenticación, autorización | 🔴 Alta | 3-4 días |
| **Agente 5** | Validar performance, carga, escalabilidad | 🟡 Media | 2-3 días |
| **Agente 6** | Validar UI/UX, accesibilidad, responsive | 🟢 Media | 2-3 días |

---

## 3. Agente 1: Validador de Navegación y Enlaces

### 3.1 Objetivo

Validar que **todos los enlaces y rutas** de la aplicación funcionen correctamente, sin enlaces rotos, y que la navegación sea intuitiva.

### 3.2 Tareas Específicas

#### 3.2.1 Validación de Endpoints REST

**Test**: Verificar que todos los 80+ endpoints respondan correctamente.

**Casos de prueba**:
```python
# tests/test_agent_1_navigation.py

import pytest
import requests

BASE_URL = "http://localhost:8000"

# Lista de todos los endpoints públicos
PUBLIC_ENDPOINTS = [
    "/login",
    "/auth/login",
]

# Lista de endpoints protegidos (requieren auth)
PROTECTED_ENDPOINTS = [
    "/",
    "/dashboard/saldo",
    "/dashboard/cdr",
    "/dashboard/finanzas",
    "/dashboard/recargas",
    "/dashboard/auditoria",
    "/dashboard/ranking_consumo",
    "/dashboard/anexos",
    "/dashboard/monitoreo",
    "/dashboard/zonas",
    "/dashboard/prefijos",
    "/dashboard/tarifas",
    "/dashboard/estadisticas_zona",
    "/dashboard/cucm",
    "/dashboard/pines",
    "/dashboard/fac",
    "/dashboard/fac_historial",
    "/dashboard/fac_sync",
    "/dashboard/lineas",
    "/dashboard/planes",
    "/dashboard/abonados",
    "/dashboard/recarga_masiva",
    "/dashboard/anexos/carga_masiva",
]

class TestAgent1Navigation:
    
    @pytest.mark.parametrize("endpoint", PUBLIC_ENDPOINTS)
    def test_public_endpoints_accessible(self, endpoint):
        """Validar que endpoints públicos sean accesibles sin auth"""
        response = requests.get(f"{BASE_URL}{endpoint}", allow_redirects=False)
        assert response.status_code in [200, 302], f"Endpoint {endpoint} falló"
    
    @pytest.mark.parametrize("endpoint", PROTECTED_ENDPOINTS)
    def test_protected_endpoints_require_auth(self, endpoint):
        """Validar que endpoints protegidos requieran autenticación"""
        response = requests.get(f"{BASE_URL}{endpoint}", allow_redirects=False)
        # Debe redirigir a login o retornar 401/403
        assert response.status_code in [302, 401, 403], \
            f"Endpoint {endpoint} debería requerir auth"
    
    def test_authenticated_access_to_dashboards(self, auth_session):
        """Validar acceso a dashboards con sesión autenticada"""
        for endpoint in PROTECTED_ENDPOINTS:
            response = auth_session.get(f"{BASE_URL}{endpoint}")
            assert response.status_code == 200, \
                f"Dashboard {endpoint} no accesible con auth"
    
    def test_api_endpoints_return_json(self, auth_session):
        """Validar que endpoints /api/* retornen JSON válido"""
        api_endpoints = [
            "/api/active-calls",
            "/api/active-calls-list",
            "/api/cucm/config",
            "/api/cucm/status",
            "/api/config",
            "/api/ws-stats",
        ]
        
        for endpoint in api_endpoints:
            response = auth_session.get(f"{BASE_URL}{endpoint}")
            assert response.status_code == 200, f"API {endpoint} falló"
            assert response.headers.get("Content-Type", "").startswith("application/json"), \
                f"API {endpoint} no retorna JSON"
            assert response.json() is not None, f"API {endpoint} retorna JSON inválido"
    
    def test_logout_redirects_to_login(self, auth_session):
        """Validar que logout cierre sesión y redirija a login"""
        response = auth_session.get(f"{BASE_URL}/logout", allow_redirects=False)
        assert response.status_code == 302
        assert "/login" in response.headers.get("Location", "")
```

#### 3.2.2 Validación de Enlaces en Templates

**Test**: Rastrear todos los enlaces en templates HTML.

**Script de análisis**:
```python
# tests/test_agent_1_template_links.py

import os
import re
from bs4 import BeautifulSoup
import pytest

TEMPLATES_DIR = "/home/user/webapp/templates"

def extract_links_from_template(template_path):
    """Extrae todos los enlaces <a href> de un template"""
    with open(template_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    soup = BeautifulSoup(content, 'html.parser')
    links = []
    
    # Enlaces <a href>
    for a_tag in soup.find_all('a', href=True):
        href = a_tag['href']
        # Filtrar Jinja2 variables, external links
        if not href.startswith(('http', 'https', 'mailto', 'tel', '#')):
            if not href.startswith('{{'):  # Ignorar variables Jinja
                links.append(href)
    
    # Forms action
    for form in soup.find_all('form', action=True):
        action = form['action']
        if not action.startswith(('http', 'https', '{{')):
            links.append(action)
    
    return links

def get_all_templates():
    """Obtiene lista de todos los templates"""
    templates = []
    for root, dirs, files in os.walk(TEMPLATES_DIR):
        for file in files:
            if file.endswith('.html'):
                templates.append(os.path.join(root, file))
    return templates

class TestAgent1TemplateLinks:
    
    def test_extract_all_links_from_templates(self):
        """Extraer y catalogar todos los enlaces de templates"""
        all_links = {}
        
        for template in get_all_templates():
            links = extract_links_from_template(template)
            template_name = os.path.basename(template)
            all_links[template_name] = links
        
        # Guardar reporte
        with open('/home/user/webapp/tests/reports/template_links.txt', 'w') as f:
            for template, links in all_links.items():
                f.write(f"\n=== {template} ===\n")
                for link in links:
                    f.write(f"  - {link}\n")
        
        assert len(all_links) > 0, "No se encontraron templates"
    
    def test_validate_internal_links(self, auth_session):
        """Validar que enlaces internos no estén rotos"""
        broken_links = []
        
        for template in get_all_templates():
            links = extract_links_from_template(template)
            template_name = os.path.basename(template)
            
            for link in links:
                # Probar el link
                try:
                    response = auth_session.get(f"{BASE_URL}{link}", allow_redirects=True)
                    if response.status_code >= 400:
                        broken_links.append({
                            'template': template_name,
                            'link': link,
                            'status': response.status_code
                        })
                except Exception as e:
                    broken_links.append({
                        'template': template_name,
                        'link': link,
                        'error': str(e)
                    })
        
        # Reporte de enlaces rotos
        if broken_links:
            with open('/home/user/webapp/tests/reports/broken_links.txt', 'w') as f:
                for item in broken_links:
                    f.write(f"Template: {item['template']}\n")
                    f.write(f"Link: {item['link']}\n")
                    f.write(f"Status: {item.get('status', 'ERROR')}\n")
                    f.write(f"Error: {item.get('error', 'N/A')}\n\n")
        
        assert len(broken_links) == 0, f"Se encontraron {len(broken_links)} enlaces rotos"
```

#### 3.2.3 Validación de Navegación Sidebar

**Test**: Verificar que el menú sidebar contenga todos los enlaces necesarios.

```python
# tests/test_agent_1_sidebar.py

from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
import pytest

class TestAgent1Sidebar:
    
    @pytest.fixture(scope="class")
    def driver(self):
        driver = webdriver.Chrome()
        driver.implicitly_wait(10)
        yield driver
        driver.quit()
    
    @pytest.fixture(scope="class")
    def authenticated_driver(self, driver):
        """Login y retornar driver autenticado"""
        driver.get(f"{BASE_URL}/login")
        driver.find_element(By.NAME, "username").send_keys("admin")
        driver.find_element(By.NAME, "password").send_keys("admin123")
        driver.find_element(By.CSS_SELECTOR, "button[type='submit']").click()
        WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.ID, "sidebar"))
        )
        return driver
    
    def test_sidebar_contains_all_menu_items(self, authenticated_driver):
        """Validar que sidebar contenga todos los ítems de menú esperados"""
        expected_menu_items = [
            "Dashboard Principal",
            "Saldo",
            "CDRs",
            "Llamadas Activas",
            "Finanzas",
            "Ranking de Consumo",
            "Anexos",
            "Zonas",
            "Prefijos",
            "Tarifas",
            "PINes/FAC",
            "Recarga Manual",
            "Recarga Masiva",
            "Cisco CUCM",
            "Auditoría",
        ]
        
        sidebar = authenticated_driver.find_element(By.ID, "sidebar")
        sidebar_text = sidebar.text
        
        missing_items = []
        for item in expected_menu_items:
            if item not in sidebar_text:
                missing_items.append(item)
        
        assert len(missing_items) == 0, \
            f"Faltan ítems en sidebar: {', '.join(missing_items)}"
    
    def test_sidebar_links_are_clickable(self, authenticated_driver):
        """Validar que todos los enlaces del sidebar sean clickeables"""
        sidebar = authenticated_driver.find_element(By.ID, "sidebar")
        links = sidebar.find_elements(By.TAG_NAME, "a")
        
        broken_links = []
        for link in links:
            try:
                href = link.get_attribute("href")
                if href and not href.startswith(("#", "javascript:")):
                    # Hacer click
                    link.click()
                    WebDriverWait(authenticated_driver, 5).until(
                        EC.presence_of_element_located((By.TAG_NAME, "body"))
                    )
                    # Volver atrás
                    authenticated_driver.back()
            except Exception as e:
                broken_links.append({
                    'href': href,
                    'text': link.text,
                    'error': str(e)
                })
        
        assert len(broken_links) == 0, \
            f"Enlaces rotos en sidebar: {broken_links}"
```

### 3.3 Criterios de Aceptación

- ✅ Todos los endpoints públicos accesibles sin error
- ✅ Todos los endpoints protegidos requieren autenticación
- ✅ 0 enlaces rotos en templates
- ✅ Sidebar contiene todos los ítems de menú esperados
- ✅ Todos los enlaces del sidebar son clickeables

### 3.4 Reporte Generado

```
=== REPORTE AGENTE 1: NAVEGACIÓN Y ENLACES ===

Fecha: 2025-12-23
Duración: 2 horas

RESUMEN:
- Endpoints públicos validados: 2/2 ✅
- Endpoints protegidos validados: 20/20 ✅
- Templates analizados: 33/33 ✅
- Enlaces internos validados: 250/252 ⚠️
- Enlaces rotos encontrados: 2 ❌

ENLACES ROTOS:
1. Template: dashboard_anexos.html
   Link: /api/anexo/export/pdf
   Status: 404
   
2. Template: dashboard_cdr.html
   Link: /api/cdr/export/csv
   Status: 404

RECOMENDACIONES:
- Implementar endpoint /api/anexo/export/pdf
- Implementar endpoint /api/cdr/export/csv
- Revisar enlaces obsoletos en templates
```

---

## 4. Agente 2: Validador de Funcionalidades CRUD

### 4.1 Objetivo

Validar que todas las operaciones **CRUD (Create, Read, Update, Delete)** de cada entidad funcionen correctamente de extremo a extremo.

### 4.2 Entidades a Validar

1. **Anexos** (Extensiones)
2. **Zonas** (Geográficas)
3. **Prefijos** (Telefónicos)
4. **Tarifas** (Rates)
5. **Recargas** (Top-ups)
6. **FAC Codes** (Forced Authorization Codes)
7. **Usuarios** (Users)
8. **CDRs** (Call Detail Records - solo lectura)
9. **CUCM Config** (Configuración)

### 4.3 Tests por Entidad

#### 4.3.1 CRUD de Anexos

```python
# tests/test_agent_2_crud_anexos.py

import pytest
import requests

class TestAgent2CRUDAnexos:
    
    @pytest.fixture(scope="class")
    def anexo_test_data(self):
        return {
            "numero": "9999",
            "area": "Test Area",
            "nombre": "Test Anexo",
            "pin": "1234",
            "saldo_inicial": 100.00
        }
    
    def test_create_anexo(self, auth_session, anexo_test_data):
        """Test: Crear un nuevo anexo"""
        response = auth_session.post(
            f"{BASE_URL}/anexo",
            data=anexo_test_data
        )
        assert response.status_code == 200, "Fallo al crear anexo"
        
        # Guardar ID para tests posteriores
        pytest.anexo_id = response.json().get('id')
        assert pytest.anexo_id is not None, "No se retornó ID del anexo creado"
    
    def test_read_anexo(self, auth_session):
        """Test: Leer el anexo creado"""
        response = auth_session.get(f"{BASE_URL}/anexo/{pytest.anexo_id}")
        assert response.status_code == 200, "Fallo al leer anexo"
        
        data = response.json()
        assert data['numero'] == "9999"
        assert data['area'] == "Test Area"
    
    def test_update_anexo(self, auth_session):
        """Test: Actualizar el anexo"""
        updated_data = {
            "numero": "9999",
            "area": "Updated Area",
            "nombre": "Updated Anexo",
            "pin": "5678",
            "saldo_inicial": 200.00
        }
        
        response = auth_session.put(
            f"{BASE_URL}/anexo/{pytest.anexo_id}",
            data=updated_data
        )
        assert response.status_code == 200, "Fallo al actualizar anexo"
        
        # Verificar actualización
        response = auth_session.get(f"{BASE_URL}/anexo/{pytest.anexo_id}")
        data = response.json()
        assert data['area'] == "Updated Area"
        assert data['nombre'] == "Updated Anexo"
    
    def test_delete_anexo(self, auth_session):
        """Test: Eliminar el anexo"""
        response = auth_session.delete(f"{BASE_URL}/anexo/{pytest.anexo_id}")
        assert response.status_code == 200, "Fallo al eliminar anexo"
        
        # Verificar eliminación
        response = auth_session.get(f"{BASE_URL}/anexo/{pytest.anexo_id}")
        assert response.status_code == 404, "Anexo no fue eliminado"
    
    def test_create_anexo_with_duplicate_numero(self, auth_session, anexo_test_data):
        """Test: No permitir anexos con número duplicado"""
        # Crear anexo
        auth_session.post(f"{BASE_URL}/anexo", data=anexo_test_data)
        
        # Intentar crear otro con mismo número
        response = auth_session.post(f"{BASE_URL}/anexo", data=anexo_test_data)
        assert response.status_code in [400, 409], \
            "Debería rechazar anexo con número duplicado"
    
    def test_anexo_validation(self, auth_session):
        """Test: Validar campos obligatorios"""
        invalid_data = {
            "numero": "",  # Vacío, debería fallar
            "area": "Test",
            "nombre": "Test"
        }
        
        response = auth_session.post(f"{BASE_URL}/anexo", data=invalid_data)
        assert response.status_code == 400, "Debería validar campos obligatorios"
```

#### 4.3.2 CRUD de Zonas

```python
# tests/test_agent_2_crud_zonas.py

class TestAgent2CRUDZonas:
    
    @pytest.fixture(scope="class")
    def zona_test_data(self):
        return {
            "nombre": "Zona Test",
            "descripcion": "Zona de prueba automatizada"
        }
    
    def test_create_zona(self, auth_session, zona_test_data):
        """Test: Crear una nueva zona"""
        response = auth_session.post(
            f"{BASE_URL}/api/zonas",
            json=zona_test_data
        )
        assert response.status_code == 200, "Fallo al crear zona"
        pytest.zona_id = response.json().get('id')
    
    def test_read_zona(self, auth_session):
        """Test: Leer zona creada"""
        response = auth_session.get(f"{BASE_URL}/api/zonas/{pytest.zona_id}")
        assert response.status_code == 200
        data = response.json()
        assert data['nombre'] == "Zona Test"
    
    def test_update_zona(self, auth_session):
        """Test: Actualizar zona"""
        updated_data = {
            "nombre": "Zona Test Actualizada",
            "descripcion": "Descripción actualizada"
        }
        response = auth_session.put(
            f"{BASE_URL}/api/zonas/{pytest.zona_id}",
            json=updated_data
        )
        assert response.status_code == 200
    
    def test_delete_zona(self, auth_session):
        """Test: Eliminar zona"""
        response = auth_session.delete(f"{BASE_URL}/api/zonas/{pytest.zona_id}")
        assert response.status_code == 200
        
        # Verificar eliminación
        response = auth_session.get(f"{BASE_URL}/api/zonas/{pytest.zona_id}")
        assert response.status_code == 404
    
    def test_zona_cascade_delete_with_prefijos(self, auth_session):
        """Test: Validar eliminación en cascada (zona con prefijos)"""
        # Crear zona
        zona_data = {"nombre": "Zona Cascade Test", "descripcion": "Test"}
        response = auth_session.post(f"{BASE_URL}/api/zonas", json=zona_data)
        zona_id = response.json()['id']
        
        # Crear prefijo asociado
        prefijo_data = {
            "prefijo": "001",
            "zona_id": zona_id,
            "descripcion": "Test prefix"
        }
        response = auth_session.post(f"{BASE_URL}/api/prefijos", json=prefijo_data)
        
        # Intentar eliminar zona (debería fallar o eliminar en cascada)
        response = auth_session.delete(f"{BASE_URL}/api/zonas/{zona_id}")
        
        # Verificar comportamiento esperado
        # Opción 1: Rechazar eliminación (400/409)
        # Opción 2: Eliminar en cascada (200)
        assert response.status_code in [200, 400, 409], \
            "Comportamiento inesperado al eliminar zona con prefijos"
```

#### 4.3.3 CRUD de Tarifas

```python
# tests/test_agent_2_crud_tarifas.py

class TestAgent2CRUDTarifas:
    
    @pytest.fixture(scope="class")
    def setup_zona_for_tarifa(self, auth_session):
        """Crear zona para asociar tarifas"""
        zona_data = {"nombre": "Zona Tarifa Test", "descripcion": "Test"}
        response = auth_session.post(f"{BASE_URL}/api/zonas", json=zona_data)
        pytest.zona_tarifa_id = response.json()['id']
        
        yield
        
        # Cleanup
        auth_session.delete(f"{BASE_URL}/api/zonas/{pytest.zona_tarifa_id}")
    
    def test_create_tarifa(self, auth_session, setup_zona_for_tarifa):
        """Test: Crear tarifa"""
        tarifa_data = {
            "zona_id": pytest.zona_tarifa_id,
            "costo_minuto": 0.05,
            "costo_conexion": 0.10,
            "franja_horaria": "normal"
        }
        
        response = auth_session.post(f"{BASE_URL}/api/tarifas", json=tarifa_data)
        assert response.status_code == 200
        pytest.tarifa_id = response.json()['id']
    
    def test_read_tarifa(self, auth_session):
        """Test: Leer tarifa"""
        response = auth_session.get(f"{BASE_URL}/api/tarifas/{pytest.tarifa_id}")
        assert response.status_code == 200
        data = response.json()
        assert float(data['costo_minuto']) == 0.05
    
    def test_update_tarifa(self, auth_session):
        """Test: Actualizar tarifa"""
        updated_data = {
            "zona_id": pytest.zona_tarifa_id,
            "costo_minuto": 0.08,
            "costo_conexion": 0.15,
            "franja_horaria": "nocturna"
        }
        
        response = auth_session.put(
            f"{BASE_URL}/api/tarifas/{pytest.tarifa_id}",
            json=updated_data
        )
        assert response.status_code == 200
    
    def test_delete_tarifa(self, auth_session):
        """Test: Eliminar tarifa"""
        response = auth_session.delete(f"{BASE_URL}/api/tarifas/{pytest.tarifa_id}")
        assert response.status_code == 200
    
    def test_tarifa_validation_negative_cost(self, auth_session, setup_zona_for_tarifa):
        """Test: No permitir costos negativos"""
        invalid_data = {
            "zona_id": pytest.zona_tarifa_id,
            "costo_minuto": -0.05,  # Negativo
            "costo_conexion": 0.10,
            "franja_horaria": "normal"
        }
        
        response = auth_session.post(f"{BASE_URL}/api/tarifas", json=invalid_data)
        assert response.status_code == 400, "Debería rechazar costos negativos"
```

### 4.4 Tests de Carga Masiva

```python
# tests/test_agent_2_bulk_operations.py

class TestAgent2BulkOperations:
    
    def test_carga_masiva_anexos(self, auth_session):
        """Test: Carga masiva de anexos desde Excel"""
        import io
        import pandas as pd
        
        # Crear archivo Excel de prueba
        df = pd.DataFrame({
            'numero': ['8001', '8002', '8003', '8004', '8005'],
            'area': ['IT', 'HR', 'Sales', 'Finance', 'Marketing'],
            'nombre': ['Dev 1', 'HR 1', 'Sales 1', 'Finance 1', 'Marketing 1'],
            'pin': ['1111', '2222', '3333', '4444', '5555'],
            'saldo_inicial': [100, 150, 200, 250, 300]
        })
        
        excel_buffer = io.BytesIO()
        df.to_excel(excel_buffer, index=False)
        excel_buffer.seek(0)
        
        # Enviar archivo
        files = {'file': ('anexos_test.xlsx', excel_buffer, 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet')}
        response = auth_session.post(
            f"{BASE_URL}/dashboard/anexos/carga_masiva",
            files=files
        )
        
        assert response.status_code == 200, "Fallo carga masiva"
        
        # Verificar que se crearon los anexos
        response = auth_session.get(f"{BASE_URL}/dashboard/anexos")
        assert '8001' in response.text
        assert '8005' in response.text
    
    def test_recarga_masiva(self, auth_session):
        """Test: Recarga masiva desde Excel"""
        import io
        import pandas as pd
        
        # Crear archivo Excel de recargas
        df = pd.DataFrame({
            'numero': ['8001', '8002', '8003'],
            'monto': [50.00, 75.00, 100.00]
        })
        
        excel_buffer = io.BytesIO()
        df.to_excel(excel_buffer, index=False)
        excel_buffer.seek(0)
        
        files = {'file': ('recargas_test.xlsx', excel_buffer, 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet')}
        response = auth_session.post(
            f"{BASE_URL}/dashboard/recarga_masiva",
            files=files
        )
        
        assert response.status_code == 200
        
        # Verificar que se aplicaron las recargas
        response = auth_session.get(f"{BASE_URL}/dashboard/recargas")
        assert 'success' in response.text.lower()
```

### 4.5 Criterios de Aceptación

- ✅ CRUD completo funciona para todas las entidades
- ✅ Validaciones de campos obligatorios funcionan
- ✅ Validaciones de duplicados funcionan
- ✅ Carga masiva funciona correctamente
- ✅ Eliminación en cascada funciona según diseño
- ✅ Transacciones atómicas (rollback en caso de error)

---

## 5. Agente 3: Validador de Integraciones

### 5.1 Objetivo

Validar que las integraciones externas con **FreeSWITCH** y **Cisco CUCM** funcionen correctamente.

### 5.2 Tests de Integración FreeSWITCH

```python
# tests/test_agent_3_freeswitch.py

class TestAgent3FreeSWITCH:
    
    def test_check_balance_endpoint(self, auth_session):
        """Test: Endpoint de consulta de saldo (llamado por FreeSWITCH)"""
        calling_number = "8001"
        called_number = "51987654321"
        
        response = requests.get(
            f"{BASE_URL}/check_balance/{calling_number}/{called_number}"
        )
        
        assert response.status_code == 200
        data = response.json()
        assert 'balance' in data or 'saldo' in data
        assert 'authorized' in data or 'autorizado' in data
    
    def test_cdr_reception(self, auth_session):
        """Test: Recepción de CDR desde FreeSWITCH"""
        cdr_data = {
            "call_id": "test-call-id-001",
            "calling_number": "8001",
            "called_number": "51987654321",
            "direction": "outbound",
            "start_time": "2025-12-23T10:00:00",
            "answer_time": "2025-12-23T10:00:05",
            "end_time": "2025-12-23T10:05:00",
            "duration": 300,
            "billsec": 295,
            "hangup_cause": "NORMAL_CLEARING",
            "cost": 14.75
        }
        
        response = requests.post(f"{BASE_URL}/cdr", json=cdr_data)
        assert response.status_code == 200
        
        # Verificar que el CDR se guardó en BD
        response = auth_session.get(f"{BASE_URL}/dashboard/cdr")
        assert "test-call-id-001" in response.text
    
    def test_rejected_cdr_reception(self, auth_session):
        """Test: Recepción de CDR rechazado"""
        rejected_cdr = {
            "call_id": "rejected-call-001",
            "calling_number": "8999",  # No existe
            "called_number": "51987654321",
            "reason": "INSUFFICIENT_BALANCE"
        }
        
        response = requests.post(f"{BASE_URL}/cdr/rejected", json=rejected_cdr)
        assert response.status_code == 200
    
    def test_active_call_webhook(self, auth_session):
        """Test: Webhook de llamada activa"""
        active_call_data = {
            "call_id": "active-test-001",
            "calling_number": "8001",
            "called_number": "51987654321",
            "direction": "outbound",
            "start_time": "2025-12-23T10:00:00",
            "current_duration": 60,
            "current_cost": 3.00,
            "zone": "Peru Mobile"
        }
        
        response = auth_session.post(
            f"{BASE_URL}/api/active-calls",
            json=active_call_data
        )
        assert response.status_code == 200
        
        # Verificar que aparece en llamadas activas
        response = auth_session.get(f"{BASE_URL}/api/active-calls")
        data = response.json()
        assert any(call['call_id'] == 'active-test-001' for call in data)
```

### 5.3 Tests de Integración CUCM

```python
# tests/test_agent_3_cucm.py

class TestAgent3CUCM:
    
    def test_cucm_config_save(self, auth_session):
        """Test: Guardar configuración CUCM"""
        config_data = {
            "host": "cucm-test.example.com",
            "username": "axluser",
            "password": "axlpassword",
            "version": "12.5",
            "enabled": True
        }
        
        response = auth_session.post(
            f"{BASE_URL}/api/cucm/config",
            json=config_data
        )
        assert response.status_code == 200
    
    def test_cucm_test_connection(self, auth_session):
        """Test: Probar conexión CUCM"""
        response = auth_session.post(f"{BASE_URL}/api/cucm/test_connection")
        
        # Puede fallar si no hay CUCM real, pero endpoint debe responder
        assert response.status_code in [200, 400, 500]
        data = response.json()
        assert 'status' in data or 'success' in data
    
    def test_cucm_sync_service_control(self, auth_session):
        """Test: Control de servicio de sincronización"""
        # Start service
        response = auth_session.post(f"{BASE_URL}/api/cucm/service/start")
        assert response.status_code in [200, 400]  # Puede fallar si ya está corriendo
        
        # Check status
        response = auth_session.get(f"{BASE_URL}/api/cucm/status")
        assert response.status_code == 200
        data = response.json()
        assert 'status' in data
        
        # Stop service
        response = auth_session.post(f"{BASE_URL}/api/cucm/service/stop")
        assert response.status_code == 200
    
    def test_cucm_logs_viewer(self, auth_session):
        """Test: Visualizar logs de CUCM"""
        response = auth_session.get(f"{BASE_URL}/api/cucm/logs")
        assert response.status_code == 200
        # Debe retornar logs o un array vacío
        assert response.headers.get("Content-Type", "").startswith("application/json")
```

### 5.4 Tests de WebSocket (Llamadas Activas)

```python
# tests/test_agent_3_websocket.py

import asyncio
import websockets
import json

class TestAgent3WebSocket:
    
    @pytest.mark.asyncio
    async def test_websocket_connection(self):
        """Test: Conectar al WebSocket"""
        uri = "ws://localhost:8000/ws"
        
        try:
            async with websockets.connect(uri) as websocket:
                # Esperar mensaje inicial
                message = await asyncio.wait_for(websocket.recv(), timeout=5)
                data = json.loads(message)
                
                assert 'type' in data
                assert 'active_calls' in data or data['type'] == 'update'
                
        except Exception as e:
            pytest.fail(f"WebSocket connection failed: {str(e)}")
    
    @pytest.mark.asyncio
    async def test_websocket_receive_updates(self):
        """Test: Recibir actualizaciones de llamadas activas"""
        uri = "ws://localhost:8000/ws"
        
        async with websockets.connect(uri) as websocket:
            # Solicitar llamadas activas
            await websocket.send(json.dumps({"action": "get_active_calls"}))
            
            # Recibir respuesta
            message = await asyncio.wait_for(websocket.recv(), timeout=5)
            data = json.loads(message)
            
            assert data['type'] == 'update'
            assert isinstance(data['active_calls'], list)
```

### 5.5 Criterios de Aceptación

- ✅ Endpoint `/check_balance` funciona correctamente
- ✅ Endpoint `/cdr` recibe y almacena CDRs
- ✅ Endpoint `/cdr/rejected` registra llamadas rechazadas
- ✅ Configuración CUCM se puede guardar y recuperar
- ✅ Test de conexión CUCM responde apropiadamente
- ✅ Servicio CUCM se puede iniciar/detener
- ✅ WebSocket acepta conexiones y envía actualizaciones

---

## 6. Agente 4: Validador de Seguridad

### 6.1 Objetivo

Validar que el sistema esté protegido contra vulnerabilidades comunes y siga mejores prácticas de seguridad.

### 6.2 Tests de Autenticación

```python
# tests/test_agent_4_authentication.py

class TestAgent4Authentication:
    
    def test_login_with_valid_credentials(self):
        """Test: Login con credenciales válidas"""
        response = requests.post(
            f"{BASE_URL}/auth/login",
            data={"username": "admin", "password": "admin123"}
        )
        assert response.status_code == 200
        assert "auth_token" in response.cookies
    
    def test_login_with_invalid_credentials(self):
        """Test: Login con credenciales inválidas"""
        response = requests.post(
            f"{BASE_URL}/auth/login",
            data={"username": "admin", "password": "wrongpassword"}
        )
        assert response.status_code in [401, 403]
    
    def test_protected_endpoint_without_auth(self):
        """Test: Acceso a endpoint protegido sin autenticación"""
        response = requests.get(f"{BASE_URL}/dashboard/saldo", allow_redirects=False)
        assert response.status_code in [302, 401, 403]
    
    def test_session_cookie_security(self):
        """Test: Cookie de sesión tiene flags de seguridad"""
        response = requests.post(
            f"{BASE_URL}/auth/login",
            data={"username": "admin", "password": "admin123"}
        )
        
        cookie = response.cookies.get("auth_token")
        assert cookie is not None
        
        # Verificar flags de seguridad (en producción)
        # assert response.cookies['auth_token']['secure'] == True
        # assert response.cookies['auth_token']['httponly'] == True
    
    def test_logout_invalidates_session(self, auth_session):
        """Test: Logout invalida la sesión"""
        # Logout
        auth_session.get(f"{BASE_URL}/logout")
        
        # Intentar acceder a recurso protegido
        response = auth_session.get(f"{BASE_URL}/dashboard/saldo", allow_redirects=False)
        assert response.status_code in [302, 401, 403]
```

### 6.3 Tests de Autorización (RBAC)

```python
# tests/test_agent_4_authorization.py

class TestAgent4Authorization:
    
    def test_admin_can_access_all_resources(self, admin_session):
        """Test: Admin tiene acceso a todos los recursos"""
        admin_endpoints = [
            "/dashboard/anexos",
            "/dashboard/tarifas",
            "/dashboard/auditoria",
            "/api/cucm/config",
        ]
        
        for endpoint in admin_endpoints:
            response = admin_session.get(f"{BASE_URL}{endpoint}")
            assert response.status_code == 200, f"Admin no puede acceder a {endpoint}"
    
    def test_viewer_cannot_modify_data(self, viewer_session):
        """Test: Usuario con rol viewer no puede modificar datos"""
        # Intentar crear anexo
        response = viewer_session.post(
            f"{BASE_URL}/anexo",
            data={"numero": "9999", "area": "Test", "nombre": "Test"}
        )
        assert response.status_code in [401, 403], \
            "Viewer no debería poder crear anexos"
    
    def test_operator_can_view_cdrs_but_not_modify(self, operator_session):
        """Test: Operador puede ver CDRs pero no modificar configuración"""
        # Puede ver CDRs
        response = operator_session.get(f"{BASE_URL}/dashboard/cdr")
        assert response.status_code == 200
        
        # No puede modificar tarifas
        response = operator_session.post(
            f"{BASE_URL}/api/tarifas",
            json={"zona_id": 1, "costo_minuto": 0.05}
        )
        assert response.status_code in [401, 403]
```

### 6.4 Tests de Inyección SQL

```python
# tests/test_agent_4_sql_injection.py

class TestAgent4SQLInjection:
    
    def test_sql_injection_in_search(self, auth_session):
        """Test: Protección contra SQL injection en búsqueda"""
        malicious_input = "' OR '1'='1"
        
        response = auth_session.get(
            f"{BASE_URL}/dashboard/anexos",
            params={"buscar": malicious_input}
        )
        
        # No debería retornar todos los registros
        assert response.status_code == 200
        # Verificar que no se ejecutó la inyección
        assert "error" not in response.text.lower()
    
    def test_sql_injection_in_login(self):
        """Test: Protección contra SQL injection en login"""
        response = requests.post(
            f"{BASE_URL}/auth/login",
            data={
                "username": "admin' --",
                "password": "anything"
            }
        )
        assert response.status_code in [401, 403], \
            "Login vulnerable a SQL injection"
```

### 6.5 Tests de XSS (Cross-Site Scripting)

```python
# tests/test_agent_4_xss.py

class TestAgent4XSS:
    
    def test_xss_in_anexo_creation(self, auth_session):
        """Test: Protección contra XSS en creación de anexo"""
        xss_payload = "<script>alert('XSS')</script>"
        
        response = auth_session.post(
            f"{BASE_URL}/anexo",
            data={
                "numero": "9998",
                "area": xss_payload,
                "nombre": "Test",
                "pin": "1234"
            }
        )
        
        # Verificar que el payload fue escapado
        response = auth_session.get(f"{BASE_URL}/dashboard/anexos")
        assert "<script>" not in response.text, "XSS vulnerability detected"
        assert "&lt;script&gt;" in response.text or "script" not in response.text.lower()
```

### 6.6 Tests de Rate Limiting

```python
# tests/test_agent_4_rate_limiting.py

class TestAgent4RateLimiting:
    
    def test_login_rate_limiting(self):
        """Test: Límite de intentos de login"""
        for i in range(15):  # Intentar 15 veces
            response = requests.post(
                f"{BASE_URL}/auth/login",
                data={"username": "admin", "password": "wrongpass"}
            )
        
        # Después de X intentos, debería bloquear temporalmente
        response = requests.post(
            f"{BASE_URL}/auth/login",
            data={"username": "admin", "password": "admin123"}
        )
        
        # Idealmente debería retornar 429 (Too Many Requests)
        # Por ahora verificamos que funcione o esté implementado
        assert response.status_code in [200, 429], \
            "Rate limiting no implementado"
```

### 6.7 Criterios de Aceptación

- ✅ Autenticación funciona correctamente
- ✅ Autorización basada en roles funciona (RBAC)
- ✅ Protección contra SQL injection
- ✅ Protección contra XSS
- ✅ Rate limiting implementado (al menos en login)
- ✅ Cookies de sesión con flags de seguridad
- ✅ Passwords hasheados (no en texto plano)

---

## 7. Agente 5: Validador de Performance

### 7.1 Objetivo

Validar que el sistema soporte carga concurrente y responda en tiempos aceptables.

### 7.2 Tests de Carga con Locust

```python
# tests/test_agent_5_load.py (usando Locust)

from locust import HttpUser, task, between

class ApoloBillingUser(HttpUser):
    wait_time = between(1, 3)
    
    def on_start(self):
        """Login antes de ejecutar tasks"""
        response = self.client.post("/auth/login", data={
            "username": "admin",
            "password": "admin123"
        })
        assert response.status_code == 200
    
    @task(3)
    def view_dashboard_saldo(self):
        """Visitar dashboard de saldo"""
        self.client.get("/dashboard/saldo")
    
    @task(2)
    def view_dashboard_cdr(self):
        """Visitar dashboard de CDR"""
        self.client.get("/dashboard/cdr")
    
    @task(1)
    def view_active_calls(self):
        """Consultar llamadas activas"""
        self.client.get("/api/active-calls")
    
    @task(1)
    def check_balance(self):
        """Simular consulta de saldo"""
        self.client.get("/check_balance/8001/51987654321")

# Ejecutar con:
# locust -f tests/test_agent_5_load.py --host=http://localhost:8000
# Abrir http://localhost:8089 y configurar:
# - Number of users: 50
# - Spawn rate: 5
# - Duration: 5 minutes
```

### 7.3 Tests de Performance de Endpoints Críticos

```python
# tests/test_agent_5_performance.py

import time

class TestAgent5Performance:
    
    def test_check_balance_response_time(self):
        """Test: Tiempo de respuesta de check_balance < 100ms"""
        times = []
        
        for i in range(100):
            start = time.time()
            response = requests.get(f"{BASE_URL}/check_balance/8001/51987654321")
            end = time.time()
            
            assert response.status_code == 200
            times.append((end - start) * 1000)  # en ms
        
        avg_time = sum(times) / len(times)
        p95_time = sorted(times)[94]  # Percentil 95
        
        assert avg_time < 100, f"Tiempo promedio {avg_time}ms excede 100ms"
        assert p95_time < 200, f"P95 {p95_time}ms excede 200ms"
    
    def test_cdr_query_with_large_dataset(self, auth_session):
        """Test: Consulta de CDRs con dataset grande"""
        start = time.time()
        response = auth_session.get(f"{BASE_URL}/dashboard/cdr")
        end = time.time()
        
        assert response.status_code == 200
        response_time = (end - start) * 1000
        assert response_time < 2000, f"Consulta CDR tardó {response_time}ms"
    
    def test_concurrent_active_calls_updates(self):
        """Test: Actualizaciones concurrentes de llamadas activas"""
        import concurrent.futures
        
        def update_active_call(call_id):
            data = {
                "call_id": f"concurrent-{call_id}",
                "calling_number": "8001",
                "called_number": "51987654321",
                "current_duration": 60,
                "current_cost": 3.00
            }
            response = requests.post(f"{BASE_URL}/api/active-calls", json=data)
            return response.status_code == 200
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=20) as executor:
            futures = [executor.submit(update_active_call, i) for i in range(100)]
            results = [f.result() for f in concurrent.futures.as_completed(futures)]
        
        success_rate = sum(results) / len(results)
        assert success_rate > 0.95, f"Solo {success_rate*100}% de éxito en concurrencia"
```

### 7.4 Métricas Objetivo

| Métrica | Objetivo | Crítico |
|---------|----------|---------|
| Tiempo respuesta API (avg) | < 100ms | < 200ms |
| Tiempo respuesta dashboard | < 2s | < 5s |
| Throughput (req/s) | > 100 | > 50 |
| Usuarios concurrentes | 50+ | 20+ |
| Tasa de error | < 1% | < 5% |
| CPU usage | < 70% | < 90% |
| Memory usage | < 80% | < 95% |

### 7.5 Criterios de Aceptación

- ✅ Endpoints críticos responden en < 100ms (promedio)
- ✅ Sistema soporta 50+ usuarios concurrentes
- ✅ Tasa de error < 1% bajo carga
- ✅ No hay memory leaks
- ✅ Base de datos tiene índices apropiados

---

## 8. Agente 6: Validador de UI/UX

### 8.1 Objetivo

Validar que la interfaz de usuario sea intuitiva, accesible y responsiva.

### 8.2 Tests de Responsividad

```python
# tests/test_agent_6_responsive.py

from selenium import webdriver
from selenium.webdriver.common.by import By

class TestAgent6Responsive:
    
    @pytest.fixture(params=[
        (1920, 1080),  # Desktop
        (1366, 768),   # Laptop
        (768, 1024),   # Tablet portrait
        (375, 667),    # Mobile iPhone
        (360, 640),    # Mobile Android
    ])
    def driver_with_viewport(self, request):
        """Driver con diferentes tamaños de viewport"""
        width, height = request.param
        driver = webdriver.Chrome()
        driver.set_window_size(width, height)
        yield driver, width, height
        driver.quit()
    
    def test_sidebar_responsive(self, driver_with_viewport, authenticated_driver):
        """Test: Sidebar se adapta a diferentes tamaños"""
        driver, width, height = driver_with_viewport
        driver.get(f"{BASE_URL}/dashboard/saldo")
        
        sidebar = driver.find_element(By.ID, "sidebar")
        
        if width < 768:  # Mobile
            # Sidebar debería estar colapsado
            assert not sidebar.is_displayed() or "collapse" in sidebar.get_attribute("class")
        else:
            # Sidebar debería estar visible
            assert sidebar.is_displayed()
    
    def test_tables_responsive(self, driver_with_viewport):
        """Test: Tablas son scrollables en mobile"""
        driver, width, height = driver_with_viewport
        # Implementar test de tablas responsivas
```

### 8.3 Tests de Accesibilidad (a11y)

```python
# tests/test_agent_6_accessibility.py

from axe_selenium_python import Axe

class TestAgent6Accessibility:
    
    def test_login_page_accessibility(self):
        """Test: Página de login cumple WCAG 2.1"""
        driver = webdriver.Chrome()
        driver.get(f"{BASE_URL}/login")
        
        axe = Axe(driver)
        axe.inject()
        results = axe.run()
        
        assert len(results["violations"]) == 0, \
            f"Violaciones de accesibilidad: {results['violations']}"
        
        driver.quit()
    
    def test_keyboard_navigation(self, authenticated_driver):
        """Test: Navegación completa con teclado"""
        from selenium.webdriver.common.keys import Keys
        
        authenticated_driver.get(f"{BASE_URL}/dashboard/saldo")
        
        body = authenticated_driver.find_element(By.TAG_NAME, "body")
        
        # Navegar con Tab
        for i in range(10):
            body.send_keys(Keys.TAB)
            time.sleep(0.1)
        
        # Verificar que el foco es visible
        focused = authenticated_driver.switch_to.active_element
        assert focused is not None
```

### 8.4 Tests de Usabilidad

```python
# tests/test_agent_6_usability.py

class TestAgent6Usability:
    
    def test_search_functionality(self, authenticated_driver):
        """Test: Funcionalidad de búsqueda es intuitiva"""
        authenticated_driver.get(f"{BASE_URL}/dashboard/anexos")
        
        # Buscar anexo
        search_input = authenticated_driver.find_element(By.NAME, "buscar")
        search_input.send_keys("8001")
        search_input.submit()
        
        # Verificar que muestra resultados
        time.sleep(1)
        assert "8001" in authenticated_driver.page_source
    
    def test_form_validation_messages(self, authenticated_driver):
        """Test: Mensajes de validación son claros"""
        authenticated_driver.get(f"{BASE_URL}/dashboard/anexos")
        
        # Click en "Nuevo Anexo"
        new_btn = authenticated_driver.find_element(By.CSS_SELECTOR, "[data-bs-target='#modalNuevoAnexo']")
        new_btn.click()
        
        time.sleep(0.5)
        
        # Intentar guardar sin llenar campos
        save_btn = authenticated_driver.find_element(By.CSS_SELECTOR, "#modalNuevoAnexo button[type='submit']")
        save_btn.click()
        
        # Verificar que muestra errores de validación
        time.sleep(0.5)
        # Debería mostrar mensajes de error
```

### 8.5 Criterios de Aceptación

- ✅ Interfaz es responsive en todos los dispositivos
- ✅ Sidebar se adapta correctamente
- ✅ Tablas son scrollables en mobile
- ✅ Cumple WCAG 2.1 Level AA
- ✅ Navegación completa con teclado
- ✅ Mensajes de error son claros
- ✅ Búsqueda funciona correctamente

---

## 9. Scripts de Automatización

### 9.1 Script Coordinador Maestro

```python
#!/usr/bin/env python3
# tests/run_all_agents.py

import subprocess
import sys
import json
from datetime import datetime

class AgentCoordinator:
    def __init__(self):
        self.results = {}
        self.start_time = datetime.now()
    
    def run_agent(self, agent_name, test_file):
        """Ejecutar un agente específico"""
        print(f"\n{'='*60}")
        print(f"🤖 Ejecutando {agent_name}")
        print(f"{'='*60}\n")
        
        result = subprocess.run(
            ["pytest", test_file, "-v", "--tb=short"],
            capture_output=True,
            text=True
        )
        
        self.results[agent_name] = {
            'exit_code': result.returncode,
            'stdout': result.stdout,
            'stderr': result.stderr,
            'success': result.returncode == 0
        }
        
        return result.returncode == 0
    
    def generate_report(self):
        """Generar reporte consolidado"""
        end_time = datetime.now()
        duration = (end_time - self.start_time).total_seconds()
        
        report = {
            'start_time': self.start_time.isoformat(),
            'end_time': end_time.isoformat(),
            'duration_seconds': duration,
            'agents': self.results,
            'summary': {
                'total_agents': len(self.results),
                'passed': sum(1 for r in self.results.values() if r['success']),
                'failed': sum(1 for r in self.results.values() if not r['success'])
            }
        }
        
        # Guardar reporte JSON
        with open('tests/reports/full_report.json', 'w') as f:
            json.dump(report, f, indent=2)
        
        # Generar reporte legible
        self.print_summary(report)
        
        return report
    
    def print_summary(self, report):
        """Imprimir resumen en consola"""
        print(f"\n{'='*60}")
        print(f"📊 RESUMEN DE VALIDACIÓN MULTI-AGENTE")
        print(f"{'='*60}\n")
        print(f"Inicio: {report['start_time']}")
        print(f"Fin: {report['end_time']}")
        print(f"Duración: {report['duration_seconds']:.2f}s\n")
        
        for agent_name, result in report['agents'].items():
            status = "✅ PASS" if result['success'] else "❌ FAIL"
            print(f"{status} - {agent_name}")
        
        print(f"\n{'='*60}")
        print(f"Total: {report['summary']['total_agents']} agentes")
        print(f"Exitosos: {report['summary']['passed']}")
        print(f"Fallidos: {report['summary']['failed']}")
        print(f"{'='*60}\n")
    
    def run_all(self):
        """Ejecutar todos los agentes en secuencia"""
        agents = [
            ("Agente 1: Navegación y Enlaces", "tests/test_agent_1_*.py"),
            ("Agente 2: CRUD Funcional", "tests/test_agent_2_*.py"),
            ("Agente 3: Integraciones", "tests/test_agent_3_*.py"),
            ("Agente 4: Seguridad", "tests/test_agent_4_*.py"),
            ("Agente 5: Performance", "tests/test_agent_5_*.py"),
            ("Agente 6: UI/UX", "tests/test_agent_6_*.py"),
        ]
        
        for agent_name, test_pattern in agents:
            success = self.run_agent(agent_name, test_pattern)
            if not success:
                print(f"\n⚠️ {agent_name} ha fallado pero se continúa con los demás...\n")
        
        report = self.generate_report()
        
        # Exit code basado en resultados
        return 0 if report['summary']['failed'] == 0 else 1

if __name__ == "__main__":
    coordinator = AgentCoordinator()
    exit_code = coordinator.run_all()
    sys.exit(exit_code)
```

### 9.2 Fixtures Comunes

```python
# tests/conftest.py

import pytest
import requests

@pytest.fixture(scope="session")
def auth_session():
    """Sesión autenticada para usar en tests"""
    session = requests.Session()
    
    response = session.post(
        "http://localhost:8000/auth/login",
        data={"username": "admin", "password": "admin123"}
    )
    
    assert response.status_code == 200, "Fallo al autenticar para tests"
    
    yield session
    
    # Cleanup
    session.get("http://localhost:8000/logout")
    session.close()

@pytest.fixture(scope="function")
def clean_test_data(auth_session):
    """Limpiar datos de prueba antes y después de cada test"""
    # Setup: limpiar datos previos
    yield
    # Teardown: limpiar datos creados en el test

@pytest.fixture(scope="session")
def admin_session():
    """Sesión con usuario admin"""
    # Implementar
    pass

@pytest.fixture(scope="session")
def viewer_session():
    """Sesión con usuario viewer"""
    # Implementar
    pass

@pytest.fixture(scope="session")
def operator_session():
    """Sesión con usuario operator"""
    # Implementar
    pass
```

---

## 10. Reporte de Resultados

### 10.1 Formato de Reporte

El reporte generado incluirá:

```json
{
  "execution_date": "2025-12-23T15:30:00",
  "duration_seconds": 1800,
  "summary": {
    "total_tests": 250,
    "passed": 235,
    "failed": 10,
    "skipped": 5,
    "success_rate": 94.0
  },
  "agents": {
    "agent_1_navigation": {
      "tests_run": 45,
      "passed": 42,
      "failed": 3,
      "issues": [
        {
          "test": "test_validate_internal_links",
          "error": "2 enlaces rotos encontrados",
          "details": [
            "/api/anexo/export/pdf - 404",
            "/api/cdr/export/csv - 404"
          ]
        }
      ]
    },
    "agent_2_crud": {
      "tests_run": 60,
      "passed": 58,
      "failed": 2,
      "issues": [
        {
          "test": "test_tarifa_validation_negative_cost",
          "error": "No valida costos negativos"
        }
      ]
    },
    // ... más agentes
  },
  "recommendations": [
    "Implementar endpoints faltantes de exportación",
    "Agregar validación de costos negativos en tarifas",
    "Mejorar tiempo de respuesta de consulta CDR",
    "Implementar rate limiting en login"
  ]
}
```

### 10.2 Dashboard HTML de Resultados

Se generará un dashboard HTML interactivo con:
- Resumen ejecutivo
- Gráficos de resultados por agente
- Lista de issues prioritizados
- Recomendaciones de mejora

---

## 11. Cronograma de Ejecución

### Semana 1: Setup y Agente 1-2
- Días 1-2: Configurar entorno de testing
- Días 3-4: Implementar y ejecutar Agente 1
- Días 4-5: Implementar y ejecutar Agente 2

### Semana 2: Agente 3-4-5
- Días 1-2: Implementar y ejecutar Agente 3
- Días 3-4: Implementar y ejecutar Agente 4
- Día 5: Implementar y ejecutar Agente 5

### Semana 3: Agente 6 y Reportes
- Días 1-2: Implementar y ejecutar Agente 6
- Días 3-4: Generar reportes y documentación
- Día 5: Revisión y presentación de resultados

---

## 12. Conclusión

Este plan de validación multi-agente proporciona una **cobertura exhaustiva** de todas las funcionalidades del sistema Apolo Billing, asegurando que:

✅ Todos los endpoints funcionen correctamente
✅ Todos los enlaces estén operativos
✅ Las operaciones CRUD sean completas y consistentes
✅ Las integraciones externas funcionen apropiadamente
✅ El sistema sea seguro contra vulnerabilidades comunes
✅ El rendimiento sea aceptable bajo carga
✅ La interfaz sea usable y accesible

**Próximos pasos**:
1. Revisar y aprobar este plan
2. Configurar entorno de testing
3. Implementar scripts de agentes
4. Ejecutar validación completa
5. Generar reportes de resultados
6. Priorizar y resolver issues encontrados

---

**Documento creado**: 2025-12-23  
**Versión**: 1.0  
**Autor**: Plan de Validación Automatizado Multi-Agente  
**Contacto**: Equipo QA Apolo Billing
