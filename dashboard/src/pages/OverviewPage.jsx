/**
 * Overview Page
 * Main dashboard with system monitoring and charts
 */

import { useState, useEffect } from 'react';
import { SystemOverview } from '../components/SystemOverview';
import { CommandExecutor } from '../components/CommandExecutor';
import { PageSection } from '../components/PageSection';
import { SystemCharts } from '../components/SystemCharts';
import { TopBar } from '../components/TopBar';
import { useSystemSnapshot } from '../hooks/useSystemSnapshot';

export function OverviewPage() {
  const [reusedCommand, setReusedCommand] = useState('');
  const [refreshKey, setRefreshKey] = useState(0);
  const { data: snapshot } = useSystemSnapshot();

  const handleRefresh = () => {
    setRefreshKey(prev => prev + 1);
  };

  return (
    <>
      <TopBar 
        title="System Overview" 
        subtitle="AI-Based System Intent Engine for Safe Linux Command Execution"
        onRefresh={handleRefresh}
      />
      
      <main className="p-8" style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
        {/* Introduction */}
        <PageSection
          title="About IntelliShell"
          description="IntelliShell translates natural language into safe Linux commands through AI-powered generation, risk assessment, and policy-controlled execution. It monitors system resources, manages processes, and provides telemetry-informed troubleshooting with persistent command history."
        />

        {/* System Metrics */}
        <PageSection title="Live System Metrics">
          <SystemOverview key={refreshKey} />
        </PageSection>

        {/* System Charts */}
        <PageSection title="Resource Trends">
          <SystemCharts currentSnapshot={snapshot} />
        </PageSection>

        {/* Command Assistant */}
        <PageSection 
          title="Quick Command Execution"
          description="Execute commands directly with automatic risk assessment and execution routing."
        >
          <CommandExecutor initialCommand={reusedCommand} key={reusedCommand} />
        </PageSection>
      </main>
    </>
  );
}


export default OverviewPage;
