/**
 * Centralized constants for the frontend application
 * These values should be the single source of truth
 */

import { get_posts_per_page } from '@/wasm/field-validator.js'

// Time conversion functions (in milliseconds)
// Note: 1000ms = 1s, 60s = 1m, 60m = 1h, 24h = 1d
export const TIME = {
  seconds: (value: number) => value * 1000,
  minutes: (value: number) => value * 60 * 1000,
  hours: (value: number) => value * 60 * 60 * 1000,
  days: (value: number) => value * 24 * 60 * 60 * 1000,
} as const

// Pagination constants
export const PAGINATION = {
  POSTS_PER_PAGE: get_posts_per_page(),
} as const

// API constants
export const API = {
  MAX_RETRIES: 3,
  TIMEOUT: 30000, // 30 seconds
  BASE_URL: import.meta.env.VITE_API_BASE_URL || '/api',
} as const

// Auth constants
export const AUTH = {
  SESSION_REFRESH_THRESHOLD: 5 * 60 * 1000, // Refresh if less than 5 minutes remaining
  TOKEN_EXPIRY_BUFFER: 60 * 1000, // 1 minute buffer
} as const

// UI constants
export const UI = {
  TOAST_DURATION: 5000, // 5 seconds
  DEBOUNCE_DELAY: 300, // milliseconds
  MAX_IMAGE_PREVIEW_SIZE: 5 * 1024 * 1024, // 5MB
} as const

// Re-export validation constants from validation.ts for convenience
export { VALIDATION_CONSTANTS } from './validation.js'