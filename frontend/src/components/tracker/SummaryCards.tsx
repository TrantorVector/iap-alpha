import { Card, CardContent } from "@/components/ui/card";
import {
    BarChart3,
    TrendingUp,
    Slash,
    Eye,
    FileSearch
} from "lucide-react";
import { cn } from "@/lib/utils";

interface SummaryCardsProps {
    summary: {
        total_analyzed: number;
        invest_count: number;
        pass_count: number;
        watchlist_count: number;
        no_thesis_count: number;
    } | null;
    onFilterChange: (verdict: string | null) => void;
    activeFilter: string | null;
}

export function SummaryCards({ summary, onFilterChange, activeFilter }: SummaryCardsProps) {
    const cards = [
        {
            title: "Total Analyzed",
            value: summary?.total_analyzed ?? 0,
            icon: BarChart3,
            color: "text-blue-500",
            bg: "bg-blue-50 dark:bg-blue-950/30",
            filter: null,
        },
        {
            title: "Invest",
            value: summary?.invest_count ?? 0,
            icon: TrendingUp,
            color: "text-emerald-500",
            bg: "bg-emerald-50 dark:bg-emerald-950/30",
            filter: "invest",
        },
        {
            title: "Pass",
            value: summary?.pass_count ?? 0,
            icon: Slash,
            color: "text-slate-500",
            bg: "bg-slate-50 dark:bg-slate-950/30",
            filter: "pass",
        },
        {
            title: "Watchlist",
            value: summary?.watchlist_count ?? 0,
            icon: Eye,
            color: "text-amber-500",
            bg: "bg-amber-50 dark:bg-amber-950/30",
            filter: "watchlist",
        },
        {
            title: "No Thesis",
            value: summary?.no_thesis_count ?? 0,
            icon: FileSearch,
            color: "text-rose-500",
            bg: "bg-rose-50 dark:bg-rose-950/30",
            filter: "no_thesis",
        },
    ];

    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
            {cards.map((card) => {
                const Icon = card.icon;
                const isActive = activeFilter === card.filter;

                return (
                    <Card
                        key={card.title}
                        className={cn(
                            "cursor-pointer transition-all hover:shadow-md border-2",
                            isActive ? "border-primary" : "border-transparent",
                            card.bg
                        )}
                        onClick={() => onFilterChange(card.filter)}
                    >
                        <CardContent className="p-4 flex items-center justify-between">
                            <div>
                                <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/80">
                                    {card.title}
                                </p>
                                <p className="text-2xl font-bold mt-1">
                                    {card.value}
                                </p>
                            </div>
                            <div className={cn("p-2 rounded-full bg-white dark:bg-slate-900 border", card.color)}>
                                <Icon className="h-5 w-5" />
                            </div>
                        </CardContent>
                    </Card>
                );
            })}
        </div>
    );
}
