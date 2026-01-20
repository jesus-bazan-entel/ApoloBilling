import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchZones, createZone } from '../api/client'
import DataTable from '../components/DataTable'
import { Globe, Plus, X } from 'lucide-react'
import type { Zone } from '../types'

export default function ZonesPage() {
  const [showModal, setShowModal] = useState(false)
  const queryClient = useQueryClient()

  const { data: zones = [], isLoading } = useQuery({
    queryKey: ['zones'],
    queryFn: fetchZones,
    refetchInterval: 30000,
  })

  const createMutation = useMutation({
    mutationFn: createZone,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['zones'] })
      setShowModal(false)
    },
  })

  const columns = [
    {
      key: 'id',
      header: 'ID',
      render: (zone: Zone) => (
        <span className="font-mono text-slate-600">{zone.id}</span>
      ),
    },
    {
      key: 'zone_name',
      header: 'Nombre de Zona',
      render: (zone: Zone) => (
        <span className="font-medium text-slate-900">{zone.zone_name}</span>
      ),
    },
    {
      key: 'description',
      header: 'Descripción',
      render: (zone: Zone) => (
        <span className="text-slate-600">{zone.description}</span>
      ),
    },
  ]

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">
            Gestión de Zonas Geográficas
          </h1>
          <p className="text-slate-500">
            Define zonas para organizar tarifas por destino
          </p>
        </div>
        <button
          onClick={() => setShowModal(true)}
          className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Plus className="w-5 h-5 mr-2" />
          Nueva Zona
        </button>
      </div>

      {/* Summary Card */}
      <div className="bg-white rounded-lg shadow-sm p-6 border border-slate-200">
        <div className="flex items-center">
          <Globe className="w-8 h-8 text-blue-500 mr-4" />
          <div>
            <p className="text-sm text-slate-500">Total de Zonas</p>
            <p className="text-3xl font-bold text-slate-900">{zones.length}</p>
          </div>
        </div>
      </div>

      {/* Zones Table */}
      <DataTable
        columns={columns}
        data={zones}
        loading={isLoading}
        emptyMessage="No hay zonas geográficas registradas"
        searchable={true}
        searchPlaceholder="Buscar por nombre o descripción..."
      />

      {/* Create Modal */}
      {showModal && (
        <ZoneModal
          onClose={() => setShowModal(false)}
          onSubmit={(data) => createMutation.mutate(data)}
          isLoading={createMutation.isPending}
        />
      )}
    </div>
  )
}

interface ZoneModalProps {
  onClose: () => void
  onSubmit: (data: Partial<Zone>) => void
  isLoading: boolean
}

function ZoneModal({ onClose, onSubmit, isLoading }: ZoneModalProps) {
  const [formData, setFormData] = useState<Partial<Zone>>({
    zone_name: '',
    description: '',
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSubmit(formData)
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-xl max-w-lg w-full mx-4">
        <div className="flex items-center justify-between p-6 border-b border-slate-200">
          <h2 className="text-xl font-bold text-slate-900">Nueva Zona Geográfica</h2>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">
              Nombre de Zona *
            </label>
            <input
              type="text"
              required
              value={formData.zone_name}
              onChange={(e) =>
                setFormData({ ...formData, zone_name: e.target.value })
              }
              placeholder="Ej: Peru_Lima, Peru_Provincias, USA, Colombia"
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
            <p className="text-xs text-slate-500 mt-1">
              Usa formato: País_Ciudad o País_Región
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">
              Descripción *
            </label>
            <textarea
              required
              rows={3}
              value={formData.description}
              onChange={(e) =>
                setFormData({ ...formData, description: e.target.value })
              }
              placeholder="Descripción detallada de la zona geográfica"
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <div className="flex justify-end space-x-3 pt-4 border-t border-slate-200">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50"
            >
              Cancelar
            </button>
            <button
              type="submit"
              disabled={isLoading}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
            >
              {isLoading ? 'Creando...' : 'Crear Zona'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
