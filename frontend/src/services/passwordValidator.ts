/**
 * Password validation service using WebAssembly
 * Provides consistent password strength validation across frontend and backend
 */

import init, {
  is_password_valid,
  get_password_errors,
  get_password_strength,
} from '@/wasm/password-validator.js'

let wasmInitialized = false

/**
 * Initialize the WebAssembly module
 * This should be called once when the application starts
 */
export async function initializePasswordValidator(): Promise<void> {
  if (!wasmInitialized) {
    await init()
    wasmInitialized = true
    console.log('WebAssembly password validator initialized')
  }
}

/**
 * Validates password strength using WebAssembly
 * @param password - The password to validate
 * @returns Promise with validation result
 */
export async function validatePassword(password: string): Promise<{
  isValid: boolean
  errors: string[]
}> {
  if (!wasmInitialized) {
    await initializePasswordValidator()
  }

  const errorsJson = get_password_errors(password)
  const errors = JSON.parse(errorsJson) as string[]

  return {
    isValid: errors.length === 0,
    errors,
  }
}

/**
 * Quick check if password is valid
 * @param password - The password to check
 * @returns True if password meets all requirements
 */
export async function isPasswordValid(password: string): Promise<boolean> {
  if (!wasmInitialized) {
    await initializePasswordValidator()
  }

  return is_password_valid(password)
}

/**
 * Get detailed password strength information
 * @param password - The password to analyze
 * @returns Detailed strength analysis including score and strength level
 */
export async function getPasswordStrengthInfo(password: string): Promise<{
  isValid: boolean
  errors: string[]
  strength: 'Weak' | 'Medium' | 'Strong'
  score: number
}> {
  if (!wasmInitialized) {
    await initializePasswordValidator()
  }

  const strengthJson = get_password_strength(password)
  const result = JSON.parse(strengthJson) as {
    is_valid: boolean
    errors: string[]
    strength: 'Weak' | 'Medium' | 'Strong'
    score: number
  }

  return {
    isValid: result.is_valid,
    errors: result.errors,
    strength: result.strength,
    score: result.score,
  }
}

/**
 * Get password score only
 * @param password - The password to analyze
 * @returns Numerical score (0-7)
 */
export async function getPasswordScore(password: string): Promise<number> {
  if (!wasmInitialized) {
    await initializePasswordValidator()
  }

  const strengthJson = get_password_strength(password)
  const result = JSON.parse(strengthJson) as {
    score: number
  }

  return result.score
}