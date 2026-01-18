
import { useState, useEffect } from 'react';
import {
    DndContext,
    closestCenter,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    DragEndEvent,
} from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    verticalListSortingStrategy,
    useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { companies } from '@/api/endpoints';
import { Document, DocumentsResponse } from '@/api/types';
import { DocumentCell } from './DocumentCell';
import { RefreshCw, GripVertical } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Skeleton } from '@/components/ui/skeleton';
import { Badge } from '@/components/ui/badge';
import { toast } from '@/hooks/use-toast';

// Document types configuration
interface DocTypeConfig {
    id: string;
    label: string;
    apiType: string;
    uploadable?: boolean;
}

const DEFAULT_DOC_TYPES: DocTypeConfig[] = [
    { id: 'transcript', label: 'Earnings Call Transcript', apiType: 'earnings_transcript' },
    { id: '10q', label: 'Quarterly Report (10-Q)', apiType: '10-q' },
    { id: '10k', label: 'Annual Report (10-K)', apiType: '10-k' },
    { id: 'presentation', label: 'Investor Presentation', apiType: 'presentation' },
    { id: 'analyst', label: 'Analyst Report', apiType: 'analyst_report', uploadable: true },
];

interface DocumentGridProps {
    companyId: string;
    periods: string[];
    data: DocumentsResponse | undefined;
    isLoading: boolean;
}

function SortableRow({
    docType,
    periods,
    documents,
    onUpload,
    onDownload
}: {
    docType: DocTypeConfig;
    periods: string[];
    documents: Document[];
    onUpload: (file: File, period: string) => void;
    onDownload: (doc: Document) => void;
}) {
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging,
    } = useSortable({ id: docType.id });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
        zIndex: isDragging ? 10 : 1,
        position: 'relative' as const,
    };

    return (
        <tr
            ref={setNodeRef}
            style={style}
            className={cn(
                "group border-b border-gray-100 hover:bg-gray-50/50 bg-white",
                isDragging && "shadow-lg opacity-90"
            )}
        >
            <td className="py-2 pl-2 pr-4 sticky left-0 bg-inherit border-r border-gray-100 z-20">
                <div className="flex items-center gap-3">
                    <button
                        {...attributes}
                        {...listeners}
                        className="p-1.5 rounded cursor-grab active:cursor-grabbing text-gray-400 hover:text-gray-600 hover:bg-gray-100 opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                        <GripVertical className="h-4 w-4" />
                    </button>
                    <span className="text-sm font-medium text-gray-700">{docType.label}</span>
                </div>
            </td>
            {periods.map((period) => {
                // Find document matching this period and type
                const matchingDoc = documents.find(d => {
                    // Match period logic:
                    // period is like "Q3-2023" or "FY-2023"
                    // doc has fiscal_year and fiscal_quarter
                    const isQuarterly = period.startsWith('Q');
                    if (isQuarterly) {
                        const [q, y] = period.split('-');
                        const year = parseInt(y);
                        const quarter = parseInt(q.replace('Q', ''));
                        return d.fiscal_year === year && d.fiscal_quarter === quarter && d.document_type === docType.apiType;
                    } else {
                        // Annual - assuming format FY-YYYY or just YYYY
                        // Usually periods are "2023" or "FY-2023" for annual
                        const yearStr = period.replace('FY-', '');
                        const year = parseInt(yearStr);
                        // If parsing fails, it might be just a year number
                        return d.fiscal_year === year && d.document_type === docType.apiType;
                    }
                });

                return (
                    <td key={period} className="p-0 border-l border-gray-100 h-[45px]">
                        <DocumentCell
                            type={docType.label}
                            // period prop is not used in DocumentCell but kept for potential future use or debugging
                            document={matchingDoc}
                            isUploadable={docType.uploadable && !matchingDoc}
                            onUpload={(file) => onUpload(file, period)}
                            onClick={onDownload}
                        />
                    </td>
                );
            })}
        </tr>
    );
}

