import { useState, useEffect, useRef, useMemo, useCallback } from "react";
import { useParams, useNavigate, useBlocker } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { companies, verdicts } from "@/api/endpoints";
import { Skeleton } from "@/components/ui/skeleton";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { AlertCircle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useQueryClient } from "@tanstack/react-query";
import { ControlsBar } from "@/components/analyzer/ControlsBar";
import { MetricsDashboard } from "@/components/analyzer/MetricsDashboard";
import { DocumentGrid } from "@/components/analyzer/DocumentGrid";
import {
  VerdictForm,
  VerdictFormHandle,
} from "@/components/analyzer/VerdictForm";
import { ConfirmCloseDialog } from "@/components/analyzer/ConfirmCloseDialog";
import { GripHorizontal } from "lucide-react";

export default function AnalyzerPage() {
  const { companyId } = useParams<{ companyId: string }>();
  const navigate = useNavigate();
  const [periodType, setPeriodType] = useState<"quarterly" | "annual">(
    "quarterly",
  );
  const [periodCount, setPeriodCount] = useState(8);
  const [isFormDirty, setIsFormDirty] = useState(false);
  const [metricsHeight, setMetricsHeight] = useState(500);
  const [isResizing, setIsResizing] = useState(false);
  const verdictFormRef = useRef<VerdictFormHandle>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const queryClient = useQueryClient();

  const { data: company, error: companyError } = useQuery({
    queryKey: ["company", companyId],
    queryFn: () => companies.getDetails(companyId!),
    enabled: !!companyId,
  });

  const {
    data: _metrics,
    isLoading: _isLoadingMetrics,
    error: metricsError,
    refetch: refetchMetrics,
  } = useQuery({
    queryKey: ["metrics", companyId, periodType, periodCount],
    queryFn: () =>
      companies.getMetrics(companyId!, {
        period_type: periodType,
        period_count: periodCount,
      }),
    enabled: !!companyId,
  });

  const {
    data: _documents,
    isLoading: _isLoadingDocs,
    error: docsError,
    refetch: refetchDocuments,
  } = useQuery({
    queryKey: ["documents", companyId],
    queryFn: () => companies.getDocuments(companyId!),
    enabled: !!companyId,
  });

  const {
    data: _verdict,
    isLoading: _isLoadingVerdict,
    error: verdictError,
    refetch: refetchVerdict,
  } = useQuery({
    queryKey: ["verdict", companyId],
    queryFn: () => verdicts.get(companyId!),
    enabled: !!companyId,
  });

  // Memoized handlers for performance
  const handlePeriodTypeChange = useCallback((type: string) => {
    setPeriodType(type as "quarterly" | "annual");
  }, []);

  const handlePeriodCountChange = useCallback((count: number) => {
    setPeriodCount(count);
  }, []);

  const handleRefresh = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: ["company", companyId] });
    queryClient.invalidateQueries({ queryKey: ["metrics", companyId] });
    queryClient.invalidateQueries({ queryKey: ["documents", companyId] });
    queryClient.invalidateQueries({ queryKey: ["verdict", companyId] });
  }, [queryClient, companyId]);

  const handleClose = useCallback(() => {
    navigate("/");
  }, [navigate]);

  const handleVerdictSaved = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: ["verdict", companyId] });
  }, [queryClient, companyId]);

  // Navigation Blocking Logic
  const hasVerdict = !!_verdict?.final_verdict;
  const isDataLoaded = !_isLoadingVerdict && !!_verdict;
  const shouldBlock = isDataLoaded && (isFormDirty || !hasVerdict);

  const blocker = useBlocker(
    ({
      currentLocation,
      nextLocation,
    }: {
      currentLocation: { pathname: string };
      nextLocation: { pathname: string };
    }) => shouldBlock && currentLocation.pathname !== nextLocation.pathname,
  );

  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (shouldBlock) {
        e.preventDefault();
        e.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [shouldBlock]);

  // Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+S / Cmd+S to save
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        verdictFormRef.current?.submit();
      }
      // Escape to close
      if (e.key === "Escape") {
        e.preventDefault();
        navigate("/");
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigate]);

  // Resizing Logic
  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      // Calculate new height based on mouse Y relative to wrapper top
      // This assumes the metrics pane is the first child in the wrapper (main)
      // A more robust way: use ref to the metrics pane itself or track delta
      if (wrapperRef.current) {
        // Find relative Y
        // We know Metrics Dashboard is the first section.
        // But wrapperRef is 'main'.
        // Let's just use movementY for simplicity if we can't find exact calculation easily?
        // No, delta is better.
        setMetricsHeight((h) => Math.max(300, Math.min(1000, h + e.movementY)));
      }
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.body.style.cursor = "default";
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    document.body.style.cursor = "row-resize";

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "default";
    };
  }, [isResizing]);

  // const _isLoading = isLoadingCompany || isLoadingMetrics || isLoadingDocs || isLoadingVerdict;
  const anyError = companyError || metricsError || docsError || verdictError;

  return (
    <div
      className="flex flex-col h-screen overflow-hidden bg-slate-50 dark:bg-slate-950 font-sans"
      role="main"
      aria-label="Analyzer Module"
    >
      <ControlsBar
        company={company}
        periodType={periodType}
        periodCount={periodCount}
        onPeriodTypeChange={handlePeriodTypeChange}
        onPeriodCountChange={handlePeriodCountChange}
        onRefresh={handleRefresh}
        onClose={handleClose}
      />

      <main
        ref={wrapperRef}
        className="flex-1 flex flex-col overflow-y-auto container max-w-[1600px] mx-auto px-6 py-4 gap-6 scroll-smooth"
      >
        {anyError ? (
          <div
            className="flex-1 flex items-center justify-center p-8"
            role="alert"
            aria-live="assertive"
          >
            <div className="max-w-md w-full">
              <Alert
                variant="destructive"
                className="shadow-lg border-2 bg-white dark:bg-slate-900"
              >
                <AlertCircle className="h-5 w-5" aria-hidden="true" />
                <AlertTitle className="text-lg font-bold">
                  Error Loading Data
                </AlertTitle>
                <AlertDescription className="mt-2 text-sm opacity-90">
                  {metricsError && <p>Metrics: Failed to load metrics data.</p>}
                  {docsError && <p>Documents: Failed to load document data.</p>}
                  {verdictError && <p>Verdict: Failed to load verdict data.</p>}
                  {companyError && (
                    <p>Company: Failed to load company details.</p>
                  )}
                  <p className="mt-2">
                    Please check your connection or try again.
                  </p>
                </AlertDescription>
                <div className="mt-4 space-y-2">
                  {metricsError && (
                    <Button
                      variant="outline"
                      className="w-full border-destructive/50 hover:bg-destructive/10"
                      onClick={() => refetchMetrics()}
                      aria-label="Retry loading metrics"
                    >
                      <RefreshCw className="mr-2 h-4 w-4" aria-hidden="true" />
                      Retry Metrics
                    </Button>
                  )}
                  {docsError && (
                    <Button
                      variant="outline"
                      className="w-full border-destructive/50 hover:bg-destructive/10"
                      onClick={() => refetchDocuments()}
                      aria-label="Retry loading documents"
                    >
                      <RefreshCw className="mr-2 h-4 w-4" aria-hidden="true" />
                      Retry Documents
                    </Button>
                  )}
                  {verdictError && (
                    <Button
                      variant="outline"
                      className="w-full border-destructive/50 hover:bg-destructive/10"
                      onClick={() => refetchVerdict()}
                      aria-label="Retry loading verdict"
                    >
                      <RefreshCw className="mr-2 h-4 w-4" aria-hidden="true" />
                      Retry Verdict
                    </Button>
                  )}
                  <Button
                    variant="outline"
                    className="w-full border-destructive/50 hover:bg-destructive/10"
                    onClick={() => queryClient.refetchQueries()}
                    aria-label="Retry all data"
                  >
                    <RefreshCw className="mr-2 h-4 w-4" aria-hidden="true" />
                    Retry All
                  </Button>
                </div>
              </Alert>
            </div>
          </div>
        ) : (
          <>
            {/* Pane 1: Key Metrics Dashboard */}
            <section
              style={{ height: metricsHeight }}
              className="bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden flex flex-col transition-height duration-0 ease-linear"
            >
              <div className="p-4 border-b bg-slate-50/50 dark:bg-slate-900/50 flex items-center justify-between flex-shrink-0">
                <h2 className="font-semibold text-slate-900 dark:text-white flex items-center gap-2">
                  <span className="w-1 h-4 bg-blue-600 rounded-full" />
                  Key Financial Metrics
                </h2>
                <div className="h-6 w-24 bg-slate-200 dark:bg-slate-800 rounded animate-pulse" />
              </div>
              <div className="flex-1 overflow-auto p-4">
                {_isLoadingMetrics ? (
                  <div className="space-y-4">
                    {[...Array(6)].map((_, i) => (
                      <div key={i} className="flex gap-4">
                        <Skeleton className="h-10 w-48" />
                        <Skeleton className="h-10 flex-1" />
                        <Skeleton className="h-10 flex-1" />
                        <Skeleton className="h-10 flex-1" />
                        <Skeleton className="h-10 flex-1" />
                      </div>
                    ))}
                  </div>
                ) : (
                  <MetricsDashboard
                    data={_metrics}
                    isLoading={_isLoadingMetrics}
                  />
                )}
              </div>
            </section>

            {/* Resize Handle */}
            <div
              className="-my-3 py-3 flex justify-center cursor-row-resize hover:bg-slate-100 dark:hover:bg-slate-800 rounded opacity-0 hover:opacity-100 transition-opacity z-10"
              onMouseDown={(e) => {
                e.preventDefault();
                setIsResizing(true);
              }}
            >
              <div className="h-1 w-32 bg-slate-300 dark:bg-slate-700 rounded-full flex items-center justify-center">
                <GripHorizontal className="h-4 w-4 text-slate-400" />
              </div>
            </div>

            {/* Pane 2: Document Grid */}
            <section className="min-h-[400px] bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden flex flex-col">
              <div className="p-4 border-b bg-slate-50/50 dark:bg-slate-900/50 flex items-center justify-between">
                <h2 className="font-semibold text-slate-900 dark:text-white flex items-center gap-2">
                  <span className="w-1 h-4 bg-emerald-600 rounded-full" />
                  Document Repository
                </h2>
                <div className="h-6 w-32 bg-slate-200 dark:bg-slate-800 rounded animate-pulse" />
              </div>
              <div className="flex-1 overflow-auto p-4">
                <DocumentGrid
                  companyId={companyId!}
                  periods={_metrics?.periods || []}
                  data={_documents}
                  isLoading={_isLoadingDocs}
                />
              </div>
            </section>

            {/* Pane 3: Verdict Recording */}
            <section className="min-h-[600px] bg-white dark:bg-slate-900 rounded-xl border-t-4 border-t-indigo-600 border border-slate-200 dark:border-slate-800 shadow-lg overflow-hidden flex flex-col mb-8">
              <div className="p-3 border-b bg-slate-50/50 dark:bg-slate-900/50 flex items-center justify-between">
                <h2 className="font-bold text-sm uppercase tracking-wider text-indigo-600 dark:text-indigo-400">
                  Analysis Verdict
                </h2>
              </div>
              <div className="flex-1 p-4 flex flex-col">
                {_isLoadingVerdict ? (
                  <div className="space-y-4 max-w-2xl mx-auto w-full pt-8">
                    <div className="flex gap-4">
                      <Skeleton className="h-12 w-32 rounded-full" />
                      <Skeleton className="h-12 w-32 rounded-full" />
                      <Skeleton className="h-12 w-32 rounded-full" />
                    </div>
                    <Skeleton className="h-32 w-full" />
                    <div className="grid grid-cols-2 gap-4">
                      <Skeleton className="h-40 w-full" />
                      <Skeleton className="h-40 w-full" />
                    </div>
                  </div>
                ) : (
                  <VerdictForm
                    ref={verdictFormRef}
                    companyId={companyId!}
                    initialData={_verdict!}
                    onSaved={handleVerdictSaved}
                    onDirtyChange={setIsFormDirty}
                  />
                )}
              </div>
            </section>
          </>
        )}
      </main>

      {blocker.state === "blocked" && (
        <ConfirmCloseDialog
          open={true}
          onOpenChange={(open) => {
            if (!open) blocker.reset();
          }}
          onConfirm={() => blocker.proceed()}
          onCancel={() => blocker.reset()}
        />
      )}
    </div>
  );
}
