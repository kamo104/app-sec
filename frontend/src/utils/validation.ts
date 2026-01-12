/**
 * Shared validation utilities for frontend
 * Uses field-validator WASM module for consistent validation across app
 */

import {
  validate_field,
  validate_password_detailed,
  validate_image_size,
  validate_image_mime,
  get_post_title_max_length,
  get_post_description_max_length,
  get_image_allowed_mime_types,
  get_posts_per_page,
  type ValidationFieldError,
  type ValidationDetailedPasswordData,
  type FieldType,
  type ValidationErrorCode,
} from '@/wasm/field-validator.js'
import { translate_field_validation_error } from '@/wasm/translator.js'

/**
 * Validates a field and returns translated errors
 */
export function validateField(
  fieldType: FieldType,
  value: string,
  locale?: string
): { valid: boolean; errors: string[] } {
  try {
    const resultJson = validate_field(fieldType, value)
    const result: ValidationFieldError = JSON.parse(resultJson)
    
    const errors = result.errors.map(err => 
      translate_field_validation_error(fieldType, err, locale)
    )
    
    return {
      valid: errors.length === 0,
      errors,
    }
  } catch (error) {
    console.error('Validation error:', error)
    return { valid: false, errors: ['INTERNAL_ERROR'] }
  }
}

/**
 * Validates password with detailed strength information
 */
export function validatePassword(
  password: string,
  locale?: string
): {
  valid: boolean
  errors: string[]
  score: number
  strength: string
} {
  try {
    const resultJson = validate_password_detailed(password)
    const result: ValidationDetailedPasswordData = JSON.parse(resultJson)
    
    const errors = result.data?.errors.map(err => 
      translate_field_validation_error('PASSWORD', err, locale)
    ) || []
    
    return {
      valid: errors.length === 0,
      errors,
      score: result.score,
      strength: result.strength,
    }
  } catch (error) {
    console.error('Password validation error:', error)
    return { valid: false, errors: ['INTERNAL_ERROR'], score: 0, strength: 'PASSWORD_STRENGTH_UNSPECIFIED' }
  }
}

/**
 * Validates image file
 */
export function validateImageFile(
  file: File,
  locale?: string
): { valid: boolean; errors: string[] } {
  const errors: string[] = []
  
  if (!validate_image_size(file.size)) {
    errors.push(translate_field_validation_error('IMAGE', 'TOO_LONG', locale))
  }
  
  // Use MIME type from browser
  if (!validate_image_mime(file.type)) {
    errors.push(translate_field_validation_error('IMAGE', 'INVALID_FORMAT', locale))
  }
  
  return {
    valid: errors.length === 0,
    errors,
  }
}

/**
 * Get validation constants
 */
export const VALIDATION_CONSTANTS = {
  POST_TITLE_MAX_LENGTH: get_post_title_max_length(),
  POST_DESCRIPTION_MAX_LENGTH: get_post_description_max_length(),
  IMAGE_ALLOWED_MIME_TYPES: get_image_allowed_mime_types().split(','),
  POSTS_PER_PAGE: get_posts_per_page(),
} as const

/**
 * Batch validate multiple fields
 */
export function validateBatch(
  fields: Array<{ fieldType: FieldType; value: string }>,
  locale?: string
): { valid: boolean; fieldErrors: Record<string, string[]> } {
  const fieldErrors: Record<string, string[]> = {}
  let allValid = true
  
  for (const { fieldType, value } of fields) {
    const result = validateField(fieldType, value, locale)
    if (!result.valid) {
      fieldErrors[fieldType] = result.errors
      allValid = false
    }
  }
  
  return { valid: allValid, fieldErrors }
}

/**
 * Helper to check if a value is valid for a field type
 */
export function isValidField(fieldType: FieldType, value: string): boolean {
  return validateField(fieldType, value).valid
}

/**
 * Helper to get single error message for a field
 */
export function getFieldErrorMessage(
  fieldType: FieldType,
  value: string,
  locale?: string
): string | null {
  const result = validateField(fieldType, value, locale)
  return result.errors.length > 0 ? result.errors[0] : null
}