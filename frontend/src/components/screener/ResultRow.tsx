import { useNavigate } from "react-router-dom";
import { ScreenerResult } from "@/api/types";
import { TableRow, TableCell } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { formatDistanceToNow } from "date-fns";
import { cn } from "@/lib/utils";

interface ResultRowProps {
  result: ScreenerResult;
}

export function ResultRow({ result }: ResultRowProps) {
  const navigate = useNavigate();

  const formatPercentage = (val: number | null) => {
    if (val === null) return "-";
    const formatted = (val * 100).toFixed(1) + "%";
    return val > 0 ? `+${formatted}` : formatted;
  };

  const getMomentumColor = (val: number | null) => {
    if (val === null) return "text-muted-foreground";
    if (val > 0.1) return "text-green-500 font-medium";
    if (val > 0) return "text-green-400";
    if (val < -0.1) return "text-red-500 font-medium";
    if (val < 0) return "text-red-400";
    return "text-muted-foreground";
  };

  const getVerdictBadge = (verdict: string | null) => {
    if (!verdict) return null;

    const variants: Record<string, string> = {
      INVEST: "bg-green-500/10 text-green-500 border-green-500/20",
      PASS: "bg-yellow-500/10 text-yellow-500 border-yellow-500/20",
      AVOID: "bg-red-500/10 text-red-500 border-red-500/20",
      WAIT: "bg-blue-500/10 text-blue-500 border-blue-500/20",
    };

    return (
      <Badge
        variant="outline"
        className={cn(
          "font-semibold text-[10px] px-1.5 py-0 h-5",
          variants[verdict] || "bg-muted text-muted-foreground",
        )}
      >
        {verdict}
      </Badge>
    );
  };

  const handleRowClick = () => {
    navigate(`/analyzer/${result.company_id}`);
  };

  return (
    <TableRow
      className="hover:bg-muted/50 cursor-pointer transition-colors group"
      onClick={handleRowClick}
    >
      <TableCell className="font-bold text-primary group-hover:underline">
        {result.symbol}
      </TableCell>
      <TableCell className="max-w-[150px] truncate" title={result.company_name}>
        {result.company_name}
      </TableCell>
      <TableCell className="text-xs text-muted-foreground">
        {result.exchange}
      </TableCell>
      <TableCell className="text-sm">{result.market_cap_formatted}</TableCell>
      <TableCell
        className="text-xs truncate max-w-[100px]"
        title={result.sector || ""}
      >
        {result.sector || "-"}
      </TableCell>
      <TableCell
        className={cn(
          "text-right text-xs",
          getMomentumColor(result.momentum_1m),
        )}
      >
        {formatPercentage(result.momentum_1m)}
      </TableCell>
      <TableCell
        className={cn(
          "text-right text-xs",
          getMomentumColor(result.momentum_3m),
        )}
      >
        {formatPercentage(result.momentum_3m)}
      </TableCell>
      <TableCell
        className={cn(
          "text-right text-xs",
          getMomentumColor(result.momentum_6m),
        )}
      >
        {formatPercentage(result.momentum_6m)}
      </TableCell>
      <TableCell className="text-right text-xs">
        {formatPercentage(result.revenue_yoy_growth)}
      </TableCell>
      <TableCell className="text-right text-xs">
        {formatPercentage(result.operating_margin)}
      </TableCell>
      <TableCell>{getVerdictBadge(result.verdict)}</TableCell>
      <TableCell className="text-xs text-muted-foreground whitespace-nowrap">
        {result.last_analyzed
          ? formatDistanceToNow(new Date(result.last_analyzed), {
              addSuffix: true,
            })
          : "-"}
      </TableCell>
      <TableCell
        className="max-w-[200px] truncate text-xs italic text-muted-foreground"
        title={result.guidance_summary || ""}
      >
        {result.guidance_summary || "-"}
      </TableCell>
    </TableRow>
  );
}
