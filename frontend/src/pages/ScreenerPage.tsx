import { useState, useEffect } from "react";
import { ScreenerList } from "@/components/screener/ScreenerList";
import { ScreenerEditor } from "@/components/screener/ScreenerEditor";
import { ResultsTable } from "@/components/screener/ResultsTable";
import { Screener, ScreenerResult, CreateScreener } from "@/api/types";
import { screeners as screenersApi } from "@/api/endpoints";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Loader2, Play } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

export default function ScreenerPage() {
  const [screeners, setScreeners] = useState<Screener[]>([]);
  const [selectedScreener, setSelectedScreener] = useState<Screener | null>(
    null,
  );
  const [results, setResults] = useState<ScreenerResult[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [isLoadingList, setIsLoadingList] = useState(false);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [editorMode, setEditorMode] = useState<"create" | "edit">("create");
  const { toast } = useToast();

  useEffect(() => {
    fetchScreeners();
  }, []);

  const fetchScreeners = async () => {
    setIsLoadingList(true);
    try {
      const data = await screenersApi.list();
      setScreeners(data);
      if (data && data.length > 0 && !selectedScreener) {
        setSelectedScreener(data[0]);
      }
    } catch (error) {
      console.error("Failed to fetch screeners", error);
    } finally {
      setIsLoadingList(false);
    }
  };

  const handleRunScreener = async (id: string) => {
    setIsRunning(true);
    const screenerToRun = screeners.find((s) => s.id === id);
    if (screenerToRun) {
      setSelectedScreener(screenerToRun);
    }

    try {
      const data = await screenersApi.run(id);
      setResults(data.results);
      toast({
        title: "Success",
        description: `Screener executed successfully. Found ${data.total_results} results.`,
      });
      fetchScreeners();
    } catch (error) {
      console.error("Failed to run screener", error);
      toast({
        title: "Error",
        description: "Failed to execute screener",
        variant: "destructive",
      });
    } finally {
      setIsRunning(false);
    }
  };

  const handleDeleteScreener = async (id: string) => {
    try {
      await screenersApi.delete(id);
      setScreeners((prev) => prev.filter((s) => s.id !== id));
      if (selectedScreener?.id === id) {
        const remaining = screeners.filter((s) => s.id !== id);
        setSelectedScreener(remaining.length > 0 ? remaining[0] : null);
        setResults([]);
      }
      toast({
        title: "Success",
        description: "Screener deleted",
      });
    } catch (error) {
      console.error("Failed to delete screener", error);
      toast({
        title: "Error",
        description: "Failed to delete screener",
        variant: "destructive",
      });
    }
  };

  const handleCreateScreener = () => {
    setEditorMode("create");
    setIsEditorOpen(true);
  };

  const handleEditScreener = (screener: Screener) => {
    setSelectedScreener(screener);
    setEditorMode("edit");
    setIsEditorOpen(true);
  };

  const handleSaveScreener = async (data: CreateScreener) => {
    try {
      if (editorMode === "create") {
        const newScreener = await screenersApi.create(data);
        setScreeners((prev) => [...prev, newScreener]);
        setSelectedScreener(newScreener);
        toast({ title: "Success", description: "Screener created" });
      } else if (selectedScreener) {
        const updatedScreener = await screenersApi.update(
          selectedScreener.id,
          data,
        );
        setScreeners((prev) =>
          prev.map((s) => (s.id === updatedScreener.id ? updatedScreener : s)),
        );
        setSelectedScreener(updatedScreener);
        toast({ title: "Success", description: "Screener updated" });
      }
      setIsEditorOpen(false);
    } catch (error) {
      console.error("Failed to save screener", error);
      toast({
        title: "Error",
        description: "Failed to save screener",
        variant: "destructive",
      });
    }
  };

  return (
    <div className="flex h-full w-full pt-4">
      <div className="w-[30%] h-full min-w-[300px] max-w-[400px] flex flex-col">
        <ScreenerList
          screeners={screeners}
          selectedId={selectedScreener?.id || null}
          onSelect={(s) => {
            if (selectedScreener?.id !== s.id) {
              setSelectedScreener(s);
              setResults([]);
            }
          }}
          onCreate={handleCreateScreener}
          onRun={handleRunScreener}
          onEdit={handleEditScreener}
          onDelete={handleDeleteScreener}
        />
      </div>

      <div className="flex-1 h-full overflow-hidden flex flex-col px-6 pb-6">
        {isLoadingList ? (
          <div className="flex h-full items-center justify-center">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
          </div>
        ) : selectedScreener ? (
          <div className="flex flex-col h-full gap-4 animate-in fade-in duration-500">
            <div className="flex justify-between items-start">
              <div className="max-w-[70%]">
                <h1
                  className="text-3xl font-bold tracking-tight truncate"
                  title={selectedScreener.title}
                >
                  {selectedScreener.title}
                </h1>
                <p className="text-muted-foreground mt-1 text-sm line-clamp-2">
                  {selectedScreener.description}
                </p>
              </div>
              <Button
                onClick={() => handleRunScreener(selectedScreener.id)}
                disabled={isRunning}
              >
                {isRunning ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Play className="mr-2 h-4 w-4" />
                )}
                Run Screener
              </Button>
            </div>

            <Card className="flex-1 overflow-hidden flex flex-col border-muted">
              <CardHeader className="py-2 px-4 border-b bg-muted/20 flex flex-row items-center justify-between">
                <CardTitle className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                  Results {results.length > 0 && `(${results.length})`}
                </CardTitle>
              </CardHeader>
              <CardContent className="flex-1 overflow-auto p-0">
                <ResultsTable results={results} isLoading={isRunning} />
              </CardContent>
            </Card>
          </div>
        ) : (
          <div className="flex h-full items-center justify-center text-muted-foreground bg-muted/5 rounded-lg border border-dashed m-4">
            <div className="text-center space-y-2">
              <p>Select a screener to view details</p>
              <Button
                variant="outline"
                onClick={handleCreateScreener}
                className="gap-2"
              >
                Or create a new one
              </Button>
            </div>
          </div>
        )}
      </div>

      <ScreenerEditor
        open={isEditorOpen}
        mode={editorMode}
        initialData={selectedScreener}
        onSave={handleSaveScreener}
        onClose={() => setIsEditorOpen(false)}
      />
    </div>
  );
}
