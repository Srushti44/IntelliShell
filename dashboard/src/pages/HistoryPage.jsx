/**
 * Command History Page
 */

import { useState } from 'react';
import { CommandHistory } from '../components/CommandHistory';
import { TopBar } from '../components/TopBar';
import { PageSection } from '../components/PageSection';

export function HistoryPage({ onCommandReuse }) {
  return (
    <>
      <TopBar 
        title="Command History" 
        subtitle="Review executed commands and their outcomes"
      />
      
      <main className="p-8" style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
        <PageSection
          description="View previously executed commands with their risk levels, execution methods, outputs, and exit codes. Reuse commands safely by reviewing them through the command assistant."
        >
          <CommandHistory onReuse={onCommandReuse} />
        </PageSection>
      </main>
    </>
  );
}


export default HistoryPage;
