import { Document } from "../../api/types";
import { Loader2, Upload, AlertCircle, FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useRef } from "react";

interface DocumentCellProps {
  document?: Document;
  isLoading?: boolean;
  type: string;
  isUploadable?: boolean;
  onUpload?: (file: File) => void;
  onClick?: (doc: Document) => void;
}

export function DocumentCell({
  document,
  isLoading,
  type,
  isUploadable,
  onUpload,
  onClick,
}: DocumentCellProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleUploadClick = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file && onUpload) {
      onUpload(file);
    }
    // Reset value so same file can be selected again if needed
    if (e.target.value) {
      e.target.value = "";
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-2 h-full min-h-[40px]">
        <Loader2 className="h-4 w-4 animate-spin text-gray-400" />
      </div>
    );
  }

  if (document) {
    if (!document.available) {
      return (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center justify-center p-2 h-full min-h-[40px] cursor-not-allowed">
                <AlertCircle className="h-4 w-4 text-gray-300" />
              </div>
            </TooltipTrigger>
            <TooltipContent>
              <p>Document unavailable</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      );
    }

    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className={cn(
                "w-full h-full min-h-[40px] rounded-none hover:bg-blue-50 transition-colors",
                "text-blue-600 flex items-center justify-center gap-2",
              )}
              onClick={() => onClick?.(document)}
            >
              <FileText className="h-4 w-4" />
              <span className="sr-only">Download {type}</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>Download {document.title || type}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  // No document exists
  if (isUploadable) {
    return (
      <div className="flex items-center justify-center p-2 h-full min-h-[40px]">
        <input
          type="file"
          ref={fileInputRef}
          className="hidden"
          accept=".pdf,.doc,.docx,.txt"
          onChange={handleFileChange}
        />
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                className="w-full h-full hover:bg-gray-100 text-gray-400 hover:text-gray-600"
                onClick={handleUploadClick}
              >
                <Upload className="h-4 w-4" />
                <span className="sr-only">Upload {type}</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>Upload {type}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>
    );
  }

  // Empty state for non-uploadable cells
  return (
    <div className="flex items-center justify-center p-2 h-full min-h-[40px]">
      <div className="w-2 h-0.5 bg-gray-200 rounded" />
    </div>
  );
}
