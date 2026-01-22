import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { Screener, CreateScreener, FilterCriteria } from "@/api/types";

const EXCHANGES = ["NASDAQ", "NYSE", "BSE", "NSE"];
const SECTORS = [
  "Technology",
  "Healthcare",
  "Financial Services",
  "Consumer Cyclical",
  "Consumer Defensive",
  "Energy",
  "Industrials",
  "Real Estate",
  "Communication Services",
  "Utilities",
  "Basic Materials",
];
const VERDICT_TYPES = ["Strong Buy", "Buy", "Hold", "Sell", "Strong Sell"];

const MARKET_CAP_PRESETS = [
  { label: "Small Cap", min: 300_000_000, max: 2_000_000_000 },
  { label: "Mid Cap", min: 2_000_000_000, max: 10_000_000_000 },
  { label: "Large Cap", min: 10_000_000_000, max: 200_000_000_000 },
  { label: "Mega Cap", min: 200_000_000_000, max: undefined },
];

const schema = z.object({
  title: z.string().min(1, "Title is required"),
  description: z.string().optional(),
  filter_criteria: z.object({
    exchanges: z.array(z.string()).default([]),
    sectors: z.array(z.string()).default([]),
    market_cap_min: z.coerce
      .number()
      .optional()
      .or(z.literal(""))
      .transform((v) => (v === "" ? undefined : v)),
    market_cap_max: z.coerce
      .number()
      .optional()
      .or(z.literal(""))
      .transform((v) => (v === "" ? undefined : v)),
    momentum_1m_min: z.coerce
      .number()
      .optional()
      .or(z.literal(""))
      .transform((v) => (v === "" ? undefined : v)),
    momentum_3m_min: z.coerce
      .number()
      .optional()
      .or(z.literal(""))
      .transform((v) => (v === "" ? undefined : v)),
    momentum_6m_min: z.coerce
      .number()
      .optional()
      .or(z.literal(""))
      .transform((v) => (v === "" ? undefined : v)),
    verdict_types: z.array(z.string()).default([]),
    needs_analysis: z.boolean().optional(),
  }),
});

type FormData = z.infer<typeof schema>;

interface ScreenerEditorProps {
  mode: "create" | "edit";
  initialData: Screener | null;
  onSave: (screener: CreateScreener) => void;
  onClose: () => void;
  open: boolean;
}

