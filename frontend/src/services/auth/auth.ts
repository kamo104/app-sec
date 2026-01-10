/**
 * Authentication API Endpoints
 */

import {
  ApiResponse,
  LoginRequest,
  LoginResponseData,
  RegistrationRequest,
  EmailVerificationRequest,
  PasswordResetRequest,
  PasswordResetCompleteRequest,
  ErrorCode,
} from '@/generated/api';
import { makeApiRequest, ApiError } from '../api-client';

/**
 * Extracts login response data from ApiResponse.
 */
function extractLoginResponse(response: ApiResponse): LoginResponseData {
  if (!response.data?.loginResponse) {
    throw new ApiError('Invalid response: missing login data', 500, ErrorCode.INTERNAL);
  }
  return response.data.loginResponse;
}

/**
 * Register a new user account.
 */
export async function registerUser(data: {
  username: string;
  email: string;
  password: string;
}): Promise<{ response: ApiResponse; bytes: Uint8Array }> {
  const request: RegistrationRequest = {
    username: data.username,
    email: data.email,
    password: data.password,
  };
  const requestBytes = RegistrationRequest.encode(request).finish();
  return makeApiRequest('/register', 'POST', requestBytes);
}

/**
 * Login with username and password.
 */
export async function loginUser(data: {
  username: string;
  password: string;
}): Promise<{ loginData: LoginResponseData; bytes: Uint8Array }> {
  const request: LoginRequest = {
    username: data.username,
    password: data.password,
  };
  const requestBytes = LoginRequest.encode(request).finish();
  const result = await makeApiRequest('/login', 'POST', requestBytes);
  return { loginData: extractLoginResponse(result.response), bytes: result.bytes };
}

/**
 * Logout the current user session.
 */
export async function logoutUser(): Promise<{ response: ApiResponse; bytes: Uint8Array }> {
  return makeApiRequest('/logout', 'POST', new Uint8Array());
}

/**
 * Check if the current session is authenticated.
 */
export async function checkAuth(): Promise<{ loginData: LoginResponseData; bytes: Uint8Array }> {
  const result = await makeApiRequest('/auth/check', 'GET');
  return { loginData: extractLoginResponse(result.response), bytes: result.bytes };
}

/**
 * Refresh the current session, extending its expiry time.
 */
export async function refreshSession(): Promise<{ loginData: LoginResponseData; bytes: Uint8Array }> {
  const result = await makeApiRequest('/auth/refresh', 'POST', new Uint8Array());
  return { loginData: extractLoginResponse(result.response), bytes: result.bytes };
}

/**
 * Verify email address with verification token.
 */
export async function verifyEmail(token: string): Promise<{ response: ApiResponse; bytes: Uint8Array }> {
  const request: EmailVerificationRequest = { token };
  const requestBytes = EmailVerificationRequest.encode(request).finish();
  return makeApiRequest('/verify-email', 'POST', requestBytes);
}

/**
 * Request a password reset email.
 */
export async function requestPasswordReset(email: string): Promise<{ response: ApiResponse; bytes: Uint8Array }> {
  const request: PasswordResetRequest = { email };
  const requestBytes = PasswordResetRequest.encode(request).finish();
  return makeApiRequest('/request-password-reset', 'POST', requestBytes);
}

/**
 * Complete password reset with token and new password.
 */
export async function completePasswordReset(data: {
  token: string;
  newPassword: string;
}): Promise<{ response: ApiResponse; bytes: Uint8Array }> {
  const request: PasswordResetCompleteRequest = {
    token: data.token,
    newPassword: data.newPassword,
  };
  const requestBytes = PasswordResetCompleteRequest.encode(request).finish();
  return makeApiRequest('/complete-password-reset', 'POST', requestBytes);
}
