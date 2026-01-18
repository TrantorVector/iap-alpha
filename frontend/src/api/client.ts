import { RefreshResponse } from './types';

const BASE_URL = import.meta.env.VITE_API_URL || '/api/v1';

class ApiError extends Error {
    status: number;
    data: any;

    constructor(status: number, data: any) {
        super(data?.error || `API Error: ${status}`);
        this.name = 'ApiError';
        this.status = status;
        this.data = data;
    }
}

interface RequestOptions extends RequestInit {
    params?: Record<string, string | number | boolean | undefined>;
}

async function handleResponse<T>(response: Response): Promise<T> {
    if (response.status === 204) {
        return {} as T;
    }

    const data = await response.json().catch(() => ({}));

    if (!response.ok) {
        throw new ApiError(response.status, data);
    }

    return data as T;
}

const getAuthToken = () => localStorage.getItem('access_token');
const getRefreshToken = () => localStorage.getItem('refresh_token');

const setTokens = (accessToken: string, refreshToken?: string) => {
    localStorage.setItem('access_token', accessToken);
    if (refreshToken) {
        localStorage.setItem('refresh_token', refreshToken);
    }
};

const clearTokens = () => {
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('user');
};

let isRefreshing = false;
let refreshSubscribers: ((token: string) => void)[] = [];

const subscribeTokenRefresh = (cb: (token: string) => void) => {
    refreshSubscribers.push(cb);
};

const onRrefreshed = (token: string) => {
    refreshSubscribers.map((cb) => cb(token));
    refreshSubscribers = [];
};

async function refreshToken(): Promise<string | null> {
    const rt = getRefreshToken();
    if (!rt) return null;

    try {
        const response = await fetch(`${BASE_URL}/auth/refresh`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ refresh_token: rt }),
        });

        if (!response.ok) {
            throw new Error('Refresh failed');
        }

        const data: RefreshResponse = await response.json();
        setTokens(data.access_token);
        return data.access_token;
    } catch (error) {
        clearTokens();
        window.location.href = '/login';
        return null;
    }
}

async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
    const { params, headers: customHeaders, ...rest } = options;

    let url = `${BASE_URL}${path}`;
    if (params) {
        const searchParams = new URLSearchParams();
        Object.entries(params).forEach(([key, value]) => {
            if (value !== undefined) {
                searchParams.append(key, String(value));
            }
        });
        const queryString = searchParams.toString();
        if (queryString) {
            url += `?${queryString}`;
        }
    }

    const token = getAuthToken();
    const headers = new Headers(customHeaders);
    if (token && !headers.has('Authorization')) {
        headers.set('Authorization', `Bearer ${token}`);
    }
    if (!headers.has('Content-Type') && !(rest.body instanceof FormData)) {
        headers.set('Content-Type', 'application/json');
    }

    const executeRequest = () => fetch(url, { ...rest, headers });

    let response = await executeRequest();

    if (response.status === 401) {
        if (!isRefreshing) {
            isRefreshing = true;
            const newToken = await refreshToken();
            isRefreshing = false;
            if (newToken) {
                onRrefreshed(newToken);
            }
        }

        if (getRefreshToken()) {
            return new Promise((resolve) => {
                subscribeTokenRefresh(async (newToken) => {
                    headers.set('Authorization', `Bearer ${newToken}`);
                    resolve(handleResponse<T>(await executeRequest()));
                });
            });
        }
    }

    return handleResponse<T>(response);
}

export const client = {
    get: <T>(path: string, params?: RequestOptions['params'], options?: RequestOptions) =>
        request<T>(path, { ...options, method: 'GET', params }),

    post: <T>(path: string, body?: any, options?: RequestOptions) =>
        request<T>(path, {
            ...options,
            method: 'POST',
            body: body instanceof FormData ? body : JSON.stringify(body)
        }),

    put: <T>(path: string, body?: any, options?: RequestOptions) =>
        request<T>(path, {
            ...options,
            method: 'PUT',
            body: body instanceof FormData ? body : JSON.stringify(body)
        }),

    delete: <T>(path: string, options?: RequestOptions) =>
        request<T>(path, { ...options, method: 'DELETE' }),

    setTokens,
    clearTokens,
};
