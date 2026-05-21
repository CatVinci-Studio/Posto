import { useState } from "react";
import { Routes, Route, Navigate, useLocation } from "react-router-dom";
import { Sidebar } from "@/components/Sidebar";
import { Inbox } from "@/views/Inbox";
import { Settings } from "@/views/Settings";
import { Onboarding } from "@/views/Onboarding";
import { Compose } from "@/views/Compose";
import { useScreenSize } from "@/hooks/useMediaQuery";

export default function App() {
  const { isMobile } = useScreenSize();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const location = useLocation();

  // On mobile, hide the sidebar entirely on onboarding/compose (full-screen flows).
  const isFullscreenRoute =
    location.pathname.startsWith("/onboarding") ||
    location.pathname.startsWith("/compose");

  const showSidebar = !isMobile || (!isFullscreenRoute && sidebarOpen);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      {!isFullscreenRoute && (
        <Sidebar
          open={showSidebar}
          onClose={() => setSidebarOpen(false)}
        />
      )}
      <main className="flex-1 overflow-hidden">
        <Routes>
          <Route path="/" element={<Navigate to="/inbox" replace />} />
          <Route
            path="/inbox"
            element={<Inbox onOpenSidebar={() => setSidebarOpen(true)} />}
          />
          <Route
            path="/inbox/:accountId"
            element={<Inbox onOpenSidebar={() => setSidebarOpen(true)} />}
          />
          <Route path="/compose" element={<Compose />} />
          <Route path="/compose/reply/:messageId" element={<Compose />} />
          <Route path="/compose/draft/:draftId" element={<Compose />} />
          <Route path="/settings/*" element={<Settings />} />
          <Route path="/onboarding/*" element={<Onboarding />} />
        </Routes>
      </main>
    </div>
  );
}
