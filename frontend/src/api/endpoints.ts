import { client } from "./client";
import * as T from "./types";

export const auth = {
  login: (credentials: { username: string; password: string }) =>
    client.post<T.LoginResponse>("/auth/login", credentials).then((res) => {
      client.setTokens(res.access_token, res.refresh_token);
      localStorage.setItem("user", JSON.stringify(res.user));
      return res;
    }),

  refresh: (refreshToken: string) =>
    client.post<T.RefreshResponse>("/auth/refresh", {
      refresh_token: refreshToken,
    }),

  logout: (refreshToken?: string) =>
    client
      .post<void>("/auth/logout", { refresh_token: refreshToken })
      .finally(() => {
        client.clearTokens();
      }),
};

export const companies = {
  getDetails: (companyId: string) =>
    client.get<T.CompanyDetails>(`/companies/${companyId}`),

  getMetrics: (
    companyId: string,
    options?: { period_type?: string; period_count?: number },
  ) =>
    client.get<T.MetricsResponse>(`/companies/${companyId}/metrics`, options),

  getDocuments: (companyId: string, options?: { document_type?: string }) =>
    client.get<T.DocumentsResponse>(
      `/companies/${companyId}/documents`,
      options,
    ),

  uploadDocument: (
    companyId: string,
    file: File,
    metadata: { document_type: string; period_end_date?: string },
  ) => {
    const formData = new FormData();
    formData.append("file", file);
    formData.append("document_type", metadata.document_type);
    if (metadata.period_end_date) {
      formData.append("period_end_date", metadata.period_end_date);
    }
    return client.post<T.Document>(
      `/companies/${companyId}/documents`,
      formData,
    );
  },

  getDownloadUrl: (companyId: string, docId: string) =>
    client.get<T.DownloadResponse>(
      `/companies/${companyId}/documents/${docId}/download`,
    ),
};

export const verdicts = {
  get: (companyId: string) =>
    client.get<T.VerdictResponse>(`/companies/${companyId}/verdict`),

  update: (companyId: string, update: T.VerdictUpdateRequest) =>
    client.put<T.VerdictResponse>(`/companies/${companyId}/verdict`, update),

  getHistory: (companyId: string) =>
    client.get<T.VerdictHistoryResponse>(
      `/companies/${companyId}/verdict/history`,
    ),
};
export const screeners = {
  list: () => client.get<T.Screener[]>("/screeners"),

  get: (id: string) => client.get<T.Screener>(`/screeners/${id}`),

  create: (screener: T.CreateScreener) =>
    client.post<T.Screener>("/screeners", screener),

  update: (id: string, screener: T.UpdateScreener) =>
    client.put<T.Screener>(`/screeners/${id}`, screener),

  delete: (id: string) => client.delete<void>(`/screeners/${id}`),

  run: (id: string, overrideCriteria?: T.FilterCriteria) =>
    client.post<T.ScreenerResultsResponse>(`/screeners/${id}/run`, {
      override_criteria: overrideCriteria,
    }),
};
