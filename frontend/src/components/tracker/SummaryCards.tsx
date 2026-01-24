import { TrackerSummaryResponse } from "@/api/types";
import { CheckCircle, XCircle, AlertCircle, MinusCircle, BarChart3 } from "lucide-react";
import { cn } from "@/lib/utils";

interface SummaryCardsProps {
    summary: TrackerSummaryResponse;
    onFilter: (verdict: string | null) => void;
    activeFilter: string | null;
}

export function SummaryCards({ summary, onFilter, activeFilter }: SummaryCardsProps) {
    const cards = [
        {
            label: "Invest",
            count: summary.invest_count,
            icon: CheckCircle,
            color: "text-green-500",
            bg: "bg-green-500/10",
            border: "border-green-500/20",
            verdict: "Invest",
        },
        {
            label: "Watchlist",
            count: summary.watchlist_count,
            icon: AlertCircle,
            color: "text-yellow-500",
            bg: "bg-yellow-500/10",
            border: "border-yellow-500/20",
            verdict: "Watchlist",
        },
        {
            label: "Pass",
            count: summary.pass_count,
            icon: MinusCircle,
            color: "text-gray-500",
            bg: "bg-gray-500/10",
            border: "border-gray-500/20",
            verdict: "Pass",
        },
        {
            label: "No Thesis",
            count: summary.no_thesis_count,
            icon: XCircle,
            color: "text-red-500",
            bg: "bg-red-500/10",
            border: "border-red-500/20",
            verdict: "No Thesis",
        },
    ];

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
            <div className="bg-card p-4 rounded-lg border flex items-center gap-4">
                <div className="p-3 rounded-full bg-primary/10 text-primary">
                    <BarChart3 className="w-6 h-6" />
                </div>
                <div>
                    <p className="text-sm text-muted-foreground font-medium">Total Analyzed</p>
                    <p className="text-2xl font-bold">{summary.total_analyzed}</p>
                </div>
            </div>

            {cards.map((card) => (
                <button
                    key={card.label}
                    onClick={() => onFilter(activeFilter === card.verdict ? null : card.verdict)}
                    className={cn(
                        "bg-card p-4 rounded-lg border flex items-center gap-4 transition-all hover:bg-accent/50 text-left",
                        activeFilter === card.verdict ? `ring-2 ring-primary ring-offset-1` : ""
                    )}
                >
                    <div className={cn("p-3 rounded-full", card.bg, card.color)}>
                        <card.icon className="w-6 h-6" />
                    </div>
                    <div>
                        <p className="text-sm text-muted-foreground font-medium">{card.label}</p>
                        <p className="text-2xl font-bold">{card.count}</p>
                    </div>
                </button>
            ))}
        </div>
    );
}
