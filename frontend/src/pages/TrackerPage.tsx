import { useState, useEffect } from "react";
import { SummaryCards } from "@/components/tracker/SummaryCards";
import { TrackerFilters } from "@/components/tracker/TrackerFilters";
import { TrackerTable } from "@/components/tracker/TrackerTable";
import { PaginationState } from "@tanstack/react-table";
import { tracker } from "@/api/endpoints";
import { TrackerSummaryResponse, TrackerItemOut, TrackerQueryParams } from "@/api/types";

export default function TrackerPage() {
    const [summary, setSummary] = useState<TrackerSummaryResponse | null>(null);
    const [items, setItems] = useState<TrackerItemOut[]>([]);
    const [total, setTotal] = useState(0);
    const [isLoading, setIsLoading] = useState(true);

    // Filters and Pagination
    const [filters, setFilters] = useState<TrackerQueryParams>({});
    const [pagination, setPagination] = useState<PaginationState>({
        pageIndex: 0,
        pageSize: 20,
    });

    const sectors = [
        "Technology", "Healthcare", "Financial Services", "Consumer Cyclical",
        "Industrials", "Energy", "Utilities", "Real Estate",
        "Basic Materials", "Communication Services", "Consumer Defensive"
    ];

    useEffect(() => {
        fetchSummary();
    }, []);

    useEffect(() => {
        fetchVerdicts();
    }, [filters, pagination]);

    const fetchSummary = async () => {
        try {
            const data = await tracker.getSummary();
            setSummary(data);
        } catch (err) {
            console.error("Failed to fetch tracker summary:", err);
        }
    };

    const fetchVerdicts = async () => {
        setIsLoading(true);
        try {
            const params: TrackerQueryParams = {
                ...filters,
                page: pagination.pageIndex + 1,
                per_page: pagination.pageSize,
            };
            const result = await tracker.getVerdicts(params);
            setItems(result.items);
            setTotal(result.total);
        } catch (err) {
            console.error("Failed to fetch tracker verdicts:", err);
        } finally {
            setIsLoading(false);
        }
    };

    const handleFilterChange = (newFilters: TrackerQueryParams) => {
        setFilters(newFilters);
        setPagination((prev) => ({ ...prev, pageIndex: 0 }));
    };

    return (
        <div className="min-h-screen bg-slate-50/50 dark:bg-slate-950/50 p-6 space-y-8">
            <div className="max-w-7xl mx-auto space-y-8">
                <header className="flex flex-col space-y-2">
                    <h1 className="text-4xl font-extrabold tracking-tight text-primary">
                        Results Tracker
                    </h1>
                    <p className="text-muted-foreground font-medium">
                        Monitor and track all your investment research and recorded verdicts.
                    </p>
                </header>

                {summary && (
                    <SummaryCards
                        summary={summary}
                        onFilter={(verdict) => {
                            // Logic: If click a card, set verdict filter to [verdict]. 
                            // If already selected, clear it.
                            const isSame = filters.verdict_type?.length === 1 && filters.verdict_type[0] === verdict;
                            handleFilterChange({
                                ...filters,
                                verdict_type: (verdict && !isSame) ? [verdict] : undefined
                            });
                        }}
                        activeFilter={filters.verdict_type?.length === 1 ? filters.verdict_type[0] : null}
                    />
                )}

                <div className="space-y-4">
                    <TrackerFilters
                        filters={filters}
                        onFilterChange={handleFilterChange}
                        sectors={sectors}
                    />

                    <TrackerTable
                        data={items}
                        total={total}
                        pagination={pagination}
                        onPaginationChange={setPagination}
                        isLoading={isLoading}
                    />
                </div>
            </div>
        </div>
    );
}
