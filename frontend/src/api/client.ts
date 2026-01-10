/**
 * API Client Configuration
 *
 * Configures the generated OpenAPI client with base URL and credentials.
 */

import { client } from '@/generated/api-client/client.gen';

// Configure the client
client.setConfig({
  baseUrl: 'http://localhost:4000',
  credentials: 'include',
});

// Re-export everything from the generated client for convenience
export * from '@/generated/api-client';
