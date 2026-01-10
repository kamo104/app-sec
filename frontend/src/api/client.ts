/**
 * API Client Configuration
 *
 * Configures the generated OpenAPI client with base URL and credentials.
 */

import { client } from '@/generated/api-client/client.gen';
import type { ErrorResponse, ValidationErrorData } from '@/generated/api-client';

// Configure the client
client.setConfig({
  baseUrl: 'http://localhost:4000',
  credentials: 'include',
});

/**
 * Custom error class for API errors with structured information.
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly errorCode?: string,
    public readonly validationError?: ValidationErrorData
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * Extract error from API response and throw ApiError.
 */
export function handleApiError(error: ErrorResponse | undefined, status: number): never {
  if (error) {
    throw new ApiError(
      error.error,
      status,
      error.error,
      error.validation ?? undefined
    );
  }
  throw new ApiError('Unknown error', status);
}

// Re-export everything from the generated client for convenience
export * from '@/generated/api-client';
