import React, { useState, useEffect } from "react";
import { CompanyDetails } from "@/api/types";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RefreshCw, X, Pin, PinOff } from "lucide-react";
import { cn } from "@/lib/utils";

interface ControlsBarProps {
  company: CompanyDetails | undefined;
  periodType: string;
  periodCount: number;
  onPeriodTypeChange: (type: string) => void;
  onPeriodCountChange: (count: number) => void;
  onRefresh: () => void;
  onClose: () => void;
}

const PREF_PINNED_KEY = "analyzer-controls-pinned";

export const ControlsBar: React.FC<ControlsBarProps> = ({
  company,
  periodType,
  periodCount,
  onPeriodTypeChange,
  onPeriodCountChange,
  onRefresh,
  onClose,
}) => {
  const [isPinned, setIsPinned] = useState<boolean>(() => {
    const saved = localStorage.getItem(PREF_PINNED_KEY);
    return saved === "true";
  });

  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    localStorage.setItem(PREF_PINNED_KEY, String(isPinned));
  }, [isPinned]);

  return (
    <header
      onMouseEnter={() => setIsVisible(true)}
      onMouseLeave={() => setIsVisible(false)}
      className={cn(
        "fixed top-0 left-0 right-0 z-50 transition-all duration-300 ease-in-out",
        "border-b bg-white/80 dark:bg-slate-900/80 backdrop-blur-xl shadow-sm",
        !isPinned && !isVisible
          ? "-translate-y-[calc(100%-4px)] opacity-0 hover:opacity-100"
          : "translate-y-0 opacity-100",
      )}
    >
      {/* Hover Trigger Area (when not pinned) */}
      {!isPinned && !isVisible && (
        <div className="absolute bottom-0 left-1/2 -translate-x-1/2 translate-y-full px-4 py-1 bg-slate-200/50 dark:bg-slate-800/50 rounded-b-full cursor-pointer hover:bg-slate-300 dark:hover:bg-slate-700 transition-colors border border-t-0 border-slate-300 dark:border-slate-700">
          <div className="w-8 h-1 bg-slate-400 dark:bg-slate-500 rounded-full" />
        </div>
      )}

      <div className="container max-w-[1600px] mx-auto flex h-14 items-center gap-6 px-6">
        {/* Left: Company Info */}
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            aria-label="Close"
            className="h-8 w-8 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors shrink-0"
          >
            <X className="h-4 w-4" />
          </Button>
          <div className="flex flex-col min-w-0">
            <h1 className="text-sm font-bold tracking-tight text-slate-900 dark:text-white leading-none truncate">
              {company?.name || "Loading..."}
            </h1>
            <p className="text-[10px] font-medium text-slate-500 dark:text-slate-400 mt-0.5 truncate uppercase tracking-wider">
              {company?.symbol}{" "}
              {company?.exchange ? `• ${company.exchange}` : ""}
            </p>
          </div>
        </div>

        {/* Center: Controls */}
        <div className="flex items-center gap-4">
          <Tabs
            value={periodType}
            onValueChange={onPeriodTypeChange}
            className="w-auto"
          >
            <TabsList className="h-8 bg-slate-100 dark:bg-slate-800 p-0.5">
              <TabsTrigger value="quarterly" className="h-7 text-xs px-3">
                Quarterly
              </TabsTrigger>
              <TabsTrigger value="annual" className="h-7 text-xs px-3">
                Annual
              </TabsTrigger>
            </TabsList>
          </Tabs>

          <div className="flex items-center gap-2">
            <span className="text-[10px] font-semibold uppercase tracking-wider text-slate-400 dark:text-slate-500">
              Periods
            </span>
            <Select
              value={String(periodCount)}
              onValueChange={(val) => onPeriodCountChange(parseInt(val))}
            >
              <SelectTrigger
                className="h-8 w-[70px] text-xs bg-slate-100/50 dark:bg-slate-800/50 border-none"
                aria-label="Period count"
              >
                <SelectValue placeholder="Count" />
              </SelectTrigger>
              <SelectContent>
                {[4, 5, 6, 7, 8, 9, 10].map((num) => (
                  <SelectItem key={num} value={String(num)} className="text-xs">
                    {num}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Right: Actions */}
        <div className="flex items-center gap-2 flex-1 justify-end">
          <Button
            variant="ghost"
            size="sm"
            onClick={onRefresh}
            className="h-8 gap-2 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:hover:text-blue-400"
          >
            <RefreshCw className="h-3.5 w-3.5" />
            <span className="hidden sm:inline">Refresh Data</span>
          </Button>

          <div className="w-px h-4 bg-slate-200 dark:bg-slate-700 mx-1" />

          <Button
            variant="ghost"
            size="icon"
            onClick={() => setIsPinned(!isPinned)}
            className={cn(
              "h-8 w-8 rounded-full transition-colors",
              isPinned
                ? "text-blue-600 bg-blue-50 dark:bg-blue-900/20"
                : "text-slate-400 hover:text-slate-600 dark:hover:text-slate-300",
            )}
            title={isPinned ? "Unpin controls" : "Pin controls"}
          >
            {isPinned ? (
              <PinOff className="h-4 w-4 fill-current" />
            ) : (
              <Pin className="h-4 w-4" />
            )}
          </Button>
        </div>
      </div>
    </header>
  );
};
