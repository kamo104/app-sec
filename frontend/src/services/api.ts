/**
 * API Service
 *
 * Handles communication with the backend API server.
 */

// Base URL for API requests
// In production, this would be configured via environment variables
const API_BASE_URL = 'http://localhost:4000/api';

/**
 * Generic API response type matching the backend ApiResponse<T>
 */
export interface ApiResponse<T = unknown> {
  success: boolean;
  message: string;
  data: T | null;
}

/**
 * Registration request payload
 */
export interface RegistrationRequest {
  username: string;
  email: string;
  password: string;
}

/**
 * Registration response data
 * Currently the backend returns empty data on success, but we keep this extensible
 */
export type RegistrationResponse = ApiResponse<null>;

/**
 * Login request payload
 */
export interface LoginRequest {
  username: string;
  password: string;
}

/**
 * Login response data
 */
export interface LoginResponseData {
  username: string;
  email: string;
}

/**
 * Login response
 */
export type LoginResponse = ApiResponse<LoginResponseData>;

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
 * Generic fetch wrapper for API calls
 */
async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const url = `${API_BASE_URL}${endpoint}`;

  // Set up default headers
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...options.headers as Record<string, string>,
  };

  try {
    const response = await fetch(url, {
      ...options,
      headers,
    });

    // Handle non-2xx responses
    if (!response.ok) {
      let errorMessage = `HTTP ${response.status}`;
      let errorCode: string | undefined;

      // Try to parse error response
      try {
        const errorData: ApiResponse<null> = await response.json();
        errorMessage = errorData.message;
        errorCode = response.status === 409 ? 'CONFLICT' :
                    response.status === 400 ? 'BAD_REQUEST' :
                    response.status === 401 ? 'UNAUTHORIZED' : undefined;
      } catch {
        // Fallback if response body isn't JSON
        errorMessage = await response.text();
      }

      throw new ApiError(errorMessage, response.status, errorCode);
    }

    // Parse successful response
    const data: ApiResponse<T> = await response.json();
    return data;
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
  data: RegistrationRequest
): Promise<RegistrationResponse> {
  return apiFetch<null>('/register', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/**
 * Health check for the API
 */
export async function healthCheck(): Promise<ApiResponse<{ status: string }>> {
  try {
    const result = await apiFetch<{ status: string }>('/health', {
      method: 'GET',
    });
    return result;
  } catch (error) {
    // Return a failed response for health check
    return {
      success: false,
      message: error instanceof ApiError ? error.message : 'Health check failed',
      data: null,
    };
  }
}

/**
 * Login a user
 */
export async function loginUser(
  data: LoginRequest
): Promise<LoginResponse> {
  return apiFetch<LoginResponseData>('/login', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}
