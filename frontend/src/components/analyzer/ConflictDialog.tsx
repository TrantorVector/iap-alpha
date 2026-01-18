import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { VerdictResponse, VerdictUpdateRequest } from "@/api/types";
import { Badge } from "@/components/ui/badge";

// Helper to map response to request format
const serverToRequest = (server: VerdictResponse): VerdictUpdateRequest => ({
  lock_version: server.lock_version,
  final_verdict: server.final_verdict,
  summary_text: server.summary_text,
  strengths: server.strengths,
  weaknesses: server.weaknesses,
  guidance_summary: server.guidance_summary,
  linked_report_ids: server.linked_reports.map((r) => r.report_id),
});

interface ConflictDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  yourVersion: VerdictUpdateRequest;
  serverVersion: VerdictResponse;
  onResolve: (resolution: VerdictUpdateRequest) => void;
}

export function ConflictDialog({
  open,
  onOpenChange,
  yourVersion,
  serverVersion,
  onResolve,
}: ConflictDialogProps) {
  const handleKeepMine = () => {
    // We keep our data, but must adopt the server's lock_version to win the next race
    // (Actually, usually we just send our data again with the NEW lock_version we saw from server?
    // Wait, if server has v2, and we sent v1. We want to overwrite v2->v3.
    // So we should send current v2 as the base.)
    onResolve({
      ...yourVersion,
      lock_version: serverVersion.lock_version,
    });
    onOpenChange(false);
  };

  const handleUseServer = () => {
    // We adopt server data completely
    onResolve(serverToRequest(serverVersion));
    onOpenChange(false);
  };

  const handleMerge = () => {
    // Basic merge: Concatenate texts, combine lists unique
    const merged: VerdictUpdateRequest = {
      lock_version: serverVersion.lock_version,
      final_verdict: yourVersion.final_verdict || serverVersion.final_verdict,
      summary_text: [
        yourVersion.summary_text,
        serverVersion.summary_text !== yourVersion.summary_text
          ? `\n[Server Version]:\n${serverVersion.summary_text}`
          : null,
      ]
        .filter(Boolean)
        .join("\n\n"),
      strengths: Array.from(
        new Set([...yourVersion.strengths, ...serverVersion.strengths]),
      ),
      weaknesses: Array.from(
        new Set([...yourVersion.weaknesses, ...serverVersion.weaknesses]),
      ),
      guidance_summary: [
        yourVersion.guidance_summary,
        serverVersion.guidance_summary !== yourVersion.guidance_summary
          ? `\n[Server Version]:\n${serverVersion.guidance_summary}`
          : null,
      ]
        .filter(Boolean)
        .join("\n\n"),
      linked_report_ids: Array.from(
        new Set([
          ...yourVersion.linked_report_ids,
          ...serverVersion.linked_reports.map((r) => r.report_id),
        ]),
      ),
    };
    onResolve(merged);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="text-destructive flex items-center gap-2">
            <span>⚠️</span> Update Conflict
          </DialogTitle>
          <DialogDescription>
            Another user has updated this verdict since you loaded it. Please
            choose how you want to resolve this conflict.
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto grid grid-cols-2 gap-4 border rounded-md p-4 min-h-0">
          <div className="space-y-4">
            <h3 className="font-semibold text-lg border-b pb-2 mb-2">
              Your Version
            </h3>

            <div className="space-y-1">
              <span className="text-xs font-medium text-muted-foreground uppercase">
                Verdict
              </span>
              <div>
                {yourVersion.final_verdict ? (
                  <Badge variant="outline">{yourVersion.final_verdict}</Badge>
                ) : (
                  <span className="text-muted-foreground italic">None</span>
                )}
              </div>
            </div>

            <div className="space-y-1">
              <span className="text-xs font-medium text-muted-foreground uppercase">
                Summary
              </span>
              <div className="text-sm bg-muted/30 p-2 rounded whitespace-pre-wrap">
                {yourVersion.summary_text || (
                  <span className="italic text-muted-foreground">Empty</span>
                )}
              </div>
            </div>

            <div className="space-y-1">
              <span className="text-xs font-medium text-muted-foreground uppercase">
                Strengths
              </span>
              <ul className="list-disc list-inside text-sm">
                {yourVersion.strengths.length ? (
                  yourVersion.strengths.map((s, i) => <li key={i}>{s}</li>)
                ) : (
                  <li className="list-none text-muted-foreground italic">
                    None
                  </li>
                )}
              </ul>
            </div>
          </div>

          <div className="space-y-4 bg-slate-50 dark:bg-slate-900/50 -m-4 p-4 border-l">
            <h3 className="font-semibold text-lg border-b pb-2 mb-2">
              Server Version
            </h3>

            <div className="space-y-1">
              <span className="text-xs font-medium text-muted-foreground uppercase">
                Verdict
              </span>
              <div>
                {serverVersion.final_verdict ? (
                  <Badge variant="outline">{serverVersion.final_verdict}</Badge>
                ) : (
                  <span className="text-muted-foreground italic">None</span>
                )}
              </div>
            </div>

            <div className="space-y-1">
              <span className="text-xs font-medium text-muted-foreground uppercase">
                Summary
              </span>
              <div className="text-sm bg-background p-2 rounded border whitespace-pre-wrap">
                {serverVersion.summary_text || (
                  <span className="italic text-muted-foreground">Empty</span>
                )}
              </div>
            </div>

            <div className="space-y-1">
              <span className="text-xs font-medium text-muted-foreground uppercase">
                Strengths
              </span>
              <ul className="list-disc list-inside text-sm">
                {serverVersion.strengths.length ? (
                  serverVersion.strengths.map((s, i) => <li key={i}>{s}</li>)
                ) : (
                  <li className="list-none text-muted-foreground italic">
                    None
                  </li>
                )}
              </ul>
            </div>
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-0 mt-4">
          <Button variant="secondary" onClick={handleUseServer}>
            Discard Mine (Use Server's)
          </Button>
          <div className="flex-1" />
          <Button variant="outline" onClick={handleMerge}>
            Merge Both
          </Button>
          <Button onClick={handleKeepMine}>Overwrite Server (Keep Mine)</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
