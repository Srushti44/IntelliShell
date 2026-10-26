/**
 * Process Manager Page
 */

import { ProcessManager } from '../components/ProcessManager';
import { TopBar } from '../components/TopBar';
import { PageSection } from '../components/PageSection';

export function ProcessesPage() {
  return (
    <>
      <TopBar 
        title="Process Manager" 
        subtitle="Monitor and control running processes"
      />
      
      <main className="p-8" style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
        <PageSection
          description="Search, filter, and manage system processes. Terminating processes can cause data loss or system instability—confirm carefully before proceeding."
        >
          <ProcessManager />
        </PageSection>
      </main>
    </>
  );
}


export default ProcessesPage;
