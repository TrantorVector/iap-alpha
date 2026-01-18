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
import { MetricsDashboard } from '@/components/analyzer/MetricsDashboard';
import { DocumentGrid } from '@/components/analyzer/DocumentGrid';
import { VerdictForm } from '@/components/analyzer/VerdictForm';

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

            <main className="flex-1 flex flex-col overflow-y-auto container max-w-[1600px] mx-auto px-6 py-4 gap-6 scroll-smooth">
                {anyError ? (
                    <div className="flex-1 flex items-center justify-center p-8">
                        <div className="max-w-md w-full">
                            <Alert variant="destructive" className="shadow-lg border-2 bg-white dark:bg-slate-900">
                                <AlertCircle className="h-5 w-5" />
                                <AlertTitle className="text-lg font-bold">Error Loading Data</AlertTitle>
                                <AlertDescription className="mt-2 text-sm opacity-90">
                                    We encountered a problem fetching the financial data for this company.
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
                        <section className="min-h-[500px] bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden flex flex-col">
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
                                    <MetricsDashboard data={_metrics} isLoading={_isLoadingMetrics} />
                                )}
                            </div>
                        </section>

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
                                        companyId={companyId!}
                                        initialData={_verdict!}
                                        onSaved={() => {
                                            queryClient.invalidateQueries({ queryKey: ['verdict', companyId] });
                                            // Also refresh history if we had that pane
                                        }}
                                    />
                                )}
                            </div>
                        </section>
                    </>
                )}
            </main>
        </div>
    );
}
