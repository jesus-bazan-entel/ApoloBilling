import axios from 'axios'
import type {
  Account,
  Zone,
  RateCard,
  CDR,
  ActiveCall,
  Reservation,
  DashboardStats,
  PaginatedResponse,
  CDRFilters,
} from '../types'

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true, // For JWT cookie authentication
})

// Health check
export const checkHealth = async (): Promise<{ status: string; version: string }> => {
  const { data } = await api.get('/health')
  return data
}

// Dashboard Stats
export const fetchStats = async (): Promise<DashboardStats> => {
  const { data } = await api.get('/stats')
  return data
}

// ============== ACCOUNTS ==============

export const fetchAccounts = async (): Promise<Account[]> => {
  const { data } = await api.get('/accounts')
  // Handle paginated response from Rust backend
  return data.data || data
}

export const fetchAccount = async (id: number): Promise<Account> => {
  const { data } = await api.get(`/accounts/${id}`)
  return data.data || data
}

export const createAccount = async (account: Partial<Account>): Promise<Account> => {
  const { data } = await api.post('/accounts', account)
  return data.data || data
}

export const updateAccount = async (id: number, account: Partial<Account>): Promise<Account> => {
  const { data } = await api.put(`/accounts/${id}`, account)
  return data.data || data
}

export const deleteAccount = async (id: number): Promise<void> => {
  await api.delete(`/accounts/${id}`)
}

export const topupAccount = async (
  id: number,
  amount: number,
  reason?: string
): Promise<{ previous_balance: number; amount: number; new_balance: number }> => {
  const { data } = await api.post(`/accounts/${id}/topup`, { amount, reason })
  return data.data || data
}

// ============== ZONES (Management) ==============

export const fetchZones = async (): Promise<Zone[]> => {
  const { data } = await api.get('/zonas')
  return data.data || data
}

export const createZone = async (zone: Partial<Zone>): Promise<Zone> => {
  const { data } = await api.post('/zonas', zone)
  return data.data || data
}

export const updateZone = async (id: number, zone: Partial<Zone>): Promise<Zone> => {
  const { data } = await api.put(`/zonas/${id}`, zone)
  return data.data || data
}

export const deleteZone = async (id: number): Promise<void> => {
  await api.delete(`/zonas/${id}`)
}

// ============== RATE CARDS ==============

export const fetchRateCards = async (): Promise<RateCard[]> => {
  const { data } = await api.get('/rate-cards')
  return data.data || data
}

export const fetchRateCard = async (id: number): Promise<RateCard> => {
  const { data } = await api.get(`/rate-cards/${id}`)
  return data.data || data
}

export const createRateCard = async (rate: Partial<RateCard>): Promise<RateCard> => {
  const { data } = await api.post('/rate-cards', rate)
  return data.data || data
}

export const updateRateCard = async (id: number, rate: Partial<RateCard>): Promise<RateCard> => {
  const { data } = await api.put(`/rate-cards/${id}`, rate)
  return data.data || data
}

export const deleteRateCard = async (id: number): Promise<void> => {
  await api.delete(`/rate-cards/${id}`)
}

// Rate lookup (LPM - Longest Prefix Match)
export const lookupRate = async (destination: string): Promise<RateCard | null> => {
  try {
    const { data } = await api.get(`/rate-cards/search/${destination}`)
    return data.data || data
  } catch {
    return null
  }
}

// ============== CDRs (Call Detail Records) ==============

export const fetchCDRs = async (
  filters?: CDRFilters,
  page = 1,
  perPage = 50
): Promise<PaginatedResponse<CDR>> => {
  const params = new URLSearchParams()
  params.append('page', page.toString())
  params.append('per_page', perPage.toString())

  if (filters) {
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined && value !== '') {
        params.append(key, value.toString())
      }
    })
  }

  const { data } = await api.get(`/cdrs?${params.toString()}`)
  return data
}

export const fetchCDR = async (id: number): Promise<CDR> => {
  const { data } = await api.get(`/cdrs/${id}`)
  return data.data || data
}

export const exportCDRs = async (filters?: CDRFilters, format: 'csv' | 'json' = 'csv'): Promise<Blob> => {
  const params = new URLSearchParams()
  params.append('format', format)

  if (filters) {
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined && value !== '') {
        params.append(key, value.toString())
      }
    })
  }

  const { data } = await api.get(`/cdrs/export?${params.toString()}`, {
    responseType: 'blob',
  })
  return data
}

