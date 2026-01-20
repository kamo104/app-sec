/**
 * API Client Configuration
 *
 * Configures the generated OpenAPI client with base URL and credentials.
 * Uses empty baseUrl to make requests relative to the current origin,
 * allowing the frontend to work regardless of which host it's served from.
 */

import { client } from '@/generated/api-client/sdk.gen'

// Configure the client with relative URL (same origin)
client.setConfig({
  baseUrl: '',
  credentials: 'include',
})

// Re-export everything from the generated client for convenience
export * from '@/generated/api-client'
