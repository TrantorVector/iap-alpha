import { useState, useMemo } from "react";
import {
  ColumnDef,
  SortingState,
  VisibilityState,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  HeaderContext,
  CellContext,
} from "@tanstack/react-table";
import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";

import { Skeleton } from "@/components/ui/skeleton";
import { ScreenerResult } from "@/api/types";
import { ResultRow } from "./ResultRow";
import { Download, Settings2, ArrowUpDown } from "lucide-react";

// Check if DropdownMenu exists in ui
// I didn't see it in the previous list_dir, so I might need to check elsewhere or use a different component.
// Wait, I saw radix-ui/react-dialog but not dropdown. Let me check if I can use select or if I should create a simple one.

interface ResultsTableProps {
  results: ScreenerResult[];
  isLoading: boolean;
}

export function ResultsTable({ results, isLoading }: ResultsTableProps) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});

  const columns = useMemo<ColumnDef<ScreenerResult>[]>(
    () => [
      {
        accessorKey: "symbol",
        header: ({ column }: HeaderContext<ScreenerResult, string>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold"
          >
            Symbol
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        ),
      },
      {
        accessorKey: "company_name",
        header: "Company Name",
      },
      {
        accessorKey: "exchange",
        header: "Exchange",
      },
      {
        accessorKey: "market_cap",
        header: ({ column }: HeaderContext<ScreenerResult, number>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold"
          >
            Market Cap
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        ),
        cell: ({ row }: CellContext<ScreenerResult, number>) =>
          row.original.market_cap_formatted,
      },
      {
        accessorKey: "sector",
        header: ({ column }: HeaderContext<ScreenerResult, string | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs"
          >
            Sector
            <ArrowUpDown className="ml-2 h-3 w-3" />
          </Button>
        ),
      },

      {
        accessorKey: "momentum_1m",
        header: ({ column }: HeaderContext<ScreenerResult, number | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs w-full justify-end"
          >
            Mom 1M
            <ArrowUpDown className="ml-1 h-3 w-3" />
          </Button>
        ),
      },
      {
        accessorKey: "momentum_3m",
        header: ({ column }: HeaderContext<ScreenerResult, number | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs w-full justify-end"
          >
            Mom 3M
            <ArrowUpDown className="ml-1 h-3 w-3" />
          </Button>
        ),
      },
      {
        accessorKey: "momentum_6m",
        header: ({ column }: HeaderContext<ScreenerResult, number | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs w-full justify-end"
          >
            Mom 6M
            <ArrowUpDown className="ml-1 h-3 w-3" />
          </Button>
        ),
      },
      {
        accessorKey: "revenue_yoy_growth",
        header: ({ column }: HeaderContext<ScreenerResult, number | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs w-full justify-end"
          >
            YoY Rev
            <ArrowUpDown className="ml-1 h-3 w-3" />
          </Button>
        ),
      },
      {
        accessorKey: "operating_margin",
        header: ({ column }: HeaderContext<ScreenerResult, number | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs w-full justify-end"
          >
            Op Margin
            <ArrowUpDown className="ml-1 h-3 w-3" />
          </Button>
        ),
      },
      {
        accessorKey: "verdict",
        header: ({ column }: HeaderContext<ScreenerResult, string | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs"
          >
            Verdict
            <ArrowUpDown className="ml-2 h-3 w-3" />
          </Button>
        ),
      },
      {
        accessorKey: "last_analyzed",
        header: ({ column }: HeaderContext<ScreenerResult, string | null>) => (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
            className="hover:bg-transparent p-0 font-semibold text-xs"
          >
            Last Analyzed
            <ArrowUpDown className="ml-2 h-3 w-3" />
          </Button>
        ),
      },
      {
        accessorKey: "guidance_summary",
        header: "Guidance",
      },
    ],
    [],
  );

  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data: results,
    columns,
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onColumnVisibilityChange: setColumnVisibility,
    state: {
      sorting,
      columnVisibility,
    },
  });

  const handleExportCSV = () => {
    if (results.length === 0) return;

    const headers = columns
      .map((c) => c.header as string)
      .filter((h) => typeof h === "string");
    const csvContent = [
      headers.join(","),
      ...results.map((row) =>
        [
          row.symbol,
          `"${row.company_name}"`,
          row.exchange,
          row.market_cap,
          `"${row.sector || ""}"`,
          row.momentum_1m || "",
          row.momentum_3m || "",
          row.momentum_6m || "",
          row.revenue_yoy_growth || "",
          row.operating_margin || "",
          row.verdict || "",
          row.last_analyzed || "",
          `"${row.guidance_summary || ""}"`,
        ].join(","),
      ),
    ].join("\n");

    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    const url = URL.createObjectURL(blob);
    link.setAttribute("href", url);
    link.setAttribute(
      "download",
      `screener_results_${new Date().toISOString().split("T")[0]}.csv`,
    );
    link.style.visibility = "hidden";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  if (isLoading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 10 }).map((_, i) => (
          <div key={i} className="flex items-center space-x-4 px-4 py-2">
            <Skeleton className="h-4 w-[100px]" />
            <Skeleton className="h-4 w-[200px]" />
            <Skeleton className="h-4 w-[100px]" />
            <Skeleton className="h-4 w-[100px]" />
            <Skeleton className="h-4 w-[100px]" />
          </div>
        ))}
      </div>
    );
  }

  if (results.length === 0) {
    return (
      <div className="flex h-full min-h-[400px] flex-col items-center justify-center text-center p-8">
        <div className="rounded-full bg-muted p-6 mb-4">
          <Settings2 className="h-10 w-10 text-muted-foreground/50" />
        </div>
        <h3 className="text-lg font-semibold">No results found</h3>
        <p className="text-muted-foreground max-w-sm mt-2">
          Try adjusting your filter criteria to see more results or run the
          screener again.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between py-4 px-6 border-b bg-background sticky top-0 z-20">
        <div className="text-sm font-medium text-muted-foreground">
          Showing {results.length} companies
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleExportCSV}
            className="h-8"
          >
            <Download className="mr-2 h-4 w-4" />
            Export CSV
          </Button>
          {/* Column toggle would go here if DropdownMenu was available */}
        </div>
      </div>
      <div className="flex-1 overflow-auto">
        <Table>
          <TableHeader className="bg-muted/30 sticky top-0 z-10">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id} className="hover:bg-transparent">
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    className="font-semibold text-xs whitespace-nowrap"
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows.map((row) => (
              <ResultRow key={row.original.company_id} result={row.original} />
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
