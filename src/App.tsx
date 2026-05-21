import { Routes, Route, Navigate } from "react-router-dom";
import { Sidebar } from "@/components/Sidebar";
import { Inbox } from "@/views/Inbox";
import { Settings } from "@/views/Settings";
import { Onboarding } from "@/views/Onboarding";

export default function App() {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      <Sidebar />
      <main className="flex-1 overflow-hidden">
        <Routes>
          <Route path="/" element={<Navigate to="/inbox" replace />} />
          <Route path="/inbox" element={<Inbox />} />
          <Route path="/inbox/:accountId" element={<Inbox />} />
          <Route path="/settings/*" element={<Settings />} />
          <Route path="/onboarding/*" element={<Onboarding />} />
        </Routes>
      </main>
    </div>
  );
}
