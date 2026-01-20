import { Screener } from "@/api/types";
import { ScreenerCard } from "./ScreenerCard";
import { Button } from "@/components/ui/button";
import { Plus } from "lucide-react";

interface ScreenerListProps {
  screeners: Screener[];
  selectedId: string | null;
  onSelect: (screener: Screener) => void;
  onCreate: () => void;
  onRun: (id: string) => void;
  onEdit: (screener: Screener) => void;
  onDelete: (id: string) => void;
}

export function ScreenerList({
  screeners,
  selectedId,
  onSelect,
  onCreate,
  onRun,
  onEdit,
  onDelete,
}: ScreenerListProps) {
  return (
    <div className="flex flex-col h-full gap-4 border-r pr-4 bg-background">
      <div className="flex items-center justify-between px-1 pt-2">
        <h2 className="text-xl font-semibold tracking-tight">Screeners</h2>
        <Button onClick={onCreate} size="sm" className="gap-1">
          <Plus className="h-4 w-4" /> New
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto pr-3 scrollbar-thin scrollbar-thumb-muted scrollbar-track-transparent">
        <div className="space-y-3 pb-8">
          {screeners.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-8 text-center text-muted-foreground border border-dashed rounded-lg bg-muted/20">
              <p className="text-sm">No screeners found</p>
              <Button
                variant="link"
                onClick={onCreate}
                className="mt-2 h-auto p-0"
              >
                Create one
              </Button>
            </div>
          ) : (
            screeners.map((screener) => (
              <div
                key={screener.id}
                onClick={() => onSelect(screener)}
                className="cursor-pointer"
              >
                <ScreenerCard
                  screener={screener}
                  isSelected={screener.id === selectedId}
                  onRun={onRun}
                  onEdit={onEdit}
                  onDelete={onDelete}
                />
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
