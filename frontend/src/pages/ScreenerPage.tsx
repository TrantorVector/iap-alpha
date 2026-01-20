import { useState, useEffect } from "react";
import { ScreenerList } from "@/components/screener/ScreenerList";
import { Screener, ScreenerResult } from "@/api/types";
import { client } from "@/api/client";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Loader2, Play } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

export default function ScreenerPage() {
    const [screeners, setScreeners] = useState<Screener[]>([]);
    const [selectedScreener, setSelectedScreener] = useState<Screener | null>(null);
    const [results, setResults] = useState<ScreenerResult[]>([]);
    const [isRunning, setIsRunning] = useState(false);
    const [, setIsLoadingList] = useState(false); // Used for initial loading state
    const { toast } = useToast();

    useEffect(() => {
        fetchScreeners();
    }, []);

    const fetchScreeners = async () => {
        setIsLoadingList(true);
        try {
            // Try to fetch screeners, if endpoint assumes empty array if 404 or empty
            const data = await client.get<Screener[]>("/screener/screeners");
            setScreeners(data);
            if (data && data.length > 0 && !selectedScreener) {
                setSelectedScreener(data[0]);
            }
        } catch (error) {
            console.error("Failed to fetch screeners", error);
            // Suppress error toast for now as backend might be empty or in dev
            // But log it.
        } finally {
            setIsLoadingList(false);
        }
    };

    const handleRunScreener = async (id: string) => {
        setIsRunning(true);
        // Find the screener to ensure we select it if not already
        const screenerToRun = screeners.find(s => s.id === id);
        if (screenerToRun) {
            setSelectedScreener(screenerToRun);
        }

        try {
            const data = await client.post<ScreenerResult[]>(`/screener/screeners/${id}/execute`);
            setResults(data);
            toast({
                title: "Success",
                description: `Screener executed successfully. Found ${data.length} results.`,
            });
            // Refresh screeners to update last_run_at
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
            await client.delete(`/screener/screeners/${id}`);
            setScreeners(prev => prev.filter(s => s.id !== id));
            if (selectedScreener?.id === id) {
                const remaining = screeners.filter(s => s.id !== id);
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
        toast({
            title: "Not Implemented",
            description: "Create screener functionality coming soon",
        });
    };

    const handleEditScreener = (screener: Screener) => {
        toast({
            title: "Not Implemented",
            description: `Edit functionality for ${screener.name} coming soon`,
        });
    };

    return (
        <div className="flex h-full w-full pt-4">
            {/* Left Pane: List - 30% */}
            <div className="w-[30%] h-full min-w-[300px] max-w-[400px] flex flex-col">
                <ScreenerList
                    screeners={screeners}
                    selectedId={selectedScreener?.id || null}
                    onSelect={(s) => {
                        if (selectedScreener?.id !== s.id) {
                            setSelectedScreener(s);
                            setResults([]); // Clear results on new selection
                        }
                    }}
                    onCreate={handleCreateScreener}
                    onRun={handleRunScreener}
                    onEdit={handleEditScreener}
                    onDelete={handleDeleteScreener}
                />
            </div>

            {/* Right Pane: Results - 70% */}
            <div className="flex-1 h-full overflow-hidden flex flex-col px-6 pb-6">
                {selectedScreener ? (
                    <div className="flex flex-col h-full gap-4 animate-in fade-in duration-500">
                        <div className="flex justify-between items-start">
                            <div>
                                <h1 className="text-3xl font-bold tracking-tight">{selectedScreener.name}</h1>
                                <p className="text-muted-foreground mt-1 text-sm">{selectedScreener.description}</p>
                            </div>
                            <Button onClick={() => handleRunScreener(selectedScreener.id)} disabled={isRunning}>
                                {isRunning ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Play className="mr-2 h-4 w-4" />}
                                Run Screener
                            </Button>
                        </div>

                        <Card className="flex-1 overflow-hidden flex flex-col border-muted">
                            <CardHeader className="py-3 px-4 border-b bg-muted/20">
                                <CardTitle className="text-sm font-medium">Results {results.length > 0 && `(${results.length})`}</CardTitle>
                            </CardHeader>
                            <CardContent className="flex-1 overflow-auto p-0">
                                {isRunning ? (
                                    <div className="flex h-full items-center justify-center flex-col gap-2">
                                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                                        <p className="text-sm text-muted-foreground">Running screener...</p>
                                    </div>
                                ) : results.length > 0 ? (
                                    <Table>
                                        <TableHeader className="bg-muted/10 sticky top-0 z-10">
                                            <TableRow>
                                                <TableHead>Symbol</TableHead>
                                                <TableHead>Name</TableHead>
                                                <TableHead>Market Cap</TableHead>
                                                <TableHead>Sector</TableHead>
                                                <TableHead>Exchange</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {results.map((result) => (
                                                <TableRow key={result.company_id} className="hover:bg-muted/10">
                                                    <TableCell className="font-medium">{result.symbol}</TableCell>
                                                    <TableCell className="max-w-[200px] truncate" title={result.name}>{result.name}</TableCell>
                                                    <TableCell>{result.market_cap_formatted}</TableCell>
                                                    <TableCell>{result.sector || "-"}</TableCell>
                                                    <TableCell>{result.exchange}</TableCell>
                                                </TableRow>
                                            ))}
                                        </TableBody>
                                    </Table>
                                ) : (
                                    <div className="flex h-full items-center justify-center text-muted-foreground flex-col gap-2">
                                        <div className="p-4 bg-muted/20 rounded-full">
                                            <Play className="h-6 w-6 text-muted-foreground/50" />
                                        </div>
                                        <p>Run the screener to see matches.</p>
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </div>
                ) : (
                    <div className="flex h-full items-center justify-center text-muted-foreground bg-muted/5 rounded-lg border border-dashed m-4">
                        <div className="text-center space-y-2">
                            <p>Select a screener to view details</p>
                            <Button variant="outline" onClick={handleCreateScreener} className="gap-2">
                                Or create a new one
                            </Button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
