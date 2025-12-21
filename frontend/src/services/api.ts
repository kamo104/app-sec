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
} from '@/generated/api';

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
    public code?: string
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
  requestBytes: Uint8Array
): Promise<ApiResponse> {
  const url = `${API_BASE_URL}${endpoint}`;

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-protobuf',
        'Accept': 'application/x-protobuf',
      },
      body: requestBytes as unknown as BodyInit,
    });

    // Handle non-2xx responses
    if (!response.ok) {
      let errorMessage = `HTTP ${response.status}`;
      let errorCode: string | undefined;

      // Try to parse error response as protobuf
      try {
        const errorData = await response.arrayBuffer();
        const decoded = decodeProtobuf(new Uint8Array(errorData), ApiResponse.decode);
        errorMessage = decoded.message;
        errorCode = response.status === 409 ? 'CONFLICT' :
                    response.status === 400 ? 'BAD_REQUEST' :
                    response.status === 401 ? 'UNAUTHORIZED' : undefined;
      } catch {
        // Fallback if response body isn't protobuf
        errorMessage = await response.text();
      }

      throw new ApiError(errorMessage, response.status, errorCode);
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
        'NETWORK_ERROR'
      );
    }

    throw new ApiError(
      (error as Error).message || 'An unexpected error occurred',
      500,
      'UNKNOWN_ERROR'
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
    const requestBytes = new Uint8Array(0); // Empty request for GET-like behavior
    const response = await fetch(`${API_BASE_URL}/health`, {
      method: 'GET',
      headers: {
        'Accept': 'application/x-protobuf',
      },
    });

    if (!response.ok) {
      return {
        success: false,
        message: `HTTP ${response.status}`,
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
      message: error instanceof ApiError ? error.message : 'Health check failed',
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

// Re-export types for convenience
export type { ApiResponse, RegistrationRequest, LoginRequest, LoginResponseData, HealthData, EmptyData, EmailVerificationRequest };
