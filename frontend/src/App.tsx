import { useState, useEffect } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import AnalyzerPage from "@/pages/AnalyzerPage";
import { Toaster } from "@/components/ui/toaster";

function HomePage() {
  const [health, setHealth] = useState<string>("Connecting...");

  useEffect(() => {
    fetch("/api/health")
      .then((res) => res.json())
      .then((data) => setHealth(`Connected to API: ${data.status}`))
      .catch((err) => setHealth(`Error connecting to API: ${err.message}`));
  }, []);

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <Card className="w-full max-w-md shadow-xl border-2">
        <CardHeader>
          <CardTitle className="text-3xl font-extrabold text-primary tracking-tight">
            IAP Alpha
          </CardTitle>
          <p className="text-muted-foreground text-sm uppercase tracking-widest font-semibold">
            Investment Research Platform
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col space-y-6">
            <div className="flex items-center space-x-3 bg-slate-50 dark:bg-slate-900 p-3 rounded-lg border">
              <div
                className={`w-3 h-3 rounded-full animate-pulse ${health.includes("Connected") ? "bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.5)]" : health.includes("Error") ? "bg-red-500 shadow-[0_0_10px_rgba(239,68,68,0.5)]" : "bg-yellow-500 shadow-[0_0_10px_rgba(234,179,8,0.5)]"}`}
              ></div>
              <p className="text-sm font-medium">{health}</p>
            </div>

            <div className="grid gap-2">
              <Button
                onClick={async () => {
                  try {
                    const response = await fetch('/api/v1/auth/login', {
                      method: 'POST',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify({ username: 'testuser', password: 'TestPass123!' })
                    });

                    if (!response.ok) {
                      const errorText = await response.text();
                      alert(`Login failed: ${response.status} - ${errorText}`);
                      return;
                    }

                    const data = await response.json();

                    if (data.access_token) {
                      localStorage.setItem('access_token', data.access_token);
                      localStorage.setItem('refresh_token', data.refresh_token);
                      localStorage.setItem('user', JSON.stringify(data.user));
                      // Redirect to AAPL analyzer with correct UUID
                      window.location.href = '/analyzer/10000000-0000-0000-0000-000000000001';
                    } else {
                      alert('Login failed: No access token received');
                    }
                  } catch (err) {
                    alert('Login failed: ' + err);
                  }
                }}
                className="w-full"
              >
                Open AAPL Analyzer (Auto-Login)
              </Button>
              <Button variant="outline" onClick={() => window.location.reload()} className="w-full">
                Refresh Connection
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/analyzer/:companyId" element={<AnalyzerPage />} />
        {/* Fallback */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      <Toaster />
    </BrowserRouter>
  );
}

export default App;
