import { Input } from "@/components/ui/card"; // Wait, Input is usually in ui/input
import { Search, Filter, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

// Let's check where Input is
// Assuming standard shadcn/ui paths

interface TrackerFiltersProps {
    onSearchChange: (value: string) => void;
    searchValue: string;
    onClearFilters: () => void;
    activeFilterCount: number;
}

export function TrackerFilters({
    onSearchChange,
    searchValue,
    onClearFilters,
    activeFilterCount
}: TrackerFiltersProps) {
    return (
        <div className="flex flex-col sm:flex-row gap-4 items-center justify-between bg-card p-4 rounded-xl border shadow-sm">
            <div className="relative w-full sm:w-80">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <input
                    type="text"
                    placeholder="Search by symbol or name..."
                    className="w-full pl-10 pr-4 py-2 rounded-lg border bg-background text-sm focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all"
                    value={searchValue}
                    onChange={(e) => onSearchChange(e.target.value)}
                />
                {searchValue && (
                    <button
                        onClick={() => onSearchChange("")}
                        className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    >
                        <X className="h-4 w-4" />
                    </button>
                )}
            </div>

            <div className="flex items-center gap-3 w-full sm:w-auto">
                <div className="flex -space-x-1">
                    {/* Placeholder for future multi-select filters */}
                    <Button variant="outline" size="sm" className="rounded-r-none h-9">
                        <Filter className="mr-2 h-4 w-4" />
                        Filters
                        {activeFilterCount > 0 && (
                            <Badge variant="secondary" className="ml-2 px-1 text-[10px] h-4 min-w-[16px]">
                                {activeFilterCount}
                            </Badge>
                        )}
                    </Button>
                    <Button
                        variant="outline"
                        size="sm"
                        className="rounded-l-none border-l-0 h-9"
                        onClick={onClearFilters}
                        disabled={activeFilterCount === 0 && !searchValue}
                    >
                        Clear
                    </Button>
                </div>
            </div>
        </div>
    );
}
