/**
 * Test Configuration and Constants
 */

export const TEST_CONFIG = {
    // Base URLs
    FRONTEND_URL: process.env.FRONTEND_URL || 'http://localhost:3000',
    API_URL: process.env.API_URL || 'http://localhost:8080',

    // Test user credentials
    TEST_USER: {
        email: process.env.TEST_USER_EMAIL || 'test@example.com',
        password: process.env.TEST_USER_PASSWORD || 'Test123!',
    },

    // Test company IDs - These should be replaced with actual UUIDs from your test data
    COMPANIES: {
        AAPL: process.env.TEST_AAPL_ID || '10000000-0000-0000-0000-000000000001',
        NO_VERDICT: process.env.TEST_NO_VERDICT_ID || '10000000-0000-0000-0000-000000000002',
    },

    // Timeouts
    TIMEOUTS: {
        DEFAULT: 30000,
        NAVIGATION: 60000,
        API: 10000,
    },
};

/**
 * Helper to get company ID for testing
 */
export function getTestCompanyId(symbol: string): string {
    const companyId = TEST_CONFIG.COMPANIES[symbol as keyof typeof TEST_CONFIG.COMPANIES];
    if (!companyId || companyId.includes('placeholder')) {
        console.warn(
            `Warning: Using placeholder company ID for ${symbol}. ` +
            `Set TEST_${symbol}_ID environment variable with actual UUID.`
        );
    }
    return companyId;
}
