/**
 * API Service
 *
 * Handles communication with the backend API server using protobuf binary format.
 * Provides type-safe wrappers around all API endpoints.
 */

// Re-export API client utilities
export { ApiError, API_BASE_URL } from './api-client';

// Re-export auth endpoints
export {
  registerUser,
  loginUser,
  logoutUser,
  checkAuth,
  refreshSession,
  verifyEmail,
  requestPasswordReset,
  completePasswordReset,
} from './auth/auth';

// Re-export counter endpoints
export { getCounter, setCounter } from './counter';

// Re-export health endpoint
export { healthCheck } from './health';

// Re-export types and enums
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
} from '@/generated/api';

export { ResponseCode, SuccessCode, ErrorCode, successCodeToJSON, errorCodeToJSON } from '@/generated/api';
