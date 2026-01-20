import { Screener } from "@/api/types";
import { Card, CardHeader, CardTitle, CardDescription, CardFooter } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Play, Edit, Trash2 } from "lucide-react";

interface ScreenerCardProps {
    screener: Screener;
    onRun: (id: string) => void;
    onEdit: (screener: Screener) => void;
    onDelete: (id: string) => void;
    isSelected?: boolean;
}

export function ScreenerCard({ screener, onRun, onEdit, onDelete, isSelected }: ScreenerCardProps) {
    return (
        <Card
            className={`relative transition-colors hover:bg-muted/50 ${isSelected ? "border-primary shadow-sm" : ""}`}
        >
            <CardHeader className="pb-2">
                <div className="flex justify-between items-start">
                    <CardTitle className="text-base font-medium leading-none">{screener.title}</CardTitle>
                </div>
                <CardDescription className="line-clamp-2 text-xs mt-1">
                    {screener.description || "No description provided."}
                </CardDescription>
            </CardHeader>
            <CardFooter className="flex justify-between pt-2 pb-4">
                <div className="text-[10px] text-muted-foreground">
                    Last run: {screener.last_run_at ? new Date(screener.last_run_at).toLocaleDateString() : "Never"}
                </div>
                <div className="flex gap-1" onClick={(e) => e.stopPropagation()}>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => onRun(screener.id)} title="Run Screener">
                        <Play className="h-3.5 w-3.5" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => onEdit(screener)} title="Edit Screener">
                        <Edit className="h-3.5 w-3.5" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-7 w-7 text-destructive hover:text-destructive" onClick={() => onDelete(screener.id)} title="Delete Screener">
                        <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                </div>
            </CardFooter>
        </Card>
    );
}
