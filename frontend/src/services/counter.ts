/**
 * Counter API Endpoints
 */

import {
  ApiResponse,
  CounterData,
  SetCounterRequest,
  ErrorCode,
} from '@/generated/api';
import { makeApiRequest, ApiError } from './api-client';

/**
 * Extracts counter data from ApiResponse.
 */
function extractCounterData(response: ApiResponse): CounterData {
  if (!response.data?.counterData) {
    throw new ApiError('Invalid response: missing counter data', 500, ErrorCode.INTERNAL);
  }
  return response.data.counterData;
}

/**
 * Get the current counter value.
 */
export async function getCounter(): Promise<{ counterData: CounterData; bytes: Uint8Array }> {
  const result = await makeApiRequest('/counter/get', 'GET');
  return { counterData: extractCounterData(result.response), bytes: result.bytes };
}

/**
 * Set the counter to a specific value.
 */
export async function setCounter(value: number): Promise<{ counterData: CounterData; bytes: Uint8Array }> {
  const request: SetCounterRequest = { value };
  const requestBytes = SetCounterRequest.encode(request).finish();
  const result = await makeApiRequest('/counter/set', 'POST', requestBytes);
  return { counterData: extractCounterData(result.response), bytes: result.bytes };
}