export const fetchCDRStats = async (filters?: CDRFilters): Promise<{
  total_cdrs: number
  total_minutes: number
  total_cost: number
  avg_duration: number
}> => {
  const params = new URLSearchParams()
  if (filters) {
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined && value !== '') {
        params.append(key, value.toString())
      }
    })
  }

  const { data } = await api.get(`/cdrs/stats?${params.toString()}`)
  return data
}

// ============== ACTIVE CALLS ==============

export const fetchActiveCalls = async (): Promise<ActiveCall[]> => {
  const { data } = await api.get('/active-calls')
  return data.data || data
}

export const createActiveCall = async (call: Partial<ActiveCall>): Promise<ActiveCall> => {
  const { data } = await api.post('/active-calls', call)
  return data.data || data
}

export const deleteActiveCall = async (callId: string): Promise<void> => {
  await api.delete(`/active-calls/${callId}`)
}

// ============== RESERVATIONS ==============

export const fetchReservations = async (status?: string): Promise<Reservation[]> => {
  const params = status ? `?status=${status}` : ''
  const { data } = await api.get(`/reservations${params}`)
  return data
}

export const fetchActiveReservations = async (): Promise<Reservation[]> => {
  const { data } = await api.get('/reservations/active')
  return data
}

// ============== AUTHENTICATION ==============

export interface LoginCredentials {
  username: string
  password: string
}

export interface AuthUser {
  id: number
  username: string
  role: string
}

export const login = async (credentials: LoginCredentials): Promise<AuthUser> => {
  const { data } = await api.post('/auth/login', credentials)
  // API returns { data: { access_token, user: {...}, ... } }
  // Return just the user object to match getCurrentUser format
  return data.data?.user || data.user || data.data || data
}

export const logout = async (): Promise<void> => {
  await api.post('/auth/logout')
}

export const getCurrentUser = async (): Promise<AuthUser | null> => {
  try {
    const { data } = await api.get('/auth/me')
    // API returns { data: { user: {...}, token_expires_at: ... } }
    return data.data?.user || data.user || data.data || data
  } catch {
    return null
  }
}

// ============== MANAGEMENT (Prefixes, Tariffs) ==============

export interface Prefix {
  id: number
  zone_id: number
  prefix: string
  description?: string
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface Tariff {
  id: number
  zone_id: number
  rate_name?: string
  rate_per_minute: number
  rate_per_call: number
  billing_increment: number
  minimum_seconds: number
  priority: number
  effective_from: string
  effective_until?: string
  enabled: boolean
  created_at: string
  updated_at: string
}

export const fetchPrefixes = async (zoneId?: number): Promise<Prefix[]> => {
  const params = zoneId ? `?zone_id=${zoneId}` : ''
  const { data } = await api.get(`/prefijos${params}`)
  return data.data || data
}

export const createPrefix = async (prefix: Partial<Prefix>): Promise<Prefix> => {
  const { data } = await api.post('/prefijos', prefix)
  return data.data || data
}

export const deletePrefix = async (id: number): Promise<void> => {
  await api.delete(`/prefijos/${id}`)
}

export const fetchTariffs = async (zoneId?: number): Promise<Tariff[]> => {
  const params = zoneId ? `?zone_id=${zoneId}` : ''
  const { data } = await api.get(`/tarifas${params}`)
  return data.data || data
}

export const createTariff = async (tariff: Partial<Tariff>): Promise<Tariff> => {
  const { data } = await api.post('/tarifas', tariff)
  return data.data || data
}

export const updateTariff = async (id: number, tariff: Partial<Tariff>): Promise<Tariff> => {
  const { data } = await api.put(`/tarifas/${id}`, tariff)
  return data.data || data
}

export const deleteTariff = async (id: number): Promise<void> => {
  await api.delete(`/tarifas/${id}`)
}

// Sync rate cards from zones/prefixes/tariffs
export const syncRateCards = async (): Promise<{ synced_count: number }> => {
  const { data } = await api.post('/sync-rate-cards')
  return data.data || data
}

export default api