export function ScreenerEditor({
  mode,
  initialData,
  onSave,
  onClose,
  open,
}: ScreenerEditorProps) {
  const {
    register,
    handleSubmit,
    reset,
    watch,
    setValue,
    formState: { errors, isSubmitting },
  } = useForm<any>({
    resolver: zodResolver(schema),
    defaultValues: {
      title: "",
      description: "",
      filter_criteria: {
        exchanges: [],
        sectors: [],
        verdict_types: [],
        needs_analysis: false,
      },
    } as any,
  });

  useEffect(() => {
    if (open) {
      if (mode === "edit" && initialData) {
        reset({
          title: initialData.title,
          description: initialData.description || "",
          filter_criteria: {
            exchanges: initialData.filter_criteria.exchanges || [],
            sectors: initialData.filter_criteria.sectors || [],
            market_cap_min: initialData.filter_criteria.market_cap_min ?? "",
            market_cap_max: initialData.filter_criteria.market_cap_max ?? "",
            momentum_1m_min: initialData.filter_criteria.momentum_1m_min ?? "",
            momentum_3m_min: initialData.filter_criteria.momentum_3m_min ?? "",
            momentum_6m_min: initialData.filter_criteria.momentum_6m_min ?? "",
            verdict_types: initialData.filter_criteria.verdict_types || [],
            needs_analysis: initialData.filter_criteria.has_verdict === false,
          },
        });
      } else {
        reset({
          title: "",
          description: "",
          filter_criteria: {
            exchanges: [],
            sectors: [],
            market_cap_min: "",
            market_cap_max: "",
            momentum_1m_min: "",
            momentum_3m_min: "",
            momentum_6m_min: "",
            verdict_types: [],
            needs_analysis: false,
          },
        });
      }
    }
  }, [open, mode, initialData, reset]);

  const onSubmit = (data: FormData) => {
    // Map needs_analysis back to has_verdict
    const { needs_analysis, ...criteria } = data.filter_criteria;
    const finalCriteria = {
      ...criteria,
      has_verdict: needs_analysis ? false : undefined,
    } as FilterCriteria;

    onSave({
      title: data.title,
      description: data.description,
      filter_criteria: finalCriteria,
    } as CreateScreener);
  };

  const toggleSelection = (
    field: "exchanges" | "sectors" | "verdict_types",
    value: string,
  ) => {
    // eslint-disable-next-line react-hooks/incompatible-library
    const current = (watch(`filter_criteria.${field}`) as string[]) || [];
    if (current.includes(value)) {
      setValue(
        `filter_criteria.${field}`,
        current.filter((v: string) => v !== value),
      );
    } else {
      setValue(`filter_criteria.${field}`, [...current, value]);
    }
  };

  const applyMarketCapPreset = (min?: number, max?: number) => {
    setValue("filter_criteria.market_cap_min", min ?? "");
    setValue("filter_criteria.market_cap_max", max ?? "");
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onClose()}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {mode === "create" ? "Create New Screener" : "Edit Screener"}
          </DialogTitle>
          <DialogDescription>
            Define your filtering criteria to find the best investment
            opportunities.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 py-4">
          <div className="space-y-4">
            <div className="grid gap-2">
              <Label htmlFor="title">Title *</Label>
              <Input
                id="title"
                placeholder="Growth Stocks 2024"
                {...register("title")}
                className={errors.title ? "border-destructive" : ""}
              />
              {errors.title && (
                <p className="text-xs text-destructive">
                  {errors.title?.message as string}
                </p>
              )}
            </div>

            <div className="grid gap-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                placeholder="Filtering for high growth companies in tech and healthcare..."
                {...register("description")}
              />
            </div>
          </div>

          <div className="space-y-6 border-t pt-6">
            <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              Filters
            </h3>

            {/* Exchange Filter */}
            <div className="space-y-3">
              <Label>Exchanges</Label>
              <div className="flex flex-wrap gap-2">
                {EXCHANGES.map((exchange) => (
                  <Badge
                    key={exchange}
                    variant={
                      watch("filter_criteria.exchanges")?.includes(exchange)
                        ? "default"
                        : "outline"
                    }
                    className="cursor-pointer px-3 py-1"
                    onClick={() => toggleSelection("exchanges", exchange)}
                  >
                    {exchange}
                  </Badge>
                ))}
              </div>
            </div>

            {/* Sector Filter */}
            <div className="space-y-3">
              <Label>Sectors</Label>
              <div className="flex flex-wrap gap-2">
                {SECTORS.map((sector) => (
                  <Badge
                    key={sector}
                    variant={
                      watch("filter_criteria.sectors")?.includes(sector)
                        ? "default"
                        : "outline"
                    }
                    className="cursor-pointer px-3 py-1"
                    onClick={() => toggleSelection("sectors", sector)}
                  >
                    {sector}
                  </Badge>
                ))}
              </div>
            </div>

            {/* Market Cap Filter */}
            <div className="space-y-3">
              <Label>Market Cap (USD)</Label>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <Label
                    htmlFor="market_cap_min"
                    className="text-[10px] text-muted-foreground"
                  >
                    Min
                  </Label>
                  <Input
                    id="market_cap_min"
                    type="number"
                    placeholder="Min"
                    {...register("filter_criteria.market_cap_min")}
                  />
                </div>
                <div className="space-y-1">
                  <Label
                    htmlFor="market_cap_max"
                    className="text-[10px] text-muted-foreground"
                  >
                    Max
                  </Label>
                  <Input
                    id="market_cap_max"
                    type="number"
                    placeholder="Max"
                    {...register("filter_criteria.market_cap_max")}
                  />
                </div>
              </div>
              <div className="flex flex-wrap gap-2 mt-2">
                {MARKET_CAP_PRESETS.map((preset) => (
                  <Button
                    key={preset.label}
                    type="button"
                    variant="outline"
                    size="sm"
                    className="text-[10px] h-7"
                    onClick={() => applyMarketCapPreset(preset.min, preset.max)}
                  >
                    {preset.label}
                  </Button>
                ))}
              </div>
            </div>

            {/* Momentum Filters */}
            <div className="space-y-3">
              <Label>Momentum (Min % Change)</Label>
              <div className="grid grid-cols-3 gap-4">
                <div className="space-y-1">
                  <Label
                    htmlFor="momentum_1m_min"
                    className="text-[10px] text-muted-foreground"
                  >
                    1 Month
                  </Label>
                  <Input
                    id="momentum_1m_min"
                    type="number"
                    step="0.1"
                    placeholder="%"
                    {...register("filter_criteria.momentum_1m_min")}
                  />
                </div>
                <div className="space-y-1">
                  <Label
                    htmlFor="momentum_3m_min"
                    className="text-[10px] text-muted-foreground"
                  >
                    3 Month
                  </Label>
                  <Input
                    id="momentum_3m_min"
                    type="number"
                    step="0.1"
                    placeholder="%"
                    {...register("filter_criteria.momentum_3m_min")}
                  />
                </div>
                <div className="space-y-1">
                  <Label
                    htmlFor="momentum_6m_min"
                    className="text-[10px] text-muted-foreground"
                  >
                    6 Month
                  </Label>
                  <Input
                    id="momentum_6m_min"
                    type="number"
                    step="0.1"
                    placeholder="%"
                    {...register("filter_criteria.momentum_6m_min")}
                  />
                </div>
              </div>
            </div>

            {/* Analysis Status Filter */}
            <div className="space-y-3">
              <Label>Analysis Status</Label>
              <div className="flex items-center space-x-2 mb-4">
                <Checkbox
                  id="needs_analysis"
                  checked={watch("filter_criteria.needs_analysis")}
                  onCheckedChange={(checked) =>
                    setValue("filter_criteria.needs_analysis", checked)
                  }
                />
                <Label
                  htmlFor="needs_analysis"
                  className="text-sm font-normal cursor-pointer"
                >
                  Needs Analysis (No current verdict)
                </Label>
              </div>
              <Label className="text-xs text-muted-foreground">
                Filter by Verdict Type
              </Label>
              <div className="flex flex-wrap gap-2">
                {VERDICT_TYPES.map((type) => (
                  <Badge
                    key={type}
                    variant={
                      watch("filter_criteria.verdict_types")?.includes(type)
                        ? "default"
                        : "outline"
                    }
                    className="cursor-pointer px-3 py-1"
                    onClick={() => toggleSelection("verdict_types", type)}
                  >
                    {type}
                  </Badge>
                ))}
              </div>
            </div>
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button type="button" variant="ghost" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {mode === "create" ? "Create Screener" : "Save Changes"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
