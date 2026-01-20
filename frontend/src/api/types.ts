export interface UserInfo {
  id: string;
  username: string;
  email: string;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: UserInfo;
}

export interface RefreshResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
}

export interface CompanyDetails {
  id: string;
  symbol: string;
  name: string;
  exchange: string;
  sector: string | null;
  market_cap: number | null;
  market_cap_formatted: string;
  currency: string;
  fiscal_year_end_month: number;
  is_active: boolean;
  last_updated: string;
}

export interface MetricValue {
  period: string;
  value: number | null;
  formatted: string;
  heat_map_quartile: number | null;
}

export interface MetricRow {
  metric_name: string;
  display_name: string;
  values: MetricValue[];
  heat_map_enabled: boolean;
}

export interface MetricsSections {
  growth_and_margins: MetricRow[];
  cash_and_leverage: MetricRow[];
  valuation: MetricRow[];
}

export interface MetricsResponse {
  company_id: string;
  period_type: string;
  periods: string[];
  sections: MetricsSections;
}

export interface Document {
  id: string;
  document_type: string;
  period_end_date: string | null;
  fiscal_year: number;
  fiscal_quarter: number | null;
  title: string;
  source_url: string | null;
  storage_key: string | null;
  file_size: number | null;
  mime_type: string | null;
  available: boolean;
}

export interface FreshnessMetadata {
  last_refreshed_at: string | null;
  is_stale: boolean;
  refresh_requested: boolean;
}

export interface DocumentsResponse {
  documents: Document[];
  freshness: FreshnessMetadata;
}

export interface LinkedReport {
  report_id: string;
  filename: string;
  uploaded_at: string;
}

export interface VerdictResponse {
  verdict_id: string | null;
  company_id: string;
  final_verdict: string | null;
  summary_text: string | null;
  strengths: string[];
  weaknesses: string[];
  guidance_summary: string | null;
  lock_version: number;
  created_at: string | null;
  updated_at: string | null;
  linked_reports: LinkedReport[];
}

export interface VerdictUpdateRequest {
  lock_version: number;
  final_verdict: string | null;
  summary_text: string | null;
  strengths: string[];
  weaknesses: string[];
  guidance_summary: string | null;
  linked_report_ids: string[];
}

export interface VerdictHistoryEntry {
  history_id: string;
  version: number;
  final_verdict: string | null;
  summary_text: string | null;
  recorded_at: string;
  linked_report: LinkedReport | null;
}

export interface VerdictHistoryResponse {
  company_id: string;
  history: VerdictHistoryEntry[];
}

export interface DownloadResponse {
  download_url: string;
  expires_in: number;
  filename: string;
  content_type: string;
}

export interface ApiErrorResponse {
  error: string;
  details?: unknown;
}

export interface FilterCriteria {
  market_cap_min?: number;
  market_cap_max?: number;
  exchanges?: string[];
  sectors?: string[];
  momentum_1m_min?: number;
  momentum_3m_min?: number;
  momentum_6m_min?: number;
  verdict_types?: string[];
  has_verdict?: boolean;
}

export interface Screener {
  id: string;
  title: string;
  description: string | null;
  filter_criteria: FilterCriteria;
  sort_config: any;
  display_columns: any;
  display_order: number | null;
  created_at: string;
  updated_at: string;
  // This is a UI field, backend doesn't have it in the main table but ScreenerResultsResponse has executed_at
  last_run_at?: string | null;
}

export interface CreateScreener {
  title: string;
  description?: string;
  filter_criteria: FilterCriteria;
  sort_config?: any;
  display_columns?: any;
}

export interface UpdateScreener {
  title?: string;
  description?: string;
  filter_criteria?: FilterCriteria;
  sort_config?: any;
  display_columns?: any;
}

export interface ScreenerResultsResponse {
  screener_id: string;
  executed_at: string;
  total_results: number;
  results: ScreenerResult[];
}

export interface ScreenerResult {
  company_id: string;
  symbol: string;
  company_name: string;
  exchange: string;
  sector: string | null;
  industry: string | null;
  market_cap: number;
  market_cap_formatted: string;
  momentum_1m: number | null;
  momentum_3m: number | null;
  momentum_6m: number | null;
  revenue_yoy_growth: number | null;
  operating_margin: number | null;
  verdict: string | null;
  last_analyzed: string | null;
  guidance_summary: string | null;
}