export function DocumentGrid({ companyId, periods, data, isLoading }: DocumentGridProps) {
    const queryClient = useQueryClient();
    const [docOrder, setDocOrder] = useState(DEFAULT_DOC_TYPES);

    // Load saved preference
    useEffect(() => {
        const savedOrder = localStorage.getItem('document_grid_order');
        if (savedOrder) {
            try {
                const orderIds = JSON.parse(savedOrder) as string[];
                const reordered = orderIds
                    .map(id => DEFAULT_DOC_TYPES.find(d => d.id === id))
                    .filter((d): d is DocTypeConfig => !!d);

                // Add any new default types that aren't in saved order
                const missing = DEFAULT_DOC_TYPES.filter(d => !orderIds.includes(d.id));
                setDocOrder([...reordered, ...missing]);
            } catch (e) {
                console.error('Failed to parse saved document order', e);
            }
        }
    }, []);

    const sensors = useSensors(
        useSensor(PointerSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        })
    );

    const uploadMutation = useMutation({
        mutationFn: async ({ file, period: _period }: { file: File, period: string }) => {
            return companies.uploadDocument(companyId, file, {
                document_type: 'analyst_report',
            });
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['documents', companyId] });
            toast({ title: "Success", description: "Document uploaded successfully" });
        },
        onError: () => {
            toast({ title: "Error", description: "Failed to upload document", variant: "destructive" });
        }
    });

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;

        if (over && active.id !== over.id) {
            setDocOrder((items) => {
                const oldIndex = items.findIndex((item) => item.id === active.id);
                const newIndex = items.findIndex((item) => item.id === over.id);

                const newOrder = arrayMove(items, oldIndex, newIndex);
                localStorage.setItem('document_grid_order', JSON.stringify(newOrder.map(i => i.id)));
                return newOrder;
            });
        }
    };

    const handleDownload = async (doc: Document) => {
        try {
            const { download_url } = await companies.getDownloadUrl(companyId, doc.id);
            window.open(download_url, '_blank');
        } catch (error) {
            toast({ title: "Error", description: "Failed to get download URL", variant: "destructive" });
        }
    };

    const handleUpload = (file: File, period: string) => {
        uploadMutation.mutate({ file, period });
    };

    if (isLoading && !data) {
        return <div className="p-4 space-y-4">
            <div className="flex justify-between">
                <Skeleton className="h-6 w-32" />
                <Skeleton className="h-6 w-24" />
            </div>
            <Skeleton className="h-[200px] w-full" />
        </div>;
    }

    const documents = data?.documents || [];
    const isRefreshing = data?.freshness.refresh_requested || false;
    const isStale = data?.freshness.is_stale || false;

    return (
        <div className="flex flex-col h-full bg-white rounded-lg">
            {/* Local Header with status badges */}
            <div className="px-4 py-2 flex items-center justify-end gap-3 border-b border-gray-100 min-h-[40px]">
                {isRefreshing && (
                    <Badge variant="secondary" className="gap-1 text-xs">
                        <RefreshCw className="h-3 w-3 animate-spin" />
                        Refreshing...
                    </Badge>
                )}
                {isStale && !isRefreshing && (
                    <Badge variant="outline" className="text-amber-600 border-amber-200 bg-amber-50 text-xs">
                        Updates Available
                    </Badge>
                )}
                {!isRefreshing && !isStale && (
                    <span className="text-xs text-gray-400">Up to date</span>
                )}
            </div>

            {/* Grid */}
            <div className="overflow-x-auto flex-1">
                <DndContext
                    sensors={sensors}
                    collisionDetection={closestCenter}
                    onDragEnd={handleDragEnd}
                >
                    <table className="w-full text-sm text-left">
                        <thead className="text-xs text-gray-500 uppercase bg-gray-50 border-b border-gray-200">
                            <tr>
                                <th className="px-4 py-3 font-medium w-64 sticky left-0 bg-gray-50 border-r border-gray-200 z-20 shadow-[2px_0_5px_-2px_rgba(0,0,0,0.1)]">Type</th>
                                {periods.map(period => (
                                    <th key={period} className="px-4 py-3 font-medium text-center min-w-[100px]">
                                        {period}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <SortableContext
                            items={docOrder.map(d => d.id)}
                            strategy={verticalListSortingStrategy}
                        >
                            <tbody className="divide-y divide-gray-100">
                                {docOrder.map(docType => (
                                    <SortableRow
                                        key={docType.id}
                                        docType={docType}
                                        periods={periods}
                                        documents={documents}
                                        onUpload={handleUpload}
                                        onDownload={handleDownload}
                                    />
                                ))}
                            </tbody>
                        </SortableContext>
                    </table>
                </DndContext>
            </div>
        </div>
    );
}
