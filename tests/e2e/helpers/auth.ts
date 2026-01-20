import { Page } from '@playwright/test';

export interface TestUser {
    username: string;
    password: string;
}

export const TEST_USER: TestUser = {
    username: 'testuser',
    password: 'TestPass123!',
};

/**
 * Logs in a user via the API and stores the auth tokens in storage
 */
export async function loginUser(page: Page, user: TestUser = TEST_USER) {
    // Visit the page to establish context
    await page.goto('/');

    // Login via API to get tokens
    const response = await page.request.post('http://localhost:8080/api/v1/auth/login', {
        data: {
            username: user.username,
            password: user.password,
        },
    });

    if (!response.ok()) {
        throw new Error(`Login failed: ${response.status()} ${await response.text()}`);
    }

    const data = await response.json();

    // Store tokens in localStorage
    await page.evaluate((tokens) => {
        localStorage.setItem('access_token', tokens.access_token);
        localStorage.setItem('refresh_token', tokens.refresh_token);
    }, data);

    // Reload to apply auth
    await page.reload();
}

/**
 * Logs out the current user
 */
export async function logoutUser(page: Page) {
    await page.evaluate(() => {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
    });
}
