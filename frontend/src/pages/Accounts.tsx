import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchAccounts, createAccount, updateAccount, deleteAccount } from '../api/client'
import DataTable from '../components/DataTable'
import Badge from '../components/Badge'
import { Users, Plus, Edit, X, DollarSign, Trash2, Power, AlertTriangle } from 'lucide-react'
import type { Account, AccountType, AccountStatus } from '../types'

export default function AccountsPage() {
  const [showModal, setShowModal] = useState(false)
  const [editingAccount, setEditingAccount] = useState<Account | null>(null)
  const [deleteConfirm, setDeleteConfirm] = useState<Account | null>(null)
  const queryClient = useQueryClient()

  const { data: accounts = [], isLoading } = useQuery({
    queryKey: ['accounts'],
    queryFn: fetchAccounts,
    refetchInterval: 10000,
  })

  const createMutation = useMutation({
    mutationFn: createAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
      setShowModal(false)
      setEditingAccount(null)
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Account> }) =>
      updateAccount(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
      setShowModal(false)
      setEditingAccount(null)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: deleteAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
      setDeleteConfirm(null)
    },
  })

  const toggleStatusMutation = useMutation({
    mutationFn: ({ id, currentStatus }: { id: number; currentStatus: string }) => {
      const newStatus = currentStatus?.toLowerCase() === 'active' ? 'suspended' : 'active'
      return updateAccount(id, { status: newStatus })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })

  const handleEdit = (account: Account) => {
    setEditingAccount(account)
    setShowModal(true)
  }

  const handleCreate = () => {
    setEditingAccount(null)
    setShowModal(true)
  }

  const columns = [
    {
      key: 'account_number',
      header: 'Número de Cuenta',
      render: (acc: Account) => (
        <span className="font-mono font-medium text-slate-900">
          {acc.account_number}
        </span>
      ),
    },
    {
      key: 'account_type',
      header: 'Tipo',
      render: (acc: Account) => {
        const isPrepaid = acc.account_type?.toLowerCase() === 'prepaid'
        return (
          <Badge variant={isPrepaid ? 'info' : 'warning'}>
            {isPrepaid ? 'Prepago' : 'Postpago'}
          </Badge>
        )
      },
    },
    {
      key: 'status',
      header: 'Estado',
      render: (acc: Account) => {
        const status = acc.status?.toLowerCase()
        return (
          <Badge
            variant={
              status === 'active'
                ? 'success'
                : status === 'suspended'
                ? 'warning'
                : 'error'
            }
          >
            {status === 'active'
              ? 'Activa'
              : status === 'suspended'
              ? 'Suspendida'
              : 'Cerrada'}
          </Badge>
        )
      },
    },
    {
      key: 'balance',
      header: 'Saldo',
      render: (acc: Account) => {
        const balance = Number(acc.balance) || 0
        return (
          <span
            className={`font-mono font-bold ${
              balance > 0 ? 'text-green-600' : 'text-red-600'
            }`}
          >
            ${balance.toFixed(2)}
          </span>
        )
      },
      className: 'text-right',
    },
    {
      key: 'credit_limit',
      header: 'Límite de Crédito',
      render: (acc: Account) => {
        const isPrepaid = acc.account_type?.toLowerCase() === 'prepaid'
        const creditLimit = Number(acc.credit_limit) || 0
        return (
          <span className="font-mono text-slate-700">
            {!isPrepaid ? `$${creditLimit.toFixed(2)}` : '-'}
          </span>
        )
      },
      className: 'text-right',
    },
    {
      key: 'max_concurrent_calls',
      header: 'Llamadas Concurrentes',
      render: (acc: Account) => (
        <span className="text-slate-700">{acc.max_concurrent_calls}</span>
      ),
      className: 'text-center',
    },
    {
      key: 'created_at',
      header: 'Creada',
      render: (acc: Account) =>
        new Date(acc.created_at).toLocaleDateString('es-PE'),
    },
    {
      key: 'actions',
      header: 'Acciones',
      render: (acc: Account) => {
        const isActive = acc.status?.toLowerCase() === 'active'
        const isToggling = toggleStatusMutation.isPending
        return (
          <div className="flex items-center gap-1">
            {/* Editar */}
            <button
              onClick={() => handleEdit(acc)}
              className="p-1.5 text-slate-500 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
              title="Editar cuenta"
            >
              <Edit className="w-4 h-4" />
            </button>

            {/* Activar/Desactivar */}
            <button
              onClick={() => toggleStatusMutation.mutate({ id: acc.id, currentStatus: acc.status })}
              disabled={isToggling}
              className={`p-1.5 rounded transition-colors ${
                isActive
                  ? 'text-slate-500 hover:text-amber-600 hover:bg-amber-50'
                  : 'text-amber-600 hover:text-green-600 hover:bg-green-50'
              }`}
              title={isActive ? 'Suspender cuenta' : 'Activar cuenta'}
            >
              <Power className="w-4 h-4" />
            </button>

            {/* Eliminar */}
            <button
              onClick={() => setDeleteConfirm(acc)}
              className="p-1.5 text-slate-500 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
              title="Eliminar cuenta"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          </div>
        )
      },
    },
  ]

  // Summary stats
  const totalBalance = accounts.reduce((sum, acc) => sum + (Number(acc.balance) || 0), 0)
  const activeAccounts = accounts.filter((acc) => acc.status?.toLowerCase() === 'active').length
  const prepaidAccounts = accounts.filter((acc) => acc.account_type?.toLowerCase() === 'prepaid').length
  const postpaidAccounts = accounts.filter((acc) => acc.account_type?.toLowerCase() === 'postpaid').length

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">
            Gestión de Cuentas
          </h1>
          <p className="text-slate-500">
            Administra cuentas prepago y postpago
          </p>
        </div>
        <button
          onClick={handleCreate}
          className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Plus className="w-5 h-5 mr-2" />
          Nueva Cuenta
        </button>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white rounded-lg shadow-sm p-4 border border-slate-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-slate-500">Total Cuentas</p>
              <p className="text-2xl font-bold text-slate-900">{accounts.length}</p>
            </div>
            <Users className="w-8 h-8 text-blue-500" />
          </div>
        </div>
        <div className="bg-white rounded-lg shadow-sm p-4 border border-slate-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-slate-500">Activas</p>
              <p className="text-2xl font-bold text-green-600">{activeAccounts}</p>
            </div>
            <Users className="w-8 h-8 text-green-500" />
          </div>
        </div>
        <div className="bg-white rounded-lg shadow-sm p-4 border border-slate-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-slate-500">Prepago / Postpago</p>
              <p className="text-2xl font-bold text-slate-900">
                {prepaidAccounts} / {postpaidAccounts}
              </p>
            </div>
            <DollarSign className="w-8 h-8 text-purple-500" />
          </div>
        </div>
        <div className="bg-white rounded-lg shadow-sm p-4 border border-slate-200">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-slate-500">Saldo Total</p>
              <p className="text-2xl font-bold text-green-600">
                ${totalBalance.toFixed(2)}
              </p>
            </div>
            <DollarSign className="w-8 h-8 text-green-500" />
          </div>
        </div>
      </div>

      {/* Accounts Table */}
      <DataTable
        columns={columns}
        data={accounts}
        loading={isLoading}
        emptyMessage="No hay cuentas registradas"
        searchable={true}
        searchPlaceholder="Buscar por número de cuenta..."
      />

      {/* Create/Edit Modal */}
      {showModal && (
        <AccountModal
          account={editingAccount}
          onClose={() => {
            setShowModal(false)
            setEditingAccount(null)
          }}
          onSubmit={(data) => {
            if (editingAccount) {
              updateMutation.mutate({ id: editingAccount.id, data })
            } else {
              createMutation.mutate(data)
            }
          }}
          isLoading={createMutation.isPending || updateMutation.isPending}
        />
      )}

      {/* Delete Confirmation Modal */}
      {deleteConfirm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-md w-full mx-4">
            <div className="p-6">
              <div className="flex items-center gap-4 mb-4">
                <div className="w-12 h-12 rounded-full bg-red-100 flex items-center justify-center">
                  <AlertTriangle className="w-6 h-6 text-red-600" />
                </div>
                <div>
                  <h3 className="text-lg font-bold text-slate-900">Eliminar Cuenta</h3>
                  <p className="text-sm text-slate-500">Esta acción no se puede deshacer</p>
                </div>
              </div>

              <div className="bg-slate-50 rounded-lg p-4 mb-4">
                <p className="text-sm text-slate-600">
                  ¿Estás seguro de eliminar la cuenta{' '}
                  <span className="font-mono font-bold text-slate-900">
                    {deleteConfirm.account_number}
                  </span>
                  ?
                </p>
                {Number(deleteConfirm.balance) > 0 && (
                  <p className="text-sm text-amber-600 mt-2">
                    <strong>Advertencia:</strong> Esta cuenta tiene un saldo de $
                    {Number(deleteConfirm.balance).toFixed(2)}
                  </p>
                )}
              </div>

              <div className="flex justify-end gap-3">
                <button
                  onClick={() => setDeleteConfirm(null)}
                  className="px-4 py-2 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50"
                >
                  Cancelar
                </button>
                <button
                  onClick={() => deleteMutation.mutate(deleteConfirm.id)}
                  disabled={deleteMutation.isPending}
                  className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
                >
                  {deleteMutation.isPending ? (
                    'Eliminando...'
                  ) : (
                    <>
                      <Trash2 className="w-4 h-4" />
                      Eliminar
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

interface AccountModalProps {
  account: Account | null
  onClose: () => void
  onSubmit: (data: Partial<Account>) => void
  isLoading: boolean
}

function AccountModal({ account, onClose, onSubmit, isLoading }: AccountModalProps) {
  const isEditing = !!account
  const [formData, setFormData] = useState({
    account_number: account?.account_number || '',
    account_type: account?.account_type?.toLowerCase() || 'prepaid',
    initial_balance: 0, // Solo para crear - backend usa initial_balance
    credit_limit: Number(account?.credit_limit) || 0,
    status: account?.status?.toLowerCase() || 'active',
    max_concurrent_calls: account?.max_concurrent_calls || 5,
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (isEditing) {
      // Solo enviar campos que se pueden editar
      onSubmit({
        status: formData.status as AccountStatus,
        credit_limit: formData.credit_limit,
        max_concurrent_calls: formData.max_concurrent_calls,
      })
    } else {
      // Crear con initial_balance
      onSubmit({
        account_number: formData.account_number,
        account_type: formData.account_type,
        initial_balance: formData.initial_balance,
        credit_limit: formData.credit_limit,
        max_concurrent_calls: formData.max_concurrent_calls,
      } as Partial<Account>)
    }
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between p-6 border-b border-slate-200">
          <h2 className="text-xl font-bold text-slate-900">
            {account ? 'Editar Cuenta' : 'Nueva Cuenta'}
          </h2>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Número de Cuenta *
              </label>
              <input
                type="text"
                required
                value={formData.account_number}
                onChange={(e) =>
                  setFormData({ ...formData, account_number: e.target.value })
                }
                disabled={!!account}
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-slate-100"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Tipo de Cuenta *
              </label>
              <select
                value={formData.account_type?.toLowerCase()}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    account_type: e.target.value as AccountType,
                  })
                }
                disabled={!!account}
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-slate-100"
              >
                <option value="prepaid">Prepago</option>
                <option value="postpaid">Postpago</option>
              </select>
            </div>

            {!isEditing ? (
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">
                  Saldo Inicial
                </label>
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  value={formData.initial_balance}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      initial_balance: parseFloat(e.target.value) || 0,
                    })
                  }
                  className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            ) : (
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">
                  Saldo Actual
                </label>
                <div className="w-full px-3 py-2 border border-slate-200 rounded-lg bg-slate-50 text-slate-700 font-mono">
                  ${Number(account?.balance || 0).toFixed(2)}
                </div>
                <p className="text-xs text-slate-500 mt-1">
                  Para modificar el saldo, usa la opción "Recargar" en Gestión de Saldos
                </p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Límite de Crédito
              </label>
              <input
                type="number"
                step="0.01"
                value={formData.credit_limit}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    credit_limit: parseFloat(e.target.value) || 0,
                  })
                }
                disabled={formData.account_type?.toLowerCase() === 'prepaid'}
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-slate-100"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Estado
              </label>
              <select
                value={formData.status?.toLowerCase()}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    status: e.target.value as AccountStatus,
                  })
                }
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="active">Activa</option>
                <option value="suspended">Suspendida</option>
                <option value="closed">Cerrada</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Llamadas Concurrentes Máx.
              </label>
              <input
                type="number"
                min="1"
                max="100"
                value={formData.max_concurrent_calls}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    max_concurrent_calls: parseInt(e.target.value) || 5,
                  })
                }
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
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
              {isLoading ? 'Guardando...' : account ? 'Actualizar' : 'Crear'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
