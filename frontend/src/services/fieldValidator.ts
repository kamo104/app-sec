/**
 * Field validation service using WebAssembly
 * Provides consistent validation across frontend and backend for all fields
 */

import init, {
  validate_field_wasm,
  validate_username_wasm,
  validate_email_wasm,
  validate_password_wasm,
  is_password_valid,
  get_password_errors,
  get_password_strength,
  is_username_valid,
  is_email_valid,
} from '@/wasm/field-validator.js'

let wasmInitialized = false

/**
 * Initialize the WebAssembly module
 * This should be called once when the application starts
 */
export async function initializeFieldValidator(): Promise<void> {
  if (!wasmInitialized) {
    await init()
    wasmInitialized = true
    console.log('WebAssembly field validator initialized')
  }
}

/**
 * Validates a field using WebAssembly
 * @param fieldType - The type of field ('username', 'email', 'password', or 'generic')
 * @param value - The value to validate
 * @param minLength - Minimum length (for username, default: 3)
 * @param maxLength - Maximum length (for username, default: 20)
 * @param validateLength - Whether to validate length (for username, default: true)
 * @returns Promise with validation result
 */
export async function validateField(
  fieldType: 'username' | 'email' | 'password' | 'generic',
  value: string,
  minLength: number = 3,
  maxLength: number = 20,
  validateLength: boolean = true
): Promise<{
  isValid: boolean
  errors: string[]
}> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  const resultJson = validate_field_wasm(fieldType, value, minLength, maxLength, validateLength)
  const result = JSON.parse(resultJson) as {
    is_valid: boolean
    errors: string[]
  }

  return {
    isValid: result.is_valid,
    errors: result.errors,
  }
}

/**
 * Validates a username using WebAssembly
 * @param username - The username to validate
 * @param minLength - Minimum length (default: 3)
 * @param maxLength - Maximum length (default: 20)
 * @param validateLength - Whether to validate length (default: true)
 * @returns Promise with validation result
 */
export async function validateUsername(
  username: string,
  minLength: number = 3,
  maxLength: number = 20,
  validateLength: boolean = true
): Promise<{
  isValid: boolean
  errors: string[]
}> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  const resultJson = validate_username_wasm(username, minLength, maxLength, validateLength)
  const result = JSON.parse(resultJson) as {
    is_valid: boolean
    errors: string[]
  }

  return {
    isValid: result.is_valid,
    errors: result.errors,
  }
}

/**
 * Validates an email using WebAssembly
 * @param email - The email to validate
 * @returns Promise with validation result
 */
export async function validateEmail(email: string): Promise<{
  isValid: boolean
  errors: string[]
}> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  const resultJson = validate_email_wasm(email)
  const result = JSON.parse(resultJson) as {
    is_valid: boolean
    errors: string[]
  }

  return {
    isValid: result.is_valid,
    errors: result.errors,
  }
}

/**
 * Validates a password using WebAssembly
 * @param password - The password to validate
 * @returns Promise with validation result
 */
export async function validatePassword(password: string): Promise<{
  isValid: boolean
  errors: string[]
}> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  const resultJson = validate_password_wasm(password)
  const result = JSON.parse(resultJson) as {
    is_valid: boolean
    errors: string[]
  }

  return {
    isValid: result.is_valid,
    errors: result.errors,
  }
}

/**
 * Quick check if password is valid
 * @param password - The password to check
 * @returns True if password meets all requirements
 */
export async function isPasswordValid(password: string): Promise<boolean> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  return is_password_valid(password)
}

/**
 * Quick check if username is valid
 * @param username - The username to check
 * @param minLength - Minimum length (default: 3)
 * @param maxLength - Maximum length (default: 20)
 * @param validateLength - Whether to validate length (default: true)
 * @returns True if username meets all requirements
 */
export async function isUsernameValid(
  username: string,
  minLength: number = 3,
  maxLength: number = 20,
  validateLength: boolean = true
): Promise<boolean> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  return is_username_valid(username, minLength, maxLength, validateLength)
}

/**
 * Quick check if email is valid
 * @param email - The email to check
 * @returns True if email meets all requirements
 */
export async function isEmailValid(email: string): Promise<boolean> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  return is_email_valid(email)
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
    await initializeFieldValidator()
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
 * Get password errors only
 * @param password - The password to check
 * @returns Array of error messages
 */
export async function getPasswordErrors(password: string): Promise<string[]> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  const errorsJson = get_password_errors(password)
  return JSON.parse(errorsJson) as string[]
}

/**
 * Get password score only
 * @param password - The password to analyze
 * @returns Numerical score (0-7)
 */
export async function getPasswordScore(password: string): Promise<number> {
  if (!wasmInitialized) {
    await initializeFieldValidator()
  }

  const strengthJson = get_password_strength(password)
  const result = JSON.parse(strengthJson) as {
    score: number
  }

  return result.score
}
