/**
 * API Client Utilities
 *
 * Shared utilities for making API requests with protobuf encoding.
 */

import {
  ApiResponse,
  ResponseCode,
  ErrorCode,
  ValidationErrorData,
} from '@/generated/api';
import { translate_response } from '@/wasm/translator.js';

export const API_BASE_URL = 'http://localhost:4000/api';

/**
 * Custom error class for API errors with structured information.
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly errorCode?: ErrorCode,
    public readonly validationError?: ValidationErrorData
  ) {
    super(message);
    this.name = 'ApiError';
    Object.setPrototypeOf(this, ApiError.prototype);
  }
}

/**
 * Makes an HTTP request to the API with protobuf encoding.
 */
export async function makeApiRequest(
  endpoint: string,
  method: 'GET' | 'POST',
  requestBody?: Uint8Array
): Promise<{ response: ApiResponse; bytes: Uint8Array }> {
  const url = `${API_BASE_URL}${endpoint}`;

  try {
    const response = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/x-protobuf',
        'Accept': 'application/x-protobuf',
      },
      credentials: 'include',
      body: requestBody as BodyInit | undefined,
    });

    const responseData = await response.arrayBuffer();
    const responseBytes = new Uint8Array(responseData);
    const decoded = ApiResponse.decode(responseBytes);

    // Check response code and handle errors
    if (decoded.code === ResponseCode.ERROR) {
      const errorCode = decoded.error ?? ErrorCode.ERROR_CODE_UNSPECIFIED;
      const errorMessage = translate_response(responseBytes, undefined);
      const validationError = decoded.data?.validationError;
      throw new ApiError(errorMessage, response.status, errorCode, validationError);
    }

    if (decoded.code !== ResponseCode.SUCCESS) {
      throw new ApiError('An unexpected response was received', response.status);
    }

    return { response: decoded, bytes: responseBytes };
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }

    if (error instanceof TypeError && error.message === 'Failed to fetch') {
      throw new ApiError(
        'Cannot connect to the server. Please ensure the backend is running.',
        0,
        ErrorCode.INTERNAL
      );
    }

    throw new ApiError(
      (error as Error).message || 'An unexpected error occurred',
      500,
      ErrorCode.INTERNAL
    );
  }
}
