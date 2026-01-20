import { useState, useEffect, forwardRef, useImperativeHandle } from "react";
import { useForm, Controller } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label"; // Using available Label component
import { VerdictResponse, VerdictUpdateRequest } from "@/api/types";
import { verdicts, companies } from "@/api/endpoints";
import { Loader2, Plus, X, Upload, FileText } from "lucide-react";
import { ConflictDialog } from "./ConflictDialog";
import { useToast } from "@/hooks/use-toast";
import { ApiError } from "@/api/client";

const verdictSchema = z.object({
  final_verdict: z.enum(["INVEST", "PASS", "WATCHLIST", "NO_THESIS"]),
  summary_text: z
    .string()
    .max(500, "Max 500 characters")
    .min(10, "Min 10 characters"),
  strengths: z.array(z.string()).min(1, "Add at least one strength"),
  weaknesses: z.array(z.string()).min(1, "Add at least one weakness"),
  guidance_summary: z.string().optional(),
  linked_report_ids: z.array(z.string()),
});

type VerdictFormValues = z.infer<typeof verdictSchema>;

interface VerdictFormProps {
  companyId: string;
  initialData: VerdictResponse | null;
  onSaved: () => void;
  onDirtyChange?: (isDirty: boolean) => void;
}

export interface VerdictFormHandle {
  submit: () => void;
}

