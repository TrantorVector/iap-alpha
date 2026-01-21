import { useNavigate } from "react-router-dom";
import {
    ColumnDef,
    flexRender,
    getCoreRowModel,
    useReactTable,
} from "@tanstack/react-table";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { format } from "date-fns";
import { ExternalLink, ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";

export interface TrackerItem {
    company_id: string;
    symbol: string;
    company_name: string;
    exchange: string;
    sector: string | null;
    verdict: string;
    verdict_date: string;
    summary_text: string;
    version: number;
}

interface TrackerTableProps {
    data: TrackerItem[];
    page: number;
    perPage: number;
    total: number;
    onPageChange: (page: number) => void;
    isLoading: boolean;
}

export function TrackerTable({
    data,
    page,
    perPage,
    total,
    onPageChange,
    isLoading
}: TrackerTableProps) {
    const navigate = useNavigate();

    const columns: ColumnDef<TrackerItem>[] = [
        {
            accessorKey: "symbol",
            header: "Symbol",
            cell: ({ row }) => (
                <div className="font-bold text-primary">{row.original.symbol}</div>
            ),
        },
        {
            accessorKey: "company_name",
            header: "Company",
            cell: ({ row }) => (
                <div className="max-w-[200px] truncate font-medium">
                    {row.original.company_name}
                </div>
            ),
        },
        {
            accessorKey: "exchange",
            header: "Exchange",
        },
        {
            accessorKey: "sector",
            header: "Sector",
            cell: ({ row }) => row.original.sector || "-",
        },
        {
            accessorKey: "verdict",
            header: "Verdict",
            cell: ({ row }) => {
                const verdict = row.original.verdict;
                const variants: Record<string, string> = {
                    invest: "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-400 border-emerald-200",
                    pass: "bg-slate-100 text-slate-800 dark:bg-slate-800 dark:text-slate-400 border-slate-200",
                    watchlist: "bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400 border-amber-200",
                    no_thesis: "bg-rose-100 text-rose-800 dark:bg-rose-900/30 dark:text-rose-400 border-rose-200",
                };

                return (
                    <Badge
                        variant="outline"
                        className={cn("capitalize", variants[verdict] || "")}
                    >
                        {verdict.replace("_", " ")}
                    </Badge>
                );
            },
        },
        {
            accessorKey: "verdict_date",
            header: "Date",
            cell: ({ row }) => format(new Date(row.original.verdict_date), "MMM d, yyyy"),
        },
        {
            accessorKey: "summary_text",
            header: "Summary",
            cell: ({ row }) => (
                <div className="max-w-[300px] truncate text-xs text-muted-foreground italic">
                    "{row.original.summary_text}"
                </div>
            ),
        },
        {
            id: "actions",
            cell: ({ row }) => (
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={(e) => {
                        e.stopPropagation();
                        navigate(`/analyzer/${row.original.company_id}`);
                    }}
                >
                    <ExternalLink className="h-4 w-4" />
                </Button>
            ),
        },
    ];

    const table = useReactTable({
        data,
        columns,
        getCoreRowModel: getCoreRowModel(),
    });

    const totalPages = Math.ceil(total / perPage);

    return (
        <div className="space-y-4">
            <div className="rounded-xl border bg-card shadow-sm overflow-hidden">
                <Table>
                    <TableHeader className="bg-muted/50">
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => (
                                    <TableHead key={header.id} className="font-semibold">
                                        {flexRender(
                                            header.column.columnDef.header,
                                            header.getContext()
                                        )}
                                    </TableHead>
                                ))}
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            Array.from({ length: 5 }).map((_, i) => (
                                <TableRow key={i}>
                                    {columns.map((_, j) => (
                                        <TableCell key={j}>
                                            <div className="h-4 w-full bg-muted animate-pulse rounded" />
                                        </TableCell>
                                    ))}
                                </TableRow>
                            ))
                        ) : data.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={columns.length} className="h-32 text-center text-muted-foreground">
                                    No analysis results found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            table.getRowModel().rows.map((row) => (
                                <TableRow
                                    key={row.id}
                                    className="cursor-pointer hover:bg-muted/50 transition-colors"
                                    onClick={() => navigate(`/analyzer/${row.original.company_id}`)}
                                >
                                    {row.getVisibleCells().map((cell) => (
                                        <TableCell key={cell.id}>
                                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                        </TableCell>
                                    ))}
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            <div className="flex items-center justify-between px-2">
                <div className="text-sm text-muted-foreground font-medium">
                    Showing <span className="text-foreground">{data.length}</span> of <span className="text-foreground">{total}</span> results
                </div>
                <div className="flex items-center space-x-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onPageChange(page - 1)}
                        disabled={page <= 1 || isLoading}
                        className="h-8 w-8 p-0"
                    >
                        <ChevronLeft className="h-4 w-4" />
                    </Button>
                    <div className="text-sm font-semibold">
                        Page {page} of {totalPages || 1}
                    </div>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onPageChange(page + 1)}
                        disabled={page >= totalPages || isLoading}
                        className="h-8 w-8 p-0"
                    >
                        <ChevronRight className="h-4 w-4" />
                    </Button>
                </div>
            </div>
        </div>
    );
}

// Helper function for class merging
function cn(...classes: (string | boolean | undefined)[]) {
    return classes.filter(Boolean).join(" ");
}
