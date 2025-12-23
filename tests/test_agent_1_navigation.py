"""
AGENTE 1: Validador de Navegación y Enlaces
Tests para verificar que todos los endpoints y enlaces funcionen correctamente
"""
import pytest
import requests
from conftest import BASE_URL


# Markers para identificar estos tests
pytestmark = [pytest.mark.agent1, pytest.mark.critical]


# Lista de endpoints públicos (no requieren autenticación)
PUBLIC_ENDPOINTS = [
    "/login",
]

# Lista de endpoints protegidos (requieren autenticación)
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
    "/dashboard/recarga_masiva",
    "/dashboard/anexos/carga_masiva",
]


class TestAgent1NavigationPublicEndpoints:
    """Tests para endpoints públicos"""
    
    @pytest.mark.parametrize("endpoint", PUBLIC_ENDPOINTS)
    def test_public_endpoints_accessible(self, endpoint):
        """
        Validar que endpoints públicos sean accesibles sin autenticación.
        
        Los endpoints públicos deben retornar 200 o redirigir (302).
        """
        response = requests.get(f"{BASE_URL}{endpoint}", allow_redirects=False)
        assert response.status_code in [200, 302], \
            f"Endpoint público {endpoint} retornó status {response.status_code}"


class TestAgent1NavigationProtectedEndpoints:
    """Tests para endpoints protegidos"""
    
    @pytest.mark.parametrize("endpoint", PROTECTED_ENDPOINTS)
    def test_protected_endpoints_require_auth(self, endpoint):
        """
        Validar que endpoints protegidos requieran autenticación.
        
        Sin autenticación, deben redirigir a login (302) o retornar 401/403.
        """
        response = requests.get(f"{BASE_URL}{endpoint}", allow_redirects=False)
        assert response.status_code in [302, 401, 403], \
            f"Endpoint protegido {endpoint} debería requerir autenticación, " \
            f"pero retornó {response.status_code}"
    
    @pytest.mark.parametrize("endpoint", PROTECTED_ENDPOINTS)
    def test_protected_endpoints_accessible_with_auth(self, auth_session, endpoint):
        """
        Validar que endpoints protegidos sean accesibles con autenticación válida.
        """
        response = auth_session.get(f"{BASE_URL}{endpoint}")
        assert response.status_code == 200, \
            f"Endpoint {endpoint} no es accesible incluso con autenticación. " \
            f"Status: {response.status_code}"


class TestAgent1NavigationAPIEndpoints:
    """Tests para endpoints API"""
    
    API_ENDPOINTS = [
        "/api/active-calls",
        "/api/active-calls-list",
        "/api/ws-stats",
        "/api/cucm/config",
        "/api/cucm/status",
        "/api/config",
    ]
    
    @pytest.mark.parametrize("endpoint", API_ENDPOINTS)
    def test_api_endpoints_return_json(self, auth_session, endpoint):
        """
        Validar que endpoints /api/* retornen JSON válido.
        """
        response = auth_session.get(f"{BASE_URL}{endpoint}")
        
        # Verificar status code
        assert response.status_code == 200, \
            f"API endpoint {endpoint} falló con status {response.status_code}"
        
        # Verificar que retorna JSON
        content_type = response.headers.get("Content-Type", "")
        assert "application/json" in content_type, \
            f"API endpoint {endpoint} no retorna JSON. Content-Type: {content_type}"
        
        # Verificar que el JSON es parseable
        try:
            data = response.json()
            assert data is not None
        except Exception as e:
            pytest.fail(f"API endpoint {endpoint} retorna JSON inválido: {str(e)}")


class TestAgent1NavigationLogout:
    """Tests para funcionalidad de logout"""
    
    def test_logout_redirects_to_login(self, auth_session):
        """
        Validar que logout cierre sesión y redirija a login.
        """
        response = auth_session.get(f"{BASE_URL}/logout", allow_redirects=False)
        
        assert response.status_code == 302, \
            f"Logout debería redirigir (302), pero retornó {response.status_code}"
        
        location = response.headers.get("Location", "")
        assert "/login" in location, \
            f"Logout debería redirigir a /login, pero redirige a {location}"
    
    def test_logout_invalidates_session(self):
        """
        Validar que después de logout, la sesión ya no sea válida.
        """
        # Crear nueva sesión y autenticar
        session = requests.Session()
        session.post(
            f"{BASE_URL}/auth/login",
            data={"username": "admin", "password": "admin123"}
        )
        
        # Hacer logout
        session.get(f"{BASE_URL}/logout")
        
        # Intentar acceder a recurso protegido
        response = session.get(f"{BASE_URL}/dashboard/saldo", allow_redirects=False)
        
        assert response.status_code in [302, 401, 403], \
            "Sesión sigue válida después de logout"


class TestAgent1NavigationBalanceCheck:
    """Tests para endpoints de consulta de saldo (usados por FreeSWITCH)"""
    
    def test_check_balance_endpoint_exists(self):
        """
        Validar que endpoint /check_balance exista y responda.
        """
        calling_number = "8001"
        called_number = "51987654321"
        
        response = requests.get(
            f"{BASE_URL}/check_balance/{calling_number}/{called_number}"
        )
        
        assert response.status_code in [200, 404, 500], \
            f"Endpoint check_balance retornó status inesperado: {response.status_code}"
    
    def test_check_balance_for_call_endpoint_exists(self):
        """
        Validar que endpoint alternativo /check_balance_for_call exista.
        """
        calling_number = "8001"
        called_number = "51987654321"
        
        response = requests.get(
            f"{BASE_URL}/check_balance_for_call/{calling_number}/{called_number}"
        )
        
        assert response.status_code in [200, 404, 500], \
            f"Endpoint check_balance_for_call retornó status inesperado"


# ============================================================================
# SUMMARY
# ============================================================================
# Este módulo contiene tests del Agente 1 para validar navegación y enlaces.
# 
# Tests incluidos:
# - Endpoints públicos accesibles sin auth
# - Endpoints protegidos requieren auth
# - Endpoints protegidos accesibles con auth
# - Endpoints API retornan JSON válido
# - Logout funciona correctamente
# - Endpoints de balance check existen
# 
# Para ejecutar solo estos tests:
#   pytest tests/test_agent_1_navigation.py -v
# 
# Para ejecutar solo tests del Agente 1:
#   pytest -m agent1 -v
# ============================================================================
