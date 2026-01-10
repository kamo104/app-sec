/**
 * Health Check API Endpoint
 */

import {
  ApiResponse,
  ResponseCode,
} from '@/generated/api';
import { API_BASE_URL } from './api-client';

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
    const decoded = ApiResponse.decode(new Uint8Array(responseData));
    return decoded.code === ResponseCode.SUCCESS;
  } catch {
    return false;
  }
}
