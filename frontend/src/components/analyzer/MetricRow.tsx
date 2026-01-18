import { MetricValue } from "@/api/types";
import {
  calculateHeatMapColor,
  getHeatMapOpacity,
  getHeatMapTextColor,
  HeatMapConfig,
} from "@/lib/heatmap";

export interface MetricRowProps {
  /** Display name of the metric (e.g., "Revenue ($B)") */
  label: string;
  /** Array of metric values for each period */
  values: (MetricValue | null)[];
  /** Whether lower values are better (for valuation metrics) */
  invertColors?: boolean;
  /** Optional formatter function for values */
  formatter?: (value: number) => string;
}

/**
 * Default formatter for metric values
 * Handles decimals and large numbers appropriately
 */
function defaultFormatter(value: number): string {
  // For very large numbers (billions), show with 2 decimals
  if (Math.abs(value) >= 1000) {
    return value.toFixed(2);
  }
  // For percentages and ratios, show with 2 decimals
  if (Math.abs(value) < 100) {
    return value.toFixed(2);
  }
  // For medium numbers, show with 1 decimal
  return value.toFixed(1);
}

/**
 * MetricRow component displays a single metric across multiple periods
 * with heat map coloring to highlight performance
 */
export function MetricRow({
  label,
  values,
  invertColors = false,
  formatter = defaultFormatter,
}: MetricRowProps) {
  // Extract numeric values for heat map calculation (excluding nulls)
  const numericValues = values
    .map((v) => v?.value)
    .filter((v): v is number => v !== null && v !== undefined && !isNaN(v));

  // Calculate min/max for heat map
  const min = numericValues.length > 0 ? Math.min(...numericValues) : 0;
  const max = numericValues.length > 0 ? Math.max(...numericValues) : 0;

  const heatMapConfig: HeatMapConfig = {
    min,
    max,
    invert: invertColors,
  };

  return (
    <tr className="border-b border-gray-200 hover:bg-gray-50/50 transition-colors">
      {/* Metric label cell */}
      <td className="py-3 px-4 text-sm font-medium text-gray-900 sticky left-0 bg-white border-r border-gray-200">
        {label}
      </td>

      {/* Value cells with heat map coloring */}
      {values.map((metricValue, index) => {
        const value = metricValue?.value;
        const formattedValue = metricValue?.formatted;

        // Handle null/N/A values
        if (value === null || value === undefined) {
          return (
            <td
              key={index}
              className="py-3 px-4 text-sm text-center text-gray-400"
            >
              N/A
            </td>
          );
        }

        // Calculate heat map colors
        const bgColor = calculateHeatMapColor(value, heatMapConfig);
        const opacity = getHeatMapOpacity(value, heatMapConfig);
        const textColor = getHeatMapTextColor(opacity);

        return (
          <td
            key={index}
            className="py-3 px-4 text-sm text-center font-medium transition-all"
            style={{
              backgroundColor: bgColor,
              opacity: opacity + 0.6, // Boost overall visibility
              color: textColor,
            }}
            title={`${label}: ${formattedValue || formatter(value)}`}
          >
            {formattedValue || formatter(value)}
          </td>
        );
      })}
    </tr>
  );
}
