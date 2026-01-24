import { TrackerQueryParams } from "@/api/types";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Search, X } from "lucide-react";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import React from "react";

interface TrackerFiltersProps {
    filters: TrackerQueryParams;
    onFilterChange: (filters: TrackerQueryParams) => void;
    sectors: string[];
}

export function TrackerFilters({ filters, onFilterChange, sectors }: TrackerFiltersProps) {
    const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        onFilterChange({ ...filters, search: e.target.value });
    };

    const handleSectorChange = (value: string) => {
        const newSector = value === "all" ? undefined : [value];
        onFilterChange({ ...filters, sector: newSector });
    };

    const handleDateChange = (field: "date_from" | "date_to", value: string) => {
        onFilterChange({ ...filters, [field]: value || undefined });
    };

    const toggleVerdict = (verdict: string) => {
        const current = filters.verdict_type || [];
        const next = current.includes(verdict)
            ? current.filter((v) => v !== verdict)
            : [...current, verdict];
        onFilterChange({
            ...filters,
            verdict_type: next.length ? next : undefined,
        });
    };

    return (
        <div className="flex flex-col gap-4 p-4 bg-card border rounded-lg shadow-sm">
            <div className="flex flex-col xl:flex-row gap-4 xl:items-end">
                <div className="flex-1 space-y-2">
                    <label className="text-sm font-medium">Search</label>
                    <div className="relative">
                        <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder="Search by symbol or company..."
                            className="pl-8"
                            value={filters.search || ""}
                            onChange={handleSearchChange}
                        />
                        {filters.search && (
                            <button
                                onClick={() => onFilterChange({ ...filters, search: "" })}
                                className="absolute right-3 top-2.5 text-muted-foreground hover:text-foreground"
                            >
                                <X className="h-4 w-4" />
                            </button>
                        )}
                    </div>
                </div>

                <div className="w-full xl:w-48 space-y-2">
                    <label className="text-sm font-medium">Sector</label>
                    <Select
                        value={filters.sector?.[0] || "all"}
                        onValueChange={handleSectorChange}
                    >
                        <SelectTrigger>
                            <SelectValue placeholder="All Sectors" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Sectors</SelectItem>
                            {sectors.map((s) => (
                                <SelectItem key={s} value={s}>
                                    {s}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>

                <div className="flex gap-2 w-full xl:w-auto">
                    <div className="space-y-2 flex-1 xl:flex-none">
                        <label className="text-sm font-medium">From Date</label>
                        <Input
                            type="date"
                            value={filters.date_from || ""}
                            onChange={(e) => handleDateChange("date_from", e.target.value)}
                        />
                    </div>

                    <div className="space-y-2 flex-1 xl:flex-none">
                        <label className="text-sm font-medium">To Date</label>
                        <Input
                            type="date"
                            value={filters.date_to || ""}
                            onChange={(e) => handleDateChange("date_to", e.target.value)}
                        />
                    </div>
                </div>

                <Button
                    variant="ghost"
                    onClick={() => onFilterChange({})}
                    className="shrink-0"
                >
                    Reset Filters
                </Button>
            </div>

            <div className="flex flex-wrap gap-2 items-center pt-2 border-t">
                <span className="text-sm font-medium mr-2">Verdicts:</span>
                {["Invest", "Watchlist", "Pass", "No Thesis"].map((v) => {
                    const isSelected = filters.verdict_type?.includes(v);
                    return (
                        <Badge
                            key={v}
                            variant={isSelected ? "default" : "outline"}
                            className="cursor-pointer hover:bg-primary/90 hover:text-primary-foreground selection:bg-none"
                            onClick={() => toggleVerdict(v)}
                        >
                            {v}
                        </Badge>
                    )
                })}
            </div>
        </div>
    );
}
