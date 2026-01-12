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
 * Get validation constants
 */
export const VALIDATION_CONSTANTS = {
  POST_TITLE_MAX_LENGTH: get_post_title_max_length(),
  POST_DESCRIPTION_MAX_LENGTH: get_post_description_max_length(),
  IMAGE_ALLOWED_MIME_TYPES: get_image_allowed_mime_types().split(','),
  POSTS_PER_PAGE: get_posts_per_page(),
} as const
