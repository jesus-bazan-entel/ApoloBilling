import { Link, useLocation, useNavigate } from 'react-router-dom'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { checkHealth, getCurrentUser, logout } from '../api/client'
import {
  LayoutDashboard,
  Users,
  FileText,
  Phone,
  DollarSign,
  Settings,
  Globe,
  Activity,
  LogOut,
  User,
} from 'lucide-react'

const navigation = [
  { name: 'Panel de Control', href: '/', icon: LayoutDashboard },
  { name: 'Llamadas Activas', href: '/calls', icon: Phone },
  { name: 'Registros CDR', href: '/cdr', icon: FileText },
  { name: 'Cuentas', href: '/accounts', icon: Users },
  { name: 'Saldos', href: '/balance', icon: DollarSign },
  { name: 'Zonas', href: '/zones', icon: Globe },
  { name: 'Tarifas', href: '/rates', icon: Settings },
]

interface LayoutProps {
  children: React.ReactNode
}

export default function Layout({ children }: LayoutProps) {
  const location = useLocation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const { data: health } = useQuery({
    queryKey: ['health'],
    queryFn: checkHealth,
    refetchInterval: 5000,
    retry: false,
  })

  const { data: currentUser } = useQuery({
    queryKey: ['currentUser'],
    queryFn: getCurrentUser,
    retry: false,
  })

  const isOnline = health?.status === 'ok' || health?.status === 'healthy'

  const handleLogout = async () => {
    try {
      await logout()
      queryClient.clear()
      navigate('/login')
    } catch (error) {
      console.error('Error al cerrar sesión:', error)
    }
  }

  return (
    <div className="min-h-screen bg-slate-100">
      {/* Sidebar */}
      <aside className="fixed inset-y-0 left-0 w-64 bg-gradient-to-b from-slate-900 to-slate-800 text-white shadow-xl">
        <div className="flex items-center h-16 px-6 border-b border-slate-700">
          <span className="text-2xl font-bold text-blue-400">Apolo</span>
          <span className="text-2xl font-light text-slate-300 ml-1">Billing</span>
        </div>

        <nav className="p-4 space-y-1">
          {navigation.map((item) => {
            const isActive = location.pathname === item.href
            const Icon = item.icon

            return (
              <Link
                key={item.name}
                to={item.href}
                className={`flex items-center px-4 py-3 rounded-lg transition-all duration-200 ${
                  isActive
                    ? 'bg-blue-600 text-white shadow-lg scale-105'
                    : 'text-slate-300 hover:bg-slate-800 hover:text-white hover:translate-x-1'
                }`}
              >
                <Icon className="w-5 h-5 mr-3" />
                <span className="font-medium">{item.name}</span>
              </Link>
            )
          })}
        </nav>

        {/* Status indicator */}
        <div className="absolute bottom-0 left-0 right-0 p-4 border-t border-slate-700 bg-slate-900">
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center">
              <Activity className="w-4 h-4 mr-2" />
              <span className="text-slate-400">Motor de Facturación</span>
            </div>
            <div className="flex items-center">
              <div
                className={`w-2 h-2 rounded-full mr-2 ${
                  isOnline ? 'bg-green-500 animate-pulse' : 'bg-red-500'
                }`}
              />
              <span className={isOnline ? 'text-green-400' : 'text-red-400'}>
                {isOnline ? 'En Línea' : 'Fuera de Línea'}
              </span>
            </div>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className="ml-64 min-h-screen">
        {/* Header with user info and logout */}
        <header className="bg-white shadow-sm border-b border-slate-200 sticky top-0 z-10">
          <div className="flex items-center justify-end h-14 px-6">
            {currentUser && (
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2 text-slate-600">
                  <User className="w-4 h-4" />
                  <span className="text-sm font-medium">{currentUser.username}</span>
                  <span className="text-xs px-2 py-0.5 bg-blue-100 text-blue-700 rounded-full">
                    {currentUser.role}
                  </span>
                </div>
                <button
                  onClick={handleLogout}
                  className="flex items-center gap-2 px-3 py-1.5 text-sm text-slate-600 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                >
                  <LogOut className="w-4 h-4" />
                  <span>Cerrar Sesión</span>
                </button>
              </div>
            )}
          </div>
        </header>
        <div className="p-6">{children}</div>
      </main>
    </div>
  )
}
