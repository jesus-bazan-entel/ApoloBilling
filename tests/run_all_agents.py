#!/usr/bin/env python3
"""
Coordinador Maestro de Validación Multi-Agente
Ejecuta todos los agentes de testing del sistema Apolo Billing
"""

import subprocess
import sys
import json
import os
from datetime import datetime
from pathlib import Path


class Colors:
    """Colores ANSI para output en terminal"""
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'


class AgentCoordinator:
    """Coordinador que ejecuta todos los agentes de testing"""
    
    def __init__(self):
        self.results = {}
        self.start_time = datetime.now()
        self.reports_dir = Path("tests/reports")
        self.reports_dir.mkdir(parents=True, exist_ok=True)
    
    def print_header(self, text: str):
        """Imprimir header colorido"""
        print(f"\n{Colors.HEADER}{'='*70}{Colors.ENDC}")
        print(f"{Colors.HEADER}{Colors.BOLD}{text.center(70)}{Colors.ENDC}")
        print(f"{Colors.HEADER}{'='*70}{Colors.ENDC}\n")
    
    def print_agent_header(self, agent_name: str):
        """Imprimir header de agente"""
        print(f"\n{Colors.OKCYAN}{'─'*70}{Colors.ENDC}")
        print(f"{Colors.OKCYAN}🤖 {agent_name}{Colors.ENDC}")
        print(f"{Colors.OKCYAN}{'─'*70}{Colors.ENDC}\n")
    
    def run_agent(self, agent_name: str, test_pattern: str, marker: str = None) -> bool:
        """
        Ejecutar un agente específico.
        
        Args:
            agent_name: Nombre descriptivo del agente
            test_pattern: Patrón de archivos de test (ej: "tests/test_agent_1_*.py")
            marker: Pytest marker opcional (ej: "agent1")
        
        Returns:
            True si todos los tests pasaron, False si alguno falló
        """
        self.print_agent_header(agent_name)
        
        # Construir comando pytest
        cmd = ["pytest", test_pattern, "-v", "--tb=short", "--color=yes"]
        
        if marker:
            cmd.extend(["-m", marker])
        
        # Agregar opciones adicionales
        cmd.extend([
            "--maxfail=5",  # Parar después de 5 fallos
            "-ra",  # Mostrar resumen de todos los tests
        ])
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300  # 5 minutos timeout por agente
            )
            
            self.results[agent_name] = {
                'exit_code': result.returncode,
                'stdout': result.stdout,
                'stderr': result.stderr,
                'success': result.returncode == 0
            }
            
            # Mostrar output
            if result.stdout:
                print(result.stdout)
            
            if result.stderr and result.returncode != 0:
                print(f"{Colors.FAIL}{result.stderr}{Colors.ENDC}")
            
            if result.returncode == 0:
                print(f"{Colors.OKGREEN}✅ {agent_name} completado exitosamente{Colors.ENDC}")
            else:
                print(f"{Colors.FAIL}❌ {agent_name} falló{Colors.ENDC}")
            
            return result.returncode == 0
            
        except subprocess.TimeoutExpired:
            print(f"{Colors.FAIL}⏱️  {agent_name} excedió el timeout de 5 minutos{Colors.ENDC}")
            self.results[agent_name] = {
                'exit_code': -1,
                'success': False,
                'error': 'Timeout'
            }
            return False
        
        except Exception as e:
            print(f"{Colors.FAIL}💥 Error ejecutando {agent_name}: {str(e)}{Colors.ENDC}")
            self.results[agent_name] = {
                'exit_code': -1,
                'success': False,
                'error': str(e)
            }
            return False
    
    def generate_report(self):
        """Generar reporte consolidado en JSON y texto"""
        end_time = datetime.now()
        duration = (end_time - self.start_time).total_seconds()
        
        report = {
            'execution_date': self.start_time.isoformat(),
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
        json_path = self.reports_dir / f"full_report_{self.start_time.strftime('%Y%m%d_%H%M%S')}.json"
        with open(json_path, 'w') as f:
            json.dump(report, f, indent=2)
        
        print(f"\n{Colors.OKBLUE}📄 Reporte JSON guardado en: {json_path}{Colors.ENDC}")
        
        # Generar y guardar reporte de texto
        text_report = self.generate_text_report(report)
        text_path = self.reports_dir / f"summary_{self.start_time.strftime('%Y%m%d_%H%M%S')}.txt"
        with open(text_path, 'w') as f:
            f.write(text_report)
        
        print(f"{Colors.OKBLUE}📄 Reporte de texto guardado en: {text_path}{Colors.ENDC}")
        
        # Imprimir resumen en consola
        self.print_summary(report)
        
        return report
    
    def generate_text_report(self, report: dict) -> str:
        """Generar reporte en formato texto"""
        lines = []
        lines.append("=" * 70)
        lines.append("REPORTE DE VALIDACIÓN MULTI-AGENTE - APOLO BILLING")
        lines.append("=" * 70)
        lines.append("")
        lines.append(f"Fecha de ejecución: {report['start_time']}")
        lines.append(f"Duración total: {report['duration_seconds']:.2f} segundos")
        lines.append("")
        lines.append("RESUMEN:")
        lines.append(f"  Total de agentes ejecutados: {report['summary']['total_agents']}")
        lines.append(f"  Agentes exitosos: {report['summary']['passed']}")
        lines.append(f"  Agentes fallidos: {report['summary']['failed']}")
        lines.append("")
        lines.append("DETALLE POR AGENTE:")
        lines.append("-" * 70)
        
        for agent_name, result in report['agents'].items():
            status = "✅ PASS" if result['success'] else "❌ FAIL"
            lines.append(f"{status} - {agent_name}")
            if not result['success']:
                error = result.get('error', 'Ver logs para más detalles')
                lines.append(f"       Error: {error}")
        
        lines.append("")
        lines.append("=" * 70)
        
        return "\n".join(lines)
    
    def print_summary(self, report: dict):
        """Imprimir resumen colorido en consola"""
        self.print_header("RESUMEN DE VALIDACIÓN MULTI-AGENTE")
        
        print(f"{Colors.BOLD}Fecha:{Colors.ENDC} {report['start_time']}")
        print(f"{Colors.BOLD}Duración:{Colors.ENDC} {report['duration_seconds']:.2f}s\n")
        
        summary = report['summary']
        total = summary['total_agents']
        passed = summary['passed']
        failed = summary['failed']
        
        print(f"{Colors.BOLD}Resultados:{Colors.ENDC}")
        print(f"  Total de agentes: {total}")
        print(f"  {Colors.OKGREEN}✅ Exitosos: {passed}{Colors.ENDC}")
        print(f"  {Colors.FAIL}❌ Fallidos: {failed}{Colors.ENDC}\n")
        
        print(f"{Colors.BOLD}Detalle por agente:{Colors.ENDC}")
        for agent_name, result in report['agents'].items():
            if result['success']:
                print(f"  {Colors.OKGREEN}✅{Colors.ENDC} {agent_name}")
            else:
                print(f"  {Colors.FAIL}❌{Colors.ENDC} {agent_name}")
        
        print(f"\n{Colors.HEADER}{'='*70}{Colors.ENDC}\n")
        
        if failed == 0:
            print(f"{Colors.OKGREEN}{Colors.BOLD}🎉 ¡Todos los tests pasaron exitosamente!{Colors.ENDC}\n")
        else:
            print(f"{Colors.WARNING}{Colors.BOLD}⚠️  Algunos tests fallaron. Revisa los reportes para más detalles.{Colors.ENDC}\n")
    
    def run_all(self, quick_mode: bool = False):
        """
        Ejecutar todos los agentes en secuencia.
        
        Args:
            quick_mode: Si es True, solo ejecuta tests críticos y rápidos
        """
        self.print_header("SISTEMA DE VALIDACIÓN MULTI-AGENTE")
        
        if quick_mode:
            print(f"{Colors.WARNING}⚡ Modo rápido activado - Solo tests críticos{Colors.ENDC}\n")
        
        # Definir agentes a ejecutar
        agents = [
            ("Agente 1: Navegación y Enlaces", "tests/test_agent_1_*.py", "agent1"),
            ("Agente 2: Funcionalidades CRUD", "tests/test_agent_2_*.py", "agent2"),
            # Agente 3, 4, 5, 6 se implementarán después
        ]
        
        # Si es modo rápido, solo ejecutar tests críticos
        if quick_mode:
            agents = [
                (name, pattern, f"{marker} and critical") 
                for name, pattern, marker in agents
            ]
        
        # Ejecutar cada agente
        all_passed = True
        for agent_name, test_pattern, marker in agents:
            success = self.run_agent(agent_name, test_pattern, marker)
            if not success:
                all_passed = False
                print(f"\n{Colors.WARNING}⚠️  {agent_name} falló, pero continuamos con los demás...{Colors.ENDC}\n")
        
        # Generar reporte final
        report = self.generate_report()
        
        # Exit code basado en resultados
        return 0 if all_passed else 1


def main():
    """Función principal"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Sistema de Validación Multi-Agente para Apolo Billing"
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="Ejecutar solo tests críticos (modo rápido)"
    )
    parser.add_argument(
        "--agent",
        type=int,
        choices=[1, 2, 3, 4, 5, 6],
        help="Ejecutar solo un agente específico (1-6)"
    )
    
    args = parser.parse_args()
    
    coordinator = AgentCoordinator()
    
    if args.agent:
        # Ejecutar solo un agente específico
        agent_map = {
            1: ("Agente 1: Navegación y Enlaces", "tests/test_agent_1_*.py", "agent1"),
            2: ("Agente 2: Funcionalidades CRUD", "tests/test_agent_2_*.py", "agent2"),
            # Agregar más agentes aquí
        }
        
        name, pattern, marker = agent_map[args.agent]
        success = coordinator.run_agent(name, pattern, marker)
        coordinator.generate_report()
        return 0 if success else 1
    else:
        # Ejecutar todos los agentes
        return coordinator.run_all(quick_mode=args.quick)


if __name__ == "__main__":
    sys.exit(main())
