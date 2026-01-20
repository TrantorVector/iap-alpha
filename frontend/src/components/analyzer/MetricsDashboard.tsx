import { useState, memo } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { MetricsResponse, MetricRow as ApiMetricRow } from "@/api/types";
import { MetricRow } from "./MetricRow";
import { Skeleton } from "@/components/ui/skeleton";

export interface MetricsDashboardProps {
  /** Metrics data from API */
  data: MetricsResponse | undefined;
  /** Loading state */
  isLoading: boolean;
}

/**
 * Section configuration mapping API sections to display titles
 */
const SECTION_TITLES: Record<keyof MetricsResponse["sections"], string> = {
  growth_and_margins: "Growth & Margins",
  cash_and_leverage: "Cash & Leverage",
  valuation: "Valuation Metrics",
};

/**
 * Loading skeleton for a metric section
 */
function MetricSectionSkeleton({ rowCount }: { rowCount: number }) {
  return (
    <div className="space-y-2">
      {Array.from({ length: rowCount }).map((_, i) => (
        <div key={i} className="flex gap-4">
          <Skeleton className="h-10 w-48" />
          <Skeleton className="h-10 flex-1" />
          <Skeleton className="h-10 flex-1" />
          <Skeleton className="h-10 flex-1" />
          <Skeleton className="h-10 flex-1" />
        </div>
      ))}
    </div>
  );
}

/**
 * Collapsible section component
 */
const MetricSectionComponent = memo(function MetricSectionComponent({
  title,
  metrics,
  periods,
}: {
  title: string;
  metrics: ApiMetricRow[];
  periods: string[];
}) {
  const [isCollapsed, setIsCollapsed] = useState(false);

  return (
    <div className="border border-gray-200 rounded-lg overflow-hidden bg-white">
      {/* Section header */}
      <button
        onClick={() => setIsCollapsed(!isCollapsed)}
        className="w-full flex items-center justify-between px-4 py-3 bg-gray-50 hover:bg-gray-100 transition-colors border-b border-gray-200"
        aria-expanded={!isCollapsed}
        aria-label={`${isCollapsed ? "Expand" : "Collapse"} ${title} section`}
      >
        <h3 className="text-base font-semibold text-gray-900">{title}</h3>
        {isCollapsed ? (
          <ChevronRight className="h-5 w-5 text-gray-500" aria-hidden="true" />
        ) : (
          <ChevronDown className="h-5 w-5 text-gray-500" aria-hidden="true" />
        )}
      </button>

      {/* Section content */}
      {!isCollapsed && (
        <div className="overflow-x-auto">
          <table className="w-full" role="table" aria-label={`${title} metrics`}>
            <thead>
              <tr className="border-b border-gray-200 bg-gray-50/50">
                <th
                  className="py-2 px-4 text-left text-xs font-semibold text-gray-700 uppercase tracking-wider sticky left-0 bg-gray-50 border-r border-gray-200"
                  scope="col"
                >
                  Metric
                </th>
                {periods.map((period, index) => (
                  <th
                    key={index}
                    className="py-2 px-4 text-center text-xs font-semibold text-gray-700 uppercase tracking-wider"
                    scope="col"
                  >
                    {period}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {metrics.map((metric) => {
                // Determine if colors should be inverted based on metric name
                // Valuation metrics (lower is better): P/E, P/S, P/B, EV/EBITDA, PEG
                const invertColors =
                  metric.metric_name.includes("_ratio") ||
                  metric.metric_name.includes("price_to") ||
                  metric.metric_name.includes("ev_to") ||
                  metric.metric_name.includes("peg");

                return (
                  <MetricRow
                    key={metric.metric_name}
                    label={metric.display_name}
                    values={metric.values}
                    invertColors={invertColors}
                  />
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
});

/**
 * MetricsDashboard - Pane 1: Key Metrics Dashboard
 *
 * Displays financial metrics in three collapsible sections:
 * 1. Growth & Margins
 * 2. Cash & Leverage
 * 3. Valuation Metrics
 *
 * Features:
 * - Heat map coloring (green = best, orange = worst)
 * - Collapsible sections
 * - Responsive table layout
 * - Loading skeletons
 * - Performance optimized with React.memo
 */
export const MetricsDashboard = memo(function MetricsDashboard({
  data,
  isLoading,
}: MetricsDashboardProps) {
  // Loading state
  if (isLoading || !data) {
    return (
      <div className="space-y-4 p-6" role="status" aria-label="Loading metrics">
        <div className="space-y-4">
          {Object.keys(SECTION_TITLES).map((sectionKey) => (
            <div
              key={sectionKey}
              className="border border-gray-200 rounded-lg p-4 bg-white"
            >
              <Skeleton className="h-6 w-48 mb-4" />
              <MetricSectionSkeleton rowCount={5} />
            </div>
          ))}
        </div>
      </div>
    );
  }

  // Render sections with data
  return (
    <div
      className="space-y-4 p-6 bg-gray-50"
      role="region"
      aria-label="Key Metrics Dashboard"
    >
      <div className="mb-4">
        <h2 className="text-2xl font-bold text-gray-900">
          Key Metrics Dashboard
        </h2>
        <p className="text-sm text-gray-600 mt-1">
          {data.period_type === "quarterly" ? "Quarterly" : "Annual"} metrics
          for {data.periods?.length || 0} periods
        </p>
      </div>

      <div className="space-y-4">
        {data.sections &&
          Object.entries(data.sections).map(([sectionKey, metrics]) => (
            <MetricSectionComponent
              key={sectionKey}
              title={SECTION_TITLES[sectionKey as keyof typeof SECTION_TITLES]}
              metrics={metrics}
              periods={data.periods || []}
            />
          ))}
      </div>
    </div>
  );
});
