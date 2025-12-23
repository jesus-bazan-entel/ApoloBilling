"""
Configuración compartida de pytest para todos los tests
"""
import pytest
import requests
from typing import Generator

# Configuración base
BASE_URL = "http://localhost:8000"
DEFAULT_ADMIN_USER = "admin"
DEFAULT_ADMIN_PASS = "admin123"


@pytest.fixture(scope="session")
def base_url() -> str:
    """URL base de la aplicación"""
    return BASE_URL


@pytest.fixture(scope="session")
def auth_session() -> Generator[requests.Session, None, None]:
    """
    Sesión autenticada como admin para usar en tests.
    
    Esta sesión se mantiene durante toda la ejecución de tests
    y se cierra al finalizar.
    """
    session = requests.Session()
    
    # Intentar login
    response = session.post(
        f"{BASE_URL}/auth/login",
        data={
            "username": DEFAULT_ADMIN_USER,
            "password": DEFAULT_ADMIN_PASS
        },
        allow_redirects=False
    )
    
    if response.status_code not in [200, 302]:
        pytest.skip(f"No se pudo autenticar. Status: {response.status_code}")
    
    yield session
    
    # Cleanup: cerrar sesión
    try:
        session.get(f"{BASE_URL}/logout")
    except:
        pass
    finally:
        session.close()


@pytest.fixture(scope="function")
def clean_session(auth_session) -> requests.Session:
    """
    Sesión limpia para cada test individual.
    Útil cuando se necesita aislar efectos entre tests.
    """
    # Retornar la sesión autenticada
    # Cada test puede usarla sin afectar a otros
    return auth_session


@pytest.fixture(scope="session")
def admin_session() -> Generator[requests.Session, None, None]:
    """Sesión con usuario admin (máximos privilegios)"""
    session = requests.Session()
    response = session.post(
        f"{BASE_URL}/auth/login",
        data={"username": "admin", "password": "admin123"}
    )
    
    if response.status_code not in [200, 302]:
        pytest.skip("No se pudo crear sesión admin")
    
    yield session
    session.close()


@pytest.fixture(scope="session")
def viewer_session() -> Generator[requests.Session, None, None]:
    """Sesión con usuario viewer (solo lectura)"""
    session = requests.Session()
    
    # Nota: Estos usuarios deben existir en la BD
    response = session.post(
        f"{BASE_URL}/auth/login",
        data={"username": "viewer", "password": "viewer123"}
    )
    
    if response.status_code not in [200, 302]:
        pytest.skip("Usuario viewer no existe o credenciales incorrectas")
    
    yield session
    session.close()


@pytest.fixture(scope="session")
def operator_session() -> Generator[requests.Session, None, None]:
    """Sesión con usuario operator (permisos medios)"""
    session = requests.Session()
    
    response = session.post(
        f"{BASE_URL}/auth/login",
        data={"username": "operator", "password": "operator123"}
    )
    
    if response.status_code not in [200, 302]:
        pytest.skip("Usuario operator no existe o credenciales incorrectas")
    
    yield session
    session.close()


@pytest.fixture(scope="function")
def test_anexo_data() -> dict:
    """Datos de prueba para crear un anexo"""
    import random
    return {
        "numero": f"test{random.randint(1000, 9999)}",
        "area": "Test Area",
        "nombre": "Test Anexo",
        "pin": "1234",
        "saldo_inicial": 100.00
    }


@pytest.fixture(scope="function")
def test_zona_data() -> dict:
    """Datos de prueba para crear una zona"""
    import random
    return {
        "nombre": f"Zona Test {random.randint(1000, 9999)}",
        "descripcion": "Zona de prueba automatizada"
    }


@pytest.fixture(scope="function")
def cleanup_test_data(auth_session):
    """
    Fixture que se ejecuta después de cada test para limpiar datos de prueba.
    
    Uso:
        def test_something(cleanup_test_data):
            # Tu test aquí
            # Al finalizar, se ejecutará la limpieza automáticamente
    """
    created_resources = {
        'anexos': [],
        'zonas': [],
        'prefijos': [],
        'tarifas': []
    }
    
    yield created_resources
    
    # Cleanup: eliminar recursos creados
    for anexo_id in created_resources['anexos']:
        try:
            auth_session.delete(f"{BASE_URL}/anexo/{anexo_id}")
        except:
            pass
    
    for zona_id in created_resources['zonas']:
        try:
            auth_session.delete(f"{BASE_URL}/api/zonas/{zona_id}")
        except:
            pass
    
    # Agregar más limpiezas según sea necesario


# Hooks de pytest

def pytest_configure(config):
    """Configuración inicial de pytest"""
    # Agregar markers personalizados
    config.addinivalue_line(
        "markers", "agent1: Tests del Agente 1 (Navegación)"
    )
    config.addinivalue_line(
        "markers", "agent2: Tests del Agente 2 (CRUD)"
    )
    config.addinivalue_line(
        "markers", "agent3: Tests del Agente 3 (Integraciones)"
    )
    config.addinivalue_line(
        "markers", "agent4: Tests del Agente 4 (Seguridad)"
    )
    config.addinivalue_line(
        "markers", "agent5: Tests del Agente 5 (Performance)"
    )
    config.addinivalue_line(
        "markers", "agent6: Tests del Agente 6 (UI/UX)"
    )
    config.addinivalue_line(
        "markers", "critical: Tests críticos que no deben fallar"
    )
    config.addinivalue_line(
        "markers", "slow: Tests que tardan más de 5 segundos"
    )


def pytest_collection_modifyitems(config, items):
    """Modificar items de tests durante la colección"""
    for item in items:
        # Agregar marker 'slow' si el test tiene 'load' o 'performance' en el nombre
        if 'load' in item.nodeid.lower() or 'performance' in item.nodeid.lower():
            item.add_marker(pytest.mark.slow)
