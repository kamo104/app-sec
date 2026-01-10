/**
 * Types returned by WASM modules (field-validator).
 *
 * These types match the JSON output from the field-validator WASM module.
 */

import type { FieldType, ValidationErrorCode } from '@/generated/api-client';

/**
 * Password strength levels.
 */
export type PasswordStrength =
  | 'PASSWORD_STRENGTH_UNSPECIFIED'
  | 'PASSWORD_STRENGTH_WEAK'
  | 'PASSWORD_STRENGTH_MEDIUM'
  | 'PASSWORD_STRENGTH_STRONG'
  | 'PASSWORD_STRENGTH_CIA';

/**
 * Validation error for a single field (returned by WASM validator).
 */
export interface WasmValidationFieldError {
  field: FieldType;
  errors: ValidationErrorCode[];
}

/**
 * Detailed password validation data including strength assessment.
 */
export interface ValidationDetailedPasswordData {
  data?: WasmValidationFieldError;
  strength: PasswordStrength;
  score: number;
}
