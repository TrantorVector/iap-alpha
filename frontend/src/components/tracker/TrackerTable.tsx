import {
  useReactTable,
  getCoreRowModel,
  flexRender,
  createColumnHelper,
  PaginationState,
} from "@tanstack/react-table";
import { TrackerItemOut } from "@/api/types";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { format } from "date-fns";

interface TrackerTableProps {
  data: TrackerItemOut[];
  total: number;
  pagination: PaginationState;
  onPaginationChange: (pagination: PaginationState) => void;
  isLoading?: boolean;
}

const columnHelper = createColumnHelper<TrackerItemOut>();

export function TrackerTable({
  data,
  total,
  pagination,
  onPaginationChange,
  isLoading,
}: TrackerTableProps) {
  const navigate = useNavigate();

  const columns = [
    columnHelper.accessor("symbol", {
      header: "Symbol",
      cell: (info) => (
        <span className="font-bold cursor-pointer text-primary">
          {info.getValue()}
        </span>
      ),
    }),
    columnHelper.accessor("company_name", {
      header: "Company",
      cell: (info) => (
        <span
          className="font-medium truncate max-w-[200px] block"
          title={info.getValue()}
        >
          {info.getValue()}
        </span>
      ),
    }),
    columnHelper.accessor("exchange", {
      header: "Exchange",
      cell: (info) => <Badge variant="outline">{info.getValue()}</Badge>,
    }),
    columnHelper.accessor("sector", {
      header: "Sector",
      cell: (info) => info.getValue() || "-",
    }),
    columnHelper.accessor("verdict", {
      header: "Verdict",
      cell: (info) => {
        const v = info.getValue();
        let colorClass = "bg-slate-500";
        if (v === "Invest")
          colorClass =
            "bg-green-500 hover:bg-green-600 border-green-200 text-white";
        if (v === "Watchlist")
          colorClass =
            "bg-yellow-500 hover:bg-yellow-600 border-yellow-200 text-white";
        if (v === "Pass")
          colorClass =
            "bg-gray-500 hover:bg-gray-600 border-gray-200 text-white";
        if (v === "No Thesis")
          colorClass = "bg-red-500 hover:bg-red-600 border-red-200 text-white";

        return (
          <Badge className={`${colorClass} font-normal`}>
            {v || "Pending"}
          </Badge>
        );
      },
    }),
    columnHelper.accessor("verdict_date", {
      header: "Date",
      cell: (info) => {
        try {
          return format(new Date(info.getValue()), "MMM d, yyyy");
        } catch {
          return info.getValue();
        }
      },
    }),
    columnHelper.accessor("summary_text", {
      header: "Summary",
      cell: (info) => (
        <div
          className="max-w-md truncate text-muted-foreground italic text-xs"
          title={info.getValue()}
        >
          {info.getValue()}
        </div>
      ),
    }),
  ];

  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    pageCount: Math.ceil(total / pagination.pageSize),
    state: {
      pagination,
    },
    onPaginationChange: (updater) => {
      if (typeof updater === "function") {
        onPaginationChange(updater(pagination));
      } else {
        onPaginationChange(updater);
      }
    },
  });

  // Safe page count
  const pageCount = table.getPageCount();

  return (
    <div className="space-y-4">
      <div className="rounded-md border bg-card">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead key={header.id}>
                    {flexRender(
                      header.column.columnDef.header,
                      header.getContext(),
                    )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  Loading...
                </TableCell>
              </TableRow>
            ) : data.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No results found.
                </TableCell>
              </TableRow>
            ) : (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  className="cursor-pointer hover:bg-muted/50"
                  onClick={() =>
                    navigate(`/analyzer/${row.original.company_id}`)
                  }
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
      <div className="flex items-center justify-end space-x-2 py-4">
        <Button
          variant="outline"
          size="sm"
          onClick={() => table.previousPage()}
          disabled={!table.getCanPreviousPage()}
        >
          <ChevronLeft className="h-4 w-4" />
        </Button>
        <span className="text-sm text-muted-foreground">
          Page {pagination.pageIndex + 1} of {pageCount > 0 ? pageCount : 1}
        </span>
        <Button
          variant="outline"
          size="sm"
          onClick={() => table.nextPage()}
          disabled={!table.getCanNextPage()}
        >
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
