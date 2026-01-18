/**
 * Heat map color calculation for metrics dashboard
 * 
 * Implements gradient coloring from deep green (best) to deep orange (worst)
 * with support for inverted scales (lower is better for valuation metrics)
 */

export interface HeatMapConfig {
    /** Minimum value in the dataset */
    min: number;
    /** Maximum value in the dataset */
    max: number;
    /** If true, lower values are better (for valuation metrics like P/E) */
    invert?: boolean;
}

/**
 * Calculate heat map color for a given value within a range
 * 
 * @param value - The value to get a color for
 * @param config - Heat map configuration (min, max, invert)
 * @returns RGB color string in format "rgb(r, g, b)"
 */
export function calculateHeatMapColor(
    value: number | null | undefined,
    config: HeatMapConfig
): string {
    // Handle null/undefined values
    if (value === null || value === undefined) {
        return 'rgb(156, 163, 175)'; // gray-400 for N/A values
    }

    const { min, max, invert = false } = config;

    // Handle edge cases
    if (min === max) {
        return 'rgb(34, 197, 94)'; // green-500 - neutral color when all values are the same
    }

    // Normalize value to 0-1 range
    let normalized = (value - min) / (max - min);

    // Invert if needed (for valuation metrics where lower is better)
    if (invert) {
        normalized = 1 - normalized;
    }

    // Clamp to 0-1 range
    normalized = Math.max(0, Math.min(1, normalized));

    // Calculate color gradient from orange (0) to green (1)
    // Orange: rgb(249, 115, 22) - orange-600
    // Yellow-ish: rgb(234, 179, 8) - yellow-600 (midpoint)
    // Green: rgb(22, 163, 74) - green-600

    let r: number, g: number, b: number;

    if (normalized < 0.5) {
        // Transition from orange to yellow
        const t = normalized * 2; // 0 to 1
        r = Math.round(249 + (234 - 249) * t);
        g = Math.round(115 + (179 - 115) * t);
        b = Math.round(22 + (8 - 22) * t);
    } else {
        // Transition from yellow to green
        const t = (normalized - 0.5) * 2; // 0 to 1
        r = Math.round(234 + (22 - 234) * t);
        g = Math.round(179 + (163 - 179) * t);
        b = Math.round(8 + (74 - 8) * t);
    }

    return `rgb(${r}, ${g}, ${b})`;
}

/**
 * Get background opacity for heat map cells
 * Higher values get more opacity for better visibility
 */
export function getHeatMapOpacity(
    value: number | null | undefined,
    config: HeatMapConfig
): number {
    if (value === null || value === undefined) {
        return 0.1;
    }

    const { min, max, invert = false } = config;

    if (min === max) {
        return 0.3;
    }

    let normalized = (value - min) / (max - min);

    if (invert) {
        normalized = 1 - normalized;
    }

    normalized = Math.max(0, Math.min(1, normalized));

    // Map to opacity range 0.15 (worst) to 0.4 (best)
    return 0.15 + (normalized * 0.25);
}

/**
 * Get text color that contrasts well with heat map background
 */
export function getHeatMapTextColor(opacity: number): string {
    // For low opacity backgrounds, use dark text
    // For high opacity backgrounds, ensure readability
    return opacity > 0.3 ? 'rgb(31, 41, 55)' : 'rgb(17, 24, 39)'; // gray-800 or gray-900
}
