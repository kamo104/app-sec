/**
 * API Service
 *
 * Handles communication with the backend API server using protobuf binary format.
 */

import { BinaryReader, BinaryWriter } from '@bufbuild/protobuf/wire';
import {
  ApiResponse,
  RegistrationRequest,
  LoginRequest,
  LoginResponseData,
  HealthData,
  EmptyData,
  EmailVerificationRequest,
  ValidationErrorData,
  ResponseCode,
  CounterData,
  SetCounterRequest,
  PasswordResetRequest,
  PasswordResetCompleteRequest,
} from '@/generated/api';
import { initializeWasm } from '@/services/wasmLoader';
import { translate_response_code } from '@/wasm/api-translator.js';

// Base URL for API requests
// In production, this would be configured via environment variables
const API_BASE_URL = 'http://localhost:4000/api';

/**
 * API Error for structured error handling
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public code?: ResponseCode,
    public validationError?: ValidationErrorData
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * Encode a protobuf message to Uint8Array
 */
function encodeProtobuf<T>(message: T, encodeFn: (msg: T, writer?: BinaryWriter) => BinaryWriter): Uint8Array {
  const writer = encodeFn(message);
  return writer.finish();
}

/**
 * Decode a protobuf response from Uint8Array
 */
function decodeProtobuf<T>(data: Uint8Array, decodeFn: (reader: BinaryReader | Uint8Array, length?: number) => T): T {
  return decodeFn(data);
}

/**
 * Generic fetch wrapper for protobuf API calls
 */
async function apiFetchProtobuf<T>(
  endpoint: string,
  requestBytes: Uint8Array | null,
  method: 'GET' | 'POST' = 'POST'
): Promise<ApiResponse> {
  const url = `${API_BASE_URL}${endpoint}`;

  try {
    const response = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/x-protobuf',
        'Accept': 'application/x-protobuf',
      },
      credentials: 'include',
      body: requestBytes as unknown as BodyInit,
    });

    // Handle non-2xx responses
    if (!response.ok) {
      let errorMessage = `HTTP ${response.status}`;
      let responseCode: ResponseCode | undefined;

      // Try to parse error response as protobuf
      try {
        const errorData = await response.arrayBuffer();
        const decoded = decodeProtobuf(new Uint8Array(errorData), ApiResponse.decode);
        responseCode = decoded.code;
        errorMessage = translate_response_code(decoded.code, undefined);

        throw new ApiError(errorMessage, response.status, responseCode, decoded.validationError);
      } catch (e) {
        if (e instanceof ApiError) throw e;
        // Fallback if response body isn't protobuf
        errorMessage = await response.text();
      }

      throw new ApiError(errorMessage, response.status, responseCode);
    }

    // Parse successful protobuf response
    const responseData = await response.arrayBuffer();
    const decoded = decodeProtobuf(new Uint8Array(responseData), ApiResponse.decode);
    return decoded;
  } catch (error) {
    // Re-throw ApiError instances, wrap others
    if (error instanceof ApiError) {
      throw error;
    }

    // Handle network errors
    if (error instanceof TypeError && error.message === 'Failed to fetch') {
      throw new ApiError(
        'Cannot connect to the server. Please ensure the backend is running.',
        0,
        ResponseCode.RESPONSE_CODE_UNSPECIFIED
      );
    }

    throw new ApiError(
      (error as Error).message || 'An unexpected error occurred',
      500,
      ResponseCode.RESPONSE_CODE_UNSPECIFIED
    );
  }
}

/**
 * Register a new user
 */
export async function registerUser(
  data: { username: string; email: string; password: string }
): Promise<ApiResponse> {
  const request: RegistrationRequest = {
    username: data.username,
    email: data.email,
    password: data.password,
  };
  const requestBytes = encodeProtobuf(request, RegistrationRequest.encode);
  return apiFetchProtobuf('/register', requestBytes);
}

/**
 * Health check for the API
 */
export async function healthCheck(): Promise<ApiResponse> {
  try {
    const response = await fetch(`${API_BASE_URL}/health`, {
      method: 'GET',
      headers: {
        'Accept': 'application/x-protobuf',
      },
    });

    if (!response.ok) {
      return {
        success: false,
        code: ResponseCode.ERROR_INTERNAL,
        loginResponse: undefined,
        healthData: undefined,
        empty: undefined,
      };
    }

    const responseData = await response.arrayBuffer();
    const decoded = decodeProtobuf(new Uint8Array(responseData), ApiResponse.decode);
    return decoded;
  } catch (error) {
    // Return a failed response for health check
    return {
      success: false,
      code: ResponseCode.ERROR_INTERNAL,
      loginResponse: undefined,
      healthData: undefined,
      empty: undefined,
    };
  }
}

/**
 * Login a user
 */
export async function loginUser(
  data: { username: string; password: string }
): Promise<ApiResponse> {
  const request: LoginRequest = {
    username: data.username,
    password: data.password,
  };
  const requestBytes = encodeProtobuf(request, LoginRequest.encode);
  return apiFetchProtobuf('/login', requestBytes);
}

/**
 * Verify email with token
 */
export async function verifyEmail(token: string): Promise<ApiResponse> {
  const request: EmailVerificationRequest = {
    token: token,
  };
  const requestBytes = encodeProtobuf(request, EmailVerificationRequest.encode);
  return apiFetchProtobuf('/verify-email', requestBytes);
}

/**
 * Get current counter value
 */
export async function getCounter(): Promise<ApiResponse> {
  return apiFetchProtobuf('/counter/get', null, 'GET');
}

/**
 * Set counter value
 */
export async function setCounter(value: number): Promise<ApiResponse> {
  const request: SetCounterRequest = {
    value,
  };
  const requestBytes = encodeProtobuf(request, SetCounterRequest.encode);
  return apiFetchProtobuf('/counter/set', requestBytes);
}

/**
 * Logout user and invalidate session
 */
export async function logoutUser(): Promise<ApiResponse> {
  return apiFetchProtobuf('/logout', new Uint8Array());
}

/**
 * Request password reset
 */
export async function requestPasswordReset(email: string): Promise<ApiResponse> {
  const request: PasswordResetRequest = { email };
  const requestBytes = encodeProtobuf(request, PasswordResetRequest.encode);
  return apiFetchProtobuf('/request-password-reset', requestBytes);
}

/**
 * Complete password reset
 */
export async function completePasswordReset(data: { token: string; newPassword: string }): Promise<ApiResponse> {
  const request: PasswordResetCompleteRequest = {
    token: data.token,
    newPassword: data.newPassword,
  };
  const requestBytes = encodeProtobuf(request, PasswordResetCompleteRequest.encode);
  return apiFetchProtobuf('/complete-password-reset', requestBytes);
}

// Re-export types for convenience
export type { ApiResponse, RegistrationRequest, LoginRequest, LoginResponseData, HealthData, EmptyData, EmailVerificationRequest, CounterData, SetCounterRequest, PasswordResetRequest, PasswordResetCompleteRequest };
export { ResponseCode };
