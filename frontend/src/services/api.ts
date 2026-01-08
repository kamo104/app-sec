/**
 * API Service
 *
 * Handles communication with the backend API server using protobuf binary format.
 * Provides type-safe wrappers around all API endpoints.
 */

import { BinaryReader, BinaryWriter } from '@bufbuild/protobuf/wire';
import {
  ApiResponse,
  ApiData,
  RegistrationRequest,
  LoginRequest,
  LoginResponseData,
  EmailVerificationRequest,
  ValidationErrorData,
  ResponseCode,
  CounterData,
  SetCounterRequest,
  PasswordResetRequest,
  PasswordResetCompleteRequest,
} from '@/generated/api';
import { translate_response_code, translate_validation_error } from '@/wasm/api-translator.js';

const API_BASE_URL = 'http://localhost:4000/api';

/**
 * Custom error class for API errors with structured information.
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code: ResponseCode,
    public readonly validationError?: ValidationErrorData
  ) {
    super(message);
    this.name = 'ApiError';
    Object.setPrototypeOf(this, ApiError.prototype);
  }
}

/**
 * Encodes a protobuf message to Uint8Array.
 */
function encodeMessage<T>(message: T, encodeFn: (msg: T, writer?: BinaryWriter) => BinaryWriter): Uint8Array {
  const writer = encodeFn(message);
  return writer.finish();
}

/**
 * Decodes a protobuf message from Uint8Array.
 */
function decodeMessage<T>(data: Uint8Array, decodeFn: (reader: BinaryReader | Uint8Array, length?: number) => T): T {
  return decodeFn(data);
}

/**
 * Makes an HTTP request to the API with protobuf encoding.
 */
async function makeApiRequest(
  endpoint: string,
  method: 'GET' | 'POST',
  requestBody?: Uint8Array
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
      body: requestBody as BodyInit | undefined,
    });

    const responseData = await response.arrayBuffer();
    const decoded = decodeMessage(new Uint8Array(responseData), ApiResponse.decode);

    if (!response.ok || decoded.code !== ResponseCode.SUCCESS) {
      const errorMessage = translate_response_code(decoded.code, undefined);
      const validationError = decoded.data?.validationError;
      throw new ApiError(errorMessage, response.status, decoded.code, validationError);
    }

    return decoded;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }

    if (error instanceof TypeError && error.message === 'Failed to fetch') {
      throw new ApiError(
        'Cannot connect to the server. Please ensure the backend is running.',
        0,
        ResponseCode.ERROR_INTERNAL
      );
    }

    throw new ApiError(
      (error as Error).message || 'An unexpected error occurred',
      500,
      ResponseCode.ERROR_INTERNAL
    );
  }
}

/**
 * Extracts login response data from ApiResponse.
 */
function extractLoginResponse(response: ApiResponse): LoginResponseData {
  if (!response.data?.loginResponse) {
    throw new ApiError('Invalid response: missing login data', 500, ResponseCode.ERROR_INTERNAL);
  }
  return response.data.loginResponse;
}

/**
 * Extracts counter data from ApiResponse.
 */
function extractCounterData(response: ApiResponse): CounterData {
  if (!response.data?.counterData) {
    throw new ApiError('Invalid response: missing counter data', 500, ResponseCode.ERROR_INTERNAL);
  }
  return response.data.counterData;
}

/**
 * Register a new user account.
 */
export async function registerUser(data: {
  username: string;
  email: string;
  password: string;
}): Promise<ApiResponse> {
  const request: RegistrationRequest = {
    username: data.username,
    email: data.email,
    password: data.password,
  };
  const requestBytes = encodeMessage(request, RegistrationRequest.encode);
  return makeApiRequest('/register', 'POST', requestBytes);
}

/**
 * Login with username and password.
 */
export async function loginUser(data: {
  username: string;
  password: string;
}): Promise<LoginResponseData> {
  const request: LoginRequest = {
    username: data.username,
    password: data.password,
  };
  const requestBytes = encodeMessage(request, LoginRequest.encode);
  const response = await makeApiRequest('/login', 'POST', requestBytes);
  return extractLoginResponse(response);
}

/**
 * Verify email address with verification token.
 */
export async function verifyEmail(token: string): Promise<ApiResponse> {
  const request: EmailVerificationRequest = { token };
  const requestBytes = encodeMessage(request, EmailVerificationRequest.encode);
  return makeApiRequest('/verify-email', 'POST', requestBytes);
}

/**
 * Get the current counter value.
 */
export async function getCounter(): Promise<CounterData> {
  const response = await makeApiRequest('/counter/get', 'GET');
  return extractCounterData(response);
}

/**
 * Set the counter to a specific value.
 */
export async function setCounter(value: number): Promise<CounterData> {
  const request: SetCounterRequest = { value };
  const requestBytes = encodeMessage(request, SetCounterRequest.encode);
  const response = await makeApiRequest('/counter/set', 'POST', requestBytes);
  return extractCounterData(response);
}

/**
 * Logout the current user session.
 */
export async function logoutUser(): Promise<ApiResponse> {
  return makeApiRequest('/logout', 'POST', new Uint8Array());
}

/**
 * Request a password reset email.
 */
export async function requestPasswordReset(email: string): Promise<ApiResponse> {
  const request: PasswordResetRequest = { email };
  const requestBytes = encodeMessage(request, PasswordResetRequest.encode);
  return makeApiRequest('/request-password-reset', 'POST', requestBytes);
}

/**
 * Complete password reset with token and new password.
 */
export async function completePasswordReset(data: {
  token: string;
  newPassword: string;
}): Promise<ApiResponse> {
  const request: PasswordResetCompleteRequest = {
    token: data.token,
    newPassword: data.newPassword,
  };
  const requestBytes = encodeMessage(request, PasswordResetCompleteRequest.encode);
  return makeApiRequest('/complete-password-reset', 'POST', requestBytes);
}

/**
 * Health check endpoint to verify API availability.
 */
export async function healthCheck(): Promise<boolean> {
  try {
    const response = await fetch(`${API_BASE_URL}/health`, {
      method: 'GET',
      headers: {
        'Accept': 'application/x-protobuf',
      },
    });

    if (!response.ok) {
      return false;
    }

    const responseData = await response.arrayBuffer();
    const decoded = decodeMessage(new Uint8Array(responseData), ApiResponse.decode);
    return decoded.code === ResponseCode.SUCCESS;
  } catch {
    return false;
  }
}

export type {
  ApiResponse,
  ApiData,
  RegistrationRequest,
  LoginRequest,
  LoginResponseData,
  EmailVerificationRequest,
  CounterData,
  SetCounterRequest,
  PasswordResetRequest,
  PasswordResetCompleteRequest,
  ValidationErrorData,
};

export { ResponseCode };
