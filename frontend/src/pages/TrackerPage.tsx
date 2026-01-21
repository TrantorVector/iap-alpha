import { useState, useEffect, useCallback } from "react";
import { SummaryCards } from "@/components/tracker/SummaryCards";
import { TrackerFilters } from "@/components/tracker/TrackerFilters";
import { TrackerTable, TrackerItem } from "@/components/tracker/TrackerTable";

interface TrackerSummary {
    total_analyzed: number;
    invest_count: number;
    pass_count: number;
    watchlist_count: number;
    no_thesis_count: number;
}

export default function TrackerPage() {
    const [summary, setSummary] = useState<TrackerSummary | null>(null);
    const [data, setData] = useState<TrackerItem[]>([]);
    const [total, setTotal] = useState(0);
    const [isLoading, setIsLoading] = useState(true);
    const [page, setPage] = useState(1);
    const [searchValue, setSearchValue] = useState("");
    const [verdictFilter, setVerdictFilter] = useState<string | null>(null);

    const perPage = 20;

    const fetchSummary = useCallback(async () => {
        try {
            const token = localStorage.getItem("access_token");
            const response = await fetch("/api/v1/tracker/summary", {
                headers: { Authorization: `Bearer ${token}` },
            });
            if (response.ok) {
                const data = await response.json();
                setSummary(data);
            }
        } catch (err) {
            console.error("Failed to fetch tracker summary:", err);
        }
    }, []);

    const fetchVerdicts = useCallback(async () => {
        setIsLoading(true);
        try {
            const token = localStorage.getItem("access_token");
            let url = `/api/v1/tracker/verdicts?page=${page}&per_page=${perPage}`;

            if (searchValue) {
                url += `&search=${encodeURIComponent(searchValue)}`;
            }

            if (verdictFilter) {
                url += `&verdict_type=${verdictFilter}`;
            }

            const response = await fetch(url, {
                headers: { Authorization: `Bearer ${token}` },
            });

            if (response.ok) {
                const result = await response.json();
                setData(result.items);
                setTotal(result.total);
            }
        } catch (err) {
            console.error("Failed to fetch tracker verdicts:", err);
        } finally {
            setIsLoading(false);
        }
    }, [page, searchValue, verdictFilter]);

    useEffect(() => {
        fetchSummary();
    }, [fetchSummary]);

    useEffect(() => {
        const timer = setTimeout(() => {
            fetchVerdicts();
        }, 300);
        return () => clearTimeout(timer);
    }, [fetchVerdicts]);

    const handleClearFilters = () => {
        setSearchValue("");
        setVerdictFilter(null);
        setPage(1);
    };

    const handleVerdictCardClick = (verdict: string | null) => {
        setVerdictFilter(verdict);
        setPage(1);
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

                <SummaryCards
                    summary={summary}
                    onFilterChange={handleVerdictCardClick}
                    activeFilter={verdictFilter}
                />

                <div className="space-y-4">
                    <TrackerFilters
                        searchValue={searchValue}
                        onSearchChange={(val) => {
                            setSearchValue(val);
                            setPage(1);
                        }}
                        onClearFilters={handleClearFilters}
                        activeFilterCount={verdictFilter ? 1 : 0}
                    />

                    <TrackerTable
                        data={data}
                        page={page}
                        perPage={perPage}
                        total={total}
                        onPageChange={setPage}
                        isLoading={isLoading}
                    />
                </div>
            </div>
        </div>
    );
}
