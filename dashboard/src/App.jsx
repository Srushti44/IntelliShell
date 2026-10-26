/**
 * IntelliShell Dashboard - Main Application (Phase 7 Swiss Design)
 * AI-Based System Intent Engine for Safe Linux Command Execution
 */

import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Sidebar } from './components/Sidebar';
import { TopBar } from './components/TopBar';
import { OverviewPage } from './pages/OverviewPage';
import { AssistantPage } from './pages/AssistantPage';
import { ProcessesPage } from './pages/ProcessesPage';
import { HistoryPage } from './pages/HistoryPage';
import { MonitoringPage } from './pages/MonitoringPage';
import { SettingsPage } from './pages/SettingsPage';
import './App.css';
import './styles/colors.css';
import './styles/typography.css';

// Create a query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

// Main layout with sidebar
function Layout({ children }) {
  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-content">
        {children}
      </div>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<Navigate to="/overview" replace />} />
            <Route path="/overview" element={<OverviewPage />} />
            <Route path="/assistant" element={<AssistantPage />} />
            <Route path="/processes" element={<ProcessesPage />} />
            <Route path="/monitoring" element={<MonitoringPage />} />
            <Route path="/history" element={<HistoryPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </Layout>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
