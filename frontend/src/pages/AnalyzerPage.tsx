import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { companies, verdicts } from '@/api/endpoints';
import { Skeleton } from '@/components/ui/skeleton';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useQueryClient } from '@tanstack/react-query';
import { ControlsBar } from '@/components/analyzer/ControlsBar';

export default function AnalyzerPage() {
    const { companyId } = useParams<{ companyId: string }>();
    const navigate = useNavigate();
    const [periodType, setPeriodType] = useState<'quarterly' | 'annual'>('quarterly');
    const [periodCount, setPeriodCount] = useState(8);
    const queryClient = useQueryClient();

    const { data: company, isLoading: _isLoadingCompany, error: companyError } = useQuery({
        queryKey: ['company', companyId],
        queryFn: () => companies.getDetails(companyId!),
        enabled: !!companyId,
    });

    const { data: _metrics, isLoading: _isLoadingMetrics, error: metricsError } = useQuery({
        queryKey: ['metrics', companyId, periodType, periodCount],
        queryFn: () => companies.getMetrics(companyId!, { period_type: periodType, period_count: periodCount }),
        enabled: !!companyId,
    });

    const { data: _documents, isLoading: _isLoadingDocs, error: docsError } = useQuery({
        queryKey: ['documents', companyId],
        queryFn: () => companies.getDocuments(companyId!),
        enabled: !!companyId,
    });

    const { data: _verdict, isLoading: _isLoadingVerdict, error: verdictError } = useQuery({
        queryKey: ['verdict', companyId],
        queryFn: () => verdicts.get(companyId!),
        enabled: !!companyId,
    });

    // const _isLoading = isLoadingCompany || isLoadingMetrics || isLoadingDocs || isLoadingVerdict;
    const anyError = companyError || metricsError || docsError || verdictError;


    return (
        <div className="flex flex-col h-screen overflow-hidden bg-slate-50 dark:bg-slate-950 font-sans">
            <ControlsBar
                company={company}
                periodType={periodType}
                periodCount={periodCount}
                onPeriodTypeChange={(type) => setPeriodType(type as 'quarterly' | 'annual')}
                onPeriodCountChange={setPeriodCount}
                onRefresh={() => {
                    queryClient.invalidateQueries({ queryKey: ['company', companyId] });
                    queryClient.invalidateQueries({ queryKey: ['metrics', companyId] });
                    queryClient.invalidateQueries({ queryKey: ['documents', companyId] });
                    queryClient.invalidateQueries({ queryKey: ['verdict', companyId] });
                }}
                onClose={() => navigate('/')}
            />

            <main className="flex-1 flex flex-col overflow-hidden container max-w-[1600px] mx-auto px-6 py-4 gap-4">
                {anyError ? (
                    <div className="flex-1 flex items-center justify-center p-8">
                        <div className="max-w-md w-full">
                            <Alert variant="destructive" className="shadow-lg border-2 bg-white dark:bg-slate-900">
                                <AlertCircle className="h-5 w-5" />
                                <AlertTitle className="text-lg font-bold">Error Loading Data</AlertTitle>
                                <AlertDescription className="mt-2 text-sm opacity-90">
                                    We encountered a problem fetching the financial data for this company ({companyId}).
                                    Please check your connection or try again later.
                                </AlertDescription>
                                <Button
                                    variant="outline"
                                    className="mt-4 w-full border-destructive/50 hover:bg-destructive/10"
                                    onClick={() => queryClient.refetchQueries()}
                                >
                                    Retry Refetch
                                </Button>
                            </Alert>
                        </div>
                    </div>
                ) : (
                    <>
                        {/* Pane 1: Key Metrics Dashboard */}
                        <section className="flex-[6] min-h-0 bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden flex flex-col">
                            <div className="p-4 border-b bg-slate-50/50 dark:bg-slate-900/50 flex items-center justify-between">
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
                                    <div className="min-h-[400px] flex flex-col items-center justify-center text-slate-400 dark:text-slate-500 bg-slate-50/30 dark:bg-slate-900/30 rounded-lg border-2 border-dashed border-slate-200 dark:border-slate-800">
                                        <div className="w-16 h-16 mb-4 rounded-2xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
                                            <span className="text-2xl opacity-50">📊</span>
                                        </div>
                                        <p className="font-medium">Metrics Dashboard Visualizer</p>
                                        <p className="text-sm opacity-70">Implementation pending Step 8.5</p>
                                    </div>
                                )}
                            </div>
                        </section>

                        {/* Pane 2: Document Grid */}
                        <section className="flex-[3] min-h-0 bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden flex flex-col">
                            <div className="p-4 border-b bg-slate-50/50 dark:bg-slate-900/50 flex items-center justify-between">
                                <h2 className="font-semibold text-slate-900 dark:text-white flex items-center gap-2">
                                    <span className="w-1 h-4 bg-emerald-600 rounded-full" />
                                    Document Repository
                                </h2>
                                <div className="h-6 w-32 bg-slate-200 dark:bg-slate-800 rounded animate-pulse" />
                            </div>
                            <div className="flex-1 overflow-auto p-4">
                                {_isLoadingDocs ? (
                                    <div className="grid grid-cols-4 gap-4">
                                        {[...Array(8)].map((_, i) => (
                                            <Skeleton key={i} className="h-24 w-full rounded-lg" />
                                        ))}
                                    </div>
                                ) : (
                                    <div className="min-h-[200px] flex flex-col items-center justify-center text-slate-400 dark:text-slate-500 bg-slate-50/30 dark:bg-slate-900/30 rounded-lg border-2 border-dashed border-slate-200 dark:border-slate-800">
                                        <div className="w-12 h-12 mb-3 rounded-xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
                                            <span className="text-xl opacity-50">📂</span>
                                        </div>
                                        <p className="font-medium text-sm">Document Grid Visualizer</p>
                                        <p className="text-xs opacity-70 mt-1">Implementation pending Step 8.6</p>
                                    </div>
                                )}
                            </div>
                        </section>

                        {/* Pane 3: Verdict Recording */}
                        <section className="h-32 shrink-0 bg-white dark:bg-slate-900 rounded-xl border-t-4 border-t-indigo-600 border border-slate-200 dark:border-slate-800 shadow-lg overflow-hidden flex flex-col">
                            <div className="p-3 border-b bg-slate-50/50 dark:bg-slate-900/50 flex items-center justify-between">
                                <h2 className="font-bold text-sm uppercase tracking-wider text-indigo-600 dark:text-indigo-400">
                                    Analysis Verdict
                                </h2>
                            </div>
                            <div className="flex-1 p-4 flex items-center justify-center">
                                {_isLoadingVerdict ? (
                                    <div className="flex gap-4 w-full">
                                        <Skeleton className="h-10 w-32" />
                                        <Skeleton className="h-10 flex-1" />
                                        <Skeleton className="h-10 w-24" />
                                    </div>
                                ) : (
                                    <div className="w-full flex items-center justify-between text-slate-400 dark:text-slate-500">
                                        <div className="flex items-center gap-4">
                                            <div className="w-10 h-10 rounded-full bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
                                                <span className="opacity-50">⚖️</span>
                                            </div>
                                            <p className="text-sm font-medium">Verdict Recording Module</p>
                                        </div>
                                        <p className="text-xs italic opacity-60">Pending Step 8.7 implementation</p>
                                    </div>
                                )}
                            </div>
                        </section>
                    </>
                )}
            </main>
        </div>
    );
}
