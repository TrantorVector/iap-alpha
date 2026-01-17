import { useState, useEffect } from "react";

function App() {
  const [health, setHealth] = useState<string>("Connecting...");

  useEffect(() => {
    fetch("/api/health")
      .then((res) => res.json())
      .then((data) => setHealth(`Connected to API: ${data.status}`))
      .catch((err) => setHealth(`Error connecting to API: ${err.message}`));
  }, []);

  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center">
      <div className="bg-white p-8 rounded-lg shadow-md">
        <h1 className="text-2xl font-bold mb-4 text-blue-600">
          Investment Research Platform
        </h1>
        <div className="flex items-center space-x-2">
          <div
            className={`w-3 h-3 rounded-full ${health.includes("Connected") ? "bg-green-500" : "bg-red-500"}`}
          ></div>
          <p className="text-gray-600">{health}</p>
        </div>
      </div>
    </div>
  );
}

export default App;
