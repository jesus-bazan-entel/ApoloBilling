// Account types based on PRD
export type AccountType = 'prepaid' | 'postpaid' | 'PREPAID' | 'POSTPAID'
export type AccountStatus = 'active' | 'suspended' | 'closed' | 'ACTIVE' | 'SUSPENDED' | 'CLOSED'

export interface Account {
  id: number
  account_number: string
  account_name?: string
  customer_phone?: string
  account_type: AccountType
  balance: number
  credit_limit: number
  currency?: string
  status: AccountStatus
  max_concurrent_calls: number
  available_balance?: number
  created_at: string
  updated_at: string
}

// Zone types
export interface Zone {
  id: number
  zone_name: string
  zone_type?: string
  network_type?: string
  description?: string
  enabled?: boolean
  created_at?: string
  updated_at?: string
}

export interface RateCard {
  id: number
  rate_name?: string
  destination_prefix: string
  destination_name: string
  rate_per_minute: number
  billing_increment: number
  connection_fee: number
  effective_start: string
  effective_end: string | null
  priority: number
  is_effective?: boolean
  rate_per_second?: number
  created_at?: string
  updated_at?: string
}

// CDR (Call Detail Record) - based on PRD schema
export interface CDR {
  id: number
  call_uuid: string
  caller_number: string
  callee_number: string
  start_time: string
  answer_time?: string
  end_time?: string
  duration: number
  billsec: number
  total_cost: number
  cost?: number // Alias for compatibility
  hangup_cause?: string
  direction: 'inbound' | 'outbound' | 'internal'
  destination_prefix?: string
  rate_per_minute?: number
  zone_id?: number
  zone_name?: string
  account_id?: number
  answered?: boolean
}

// Active Call for real-time monitoring
export interface ActiveCall {
  id?: string | number
  call_uuid: string
  uuid?: string // Alias
  caller_number: string
  callee_number: string
  direction: string
  start_time: string
  answer_time?: string
  duration_seconds?: number
  duration?: number
  status: 'dialing' | 'answered' | 'ringing' | 'active'
  account_id?: number
  destination_prefix?: string
  rate_per_minute?: number
  reserved_amount?: number
  zone_name?: string
  estimated_cost?: number
  remaining_duration?: number
}

// Balance Reservation
export interface Reservation {
  id: string | number
  account_id: number
  call_uuid: string
  reserved_amount: number
  consumed_amount: number
  released_amount: number
  status: string
  reservation_type: string
  destination_prefix?: string
  rate_per_minute: number
  reserved_minutes?: number
  created_at: string
  updated_at?: string
  expires_at: string
}

// Balance Transaction for audit
export interface BalanceTransaction {
  id: number
  account_id: number
  transaction_type: 'recharge' | 'deduction' | 'adjustment' | 'refund'
  amount: number
  balance_before: number
  balance_after: number
  description: string
  created_at: string
  created_by: string
}

// Dashboard Statistics - matches Rust backend DashboardStats
export interface DashboardStats {
  total_accounts: number
  active_accounts: number
  total_balance: number
  active_calls: number
  active_reservations: number
  cdrs_today: number
  revenue_today: number
  minutes_today: number
  // Legacy fields for compatibility
  reserved_amount?: number
  calls_today?: number
  calls_this_month?: number
  revenue_this_month?: number
}

// Authorization Response from Rust engine
export interface AuthResponse {
  authorized: boolean
  reason: string
  uuid: string
  account_id?: number
  account_number?: string
  reservation_id?: string
  reserved_amount?: number
  max_duration_seconds?: number
  rate_per_minute?: number
}

// WebSocket Message Types
export interface WSMessage {
  type: 'call_start' | 'call_update' | 'call_end' | 'stats_update'
  data: ActiveCall | DashboardStats
}

// API Response wrapper - matches Rust backend ApiResponse
export interface ApiResponse<T> {
  success: boolean
  data?: T
  message?: string
  error?: string
}

// Pagination - matches Rust backend PaginatedResponse
export interface PaginatedResponse<T> {
  data: T[]
  items?: T[] // Alias for compatibility
  total: number
  page: number
  per_page: number
  total_pages: number
}

// Filter params for CDR
export interface CDRFilters {
  start_date?: string
  end_date?: string
  caller_number?: string
  callee_number?: string
  direction?: string
  destination_prefix?: string
  account_id?: number
  zone_id?: number
  min_cost?: number
  max_cost?: number
  min_duration?: number
  max_duration?: number
  hangup_cause?: string
}