export const VerdictForm = forwardRef<VerdictFormHandle, VerdictFormProps>(
  ({ companyId, initialData, onSaved, onDirtyChange }, ref) => {
    const { toast } = useToast();
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [isUploading, setIsUploading] = useState(false);
    const [conflictStart, setConflictStart] = useState<{
      your: VerdictUpdateRequest;
      server: VerdictResponse;
    } | null>(null);

    // Initial default values
    const defaultValues: VerdictFormValues = {
      final_verdict:
        (initialData?.final_verdict as VerdictFormValues["final_verdict"]) ||
        undefined,
      summary_text: initialData?.summary_text || "",
      strengths: initialData?.strengths?.length ? initialData.strengths : [""],
      weaknesses: initialData?.weaknesses?.length
        ? initialData.weaknesses
        : [""],
      guidance_summary: initialData?.guidance_summary || "",
      linked_report_ids:
        initialData?.linked_reports?.map((r) => r.report_id) || [],
    };

    const [lockVersion, setLockVersion] = useState(
      initialData?.lock_version || 0,
    );
    const [linkedReports, setLinkedReports] = useState<
      { id: string; name: string }[]
    >(
      initialData?.linked_reports?.map((r) => ({
        id: r.report_id,
        name: r.filename,
      })) || [],
    );

    const {
      control,
      register,
      handleSubmit,
      watch,
      setValue,
      reset,
      getValues,
      formState: { errors, isDirty },
    } = useForm<VerdictFormValues>({
      resolver: zodResolver(verdictSchema),
      defaultValues,
    });

    // Notify parent of dirty state
    useEffect(() => {
      onDirtyChange?.(isDirty);
    }, [isDirty, onDirtyChange]);

    const strengths = watch("strengths");
    const weaknesses = watch("weaknesses");

    const addStrength = () => setValue("strengths", [...strengths, ""]);
    const removeStrength = (idx: number) =>
      setValue(
        "strengths",
        strengths.filter((_, i) => i !== idx),
      );
    const updateStrength = (idx: number, val: string) => {
      const newArr = [...strengths];
      newArr[idx] = val;
      setValue("strengths", newArr);
    };

    const addWeakness = () => setValue("weaknesses", [...weaknesses, ""]);
    const removeWeakness = (idx: number) =>
      setValue(
        "weaknesses",
        weaknesses.filter((_, i) => i !== idx),
      );
    const updateWeakness = (idx: number, val: string) => {
      const newArr = [...weaknesses];
      newArr[idx] = val;
      setValue("weaknesses", newArr);
    };

    const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
      if (!e.target.files?.length) return;
      setIsUploading(true);
      try {
        const file = e.target.files[0];
        const doc = await companies.uploadDocument(companyId, file, {
          document_type: "analyst_report",
          period_end_date: new Date().toISOString().split("T")[0],
        });

        const currentIds = getValues("linked_report_ids");
        setValue("linked_report_ids", [...currentIds, doc.id]);
        setLinkedReports((prev) => [
          ...prev,
          { id: doc.id, name: doc.title || file.name },
        ]);
        toast({ title: "Report uploaded" });
      } catch (err) {
        console.error(err);
        toast({
          title: "Upload failed",
          variant: "destructive",
          description: (err as Error).message || "Could not upload document",
        });
      } finally {
        setIsUploading(false);
        e.target.value = "";
      }
    };

    const onSubmit = async (data: VerdictFormValues) => {
      setIsSubmitting(true);
      try {
        const updateReq: VerdictUpdateRequest = {
          ...data,
          lock_version: lockVersion,
          strengths: data.strengths.filter((s) => s.trim()),
          weaknesses: data.weaknesses.filter((s) => s.trim()),
          guidance_summary: data.guidance_summary || null, // Handle optional string to null
          final_verdict: data.final_verdict, // Enum matches
        };

        const response = await verdicts.update(companyId, updateReq);
        setLockVersion(response.lock_version);

        // Allow the form to believe these are the new clean values
        reset(data);

        toast({ title: "Verdict saved successfully" });
        onSaved();
      } catch (error) {
        if (error instanceof ApiError && error.status === 409) {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const details = (error.data as any)?.error?.details;
          if (details && details.current_state) {
            setConflictStart({
              your: {
                ...data,
                lock_version: lockVersion,
                strengths: data.strengths.filter((s) => s.trim()),
                weaknesses: data.weaknesses.filter((s) => s.trim()),
                guidance_summary: data.guidance_summary || null,
                final_verdict: data.final_verdict,
              },
              server: details.current_state as VerdictResponse,
            });
          }
        } else {
          toast({
            title: "Error saving verdict",
            description: (error as Error).message || "Unknown error",
            variant: "destructive",
          });
        }
      } finally {
        setIsSubmitting(false);
      }
    };

    // Expose submit method to parent
    useImperativeHandle(ref, () => ({
      submit: () => {
        handleSubmit(onSubmit)();
      },
    }));

    const handleConflictResolve = (resolution: VerdictUpdateRequest) => {
      reset({
        final_verdict:
          (resolution.final_verdict as VerdictFormValues["final_verdict"]) ||
          undefined,
        summary_text: resolution.summary_text || "",
        strengths: resolution.strengths,
        weaknesses: resolution.weaknesses,
        guidance_summary: resolution.guidance_summary || "",
        linked_report_ids: resolution.linked_report_ids,
      });

      setLockVersion(resolution.lock_version);
      toast({
        title: "Conflict resolved",
        description: "Please review and save again.",
      });
    };

    return (
      <div className="h-full flex flex-col">
        <form
          onSubmit={handleSubmit(onSubmit)}
          className="space-y-6 flex-1 flex flex-col"
        >
          <div className="flex-1 overflow-y-auto pr-2 space-y-6">
            {/* Verdict Selection */}
            <div className="space-y-3">
              <Label>Final Verdict</Label>
              <Controller
                control={control}
                name="final_verdict"
                render={({ field }) => (
                  <div className="flex flex-col sm:flex-row gap-4">
                    {["INVEST", "WATCHLIST", "PASS", "NO_THESIS"].map((opt) => (
                      <label
                        key={opt}
                        className="flex items-center space-x-2 cursor-pointer"
                      >
                        <input
                          type="radio"
                          value={opt}
                          checked={field.value === opt}
                          onChange={field.onChange}
                          className="accent-indigo-600 h-4 w-4"
                        />
                        <span className="text-sm font-medium">
                          {opt.replace("_", " ")}
                        </span>
                      </label>
                    ))}
                  </div>
                )}
              />
              {errors.final_verdict && (
                <p className="text-sm font-medium text-destructive">
                  {errors.final_verdict.message}
                </p>
              )}
            </div>

            {/* Summary Text */}
            <div className="space-y-3">
              <Label htmlFor="summary_text">
                Investment Summary (50-100 words)
              </Label>
              <Textarea
                id="summary_text"
                placeholder="Write your executive summary here..."
                className="min-h-[120px]"
                {...register("summary_text")}
              />
              {errors.summary_text && (
                <p className="text-sm font-medium text-destructive">
                  {errors.summary_text.message}
                </p>
              )}
            </div>

            {/* Strengths & Weaknesses Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <Label>Strengths</Label>
                <div className="space-y-2">
                  {strengths.map((_, index) => (
                    <div key={index} className="flex gap-2">
                      <Input
                        value={strengths[index]}
                        onChange={(e) => updateStrength(index, e.target.value)}
                        placeholder="Strength point..."
                        aria-label={`Strength ${index + 1}`}
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        onClick={() => removeStrength(index)}
                        disabled={strengths.length === 1 && index === 0}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={addStrength}
                    className="w-full"
                  >
                    <Plus className="h-3 w-3 mr-2" /> Add Strength
                  </Button>
                </div>
                {errors.strengths && (
                  <p className="text-sm font-medium text-destructive">
                    {errors.strengths.message}
                  </p>
                )}
              </div>

              <div className="space-y-2">
                <Label>Weaknesses</Label>
                <div className="space-y-2">
                  {weaknesses.map((_, index) => (
                    <div key={index} className="flex gap-2">
                      <Input
                        value={weaknesses[index]}
                        onChange={(e) => updateWeakness(index, e.target.value)}
                        placeholder="Weakness point..."
                        aria-label={`Weakness ${index + 1}`}
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        onClick={() => removeWeakness(index)}
                        disabled={weaknesses.length === 1 && index === 0}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={addWeakness}
                    className="w-full"
                  >
                    <Plus className="h-3 w-3 mr-2" /> Add Weakness
                  </Button>
                </div>
                {errors.weaknesses && (
                  <p className="text-sm font-medium text-destructive">
                    {errors.weaknesses.message}
                  </p>
                )}
              </div>
            </div>

            <div className="space-y-3">
              <Label htmlFor="guidance_summary">
                Management Guidance (Optional)
              </Label>
              <Textarea
                id="guidance_summary"
                placeholder="Summary of management guidance..."
                className="min-h-[80px]"
                {...register("guidance_summary")}
              />
              {errors.guidance_summary && (
                <p className="text-sm font-medium text-destructive">
                  {errors.guidance_summary.message}
                </p>
              )}
            </div>

            <div className="space-y-3">
              <Label>Linked Documents</Label>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
                {linkedReports.map((report) => (
                  <div
                    key={report.id}
                    className="flex items-center p-2 border rounded bg-slate-50 dark:bg-slate-900 text-sm"
                  >
                    <FileText className="h-4 w-4 mr-2 opacity-50" />
                    <span className="truncate flex-1">{report.name}</span>
                  </div>
                ))}
                <div className="relative border border-dashed rounded flex flex-col items-center justify-center p-4 hover:bg-slate-50 dark:hover:bg-slate-900/50 transition-colors cursor-pointer group">
                  {isUploading ? (
                    <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                  ) : (
                    <>
                      <Upload className="h-5 w-5 mb-1 text-muted-foreground group-hover:text-foreground" />
                      <span className="text-xs text-muted-foreground">
                        Attach Report
                      </span>
                    </>
                  )}
                  <input
                    type="file"
                    className="absolute inset-0 opacity-0 cursor-pointer"
                    onChange={handleFileUpload}
                    disabled={isUploading}
                    accept=".pdf,.doc,.docx"
                  />
                </div>
              </div>
            </div>
          </div>

          <div className="pt-4 border-t flex justify-end gap-2">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              Save Verdict
            </Button>
          </div>
        </form>

        {conflictStart && (
          <ConflictDialog
            open={!!conflictStart}
            onOpenChange={(v) => !v && setConflictStart(null)}
            yourVersion={conflictStart.your}
            serverVersion={conflictStart.server}
            onResolve={handleConflictResolve}
          />
        )}
      </div>
    );
  },
);
