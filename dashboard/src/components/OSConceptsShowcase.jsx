/**
 * OSConceptsShowcase Component
 * Interactive showcase of Operating System concepts implemented in IntelliShell
 */

import React, { useState } from 'react';
import { 
  Cpu, 
  Clock, 
  Database,
  MemoryStick,
  HardDrive,
  Network,
  Zap,
  Lock,
  GitBranch,
  Activity,
  Terminal,
  Shield,
  Globe,
  BarChart3
} from 'lucide-react';
import '../styles/OSConceptsShowcase.css';

const osConcepts = [
  {
    id: 1,
    icon: Activity,
    title: 'Process Management',
    theory: 'Management of processes throughout their lifecycle, including execution, waiting and termination.',
    implementation: 'Retrieves process information from Linux /proc/[pid]. Identifies running, sleeping and zombie processes. Displays PID, state, memory and CPU usage. Supports process termination through SIGTERM.',
    mapping: 'Process Manager · read_process_info() · get_process_states() · control_process()',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-deep)'
  },
  {
    id: 2,
    icon: Clock,
    title: 'Process Scheduling',
    theory: 'The operating system allocates CPU execution time to processes.',
    implementation: 'Reads system load averages over 1, 5 and 15 minutes. Displays system load to help understand resource pressure.',
    mapping: 'System Monitoring · get_system_load() · /proc/loadavg',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-terracotta)'
  },
  {
    id: 3,
    icon: Database,
    title: 'Process Control Block (PCB)',
    theory: 'The OS maintains process-related information such as PID, state, memory and CPU time.',
    implementation: 'Reads /proc/[pid]/stat and /proc/[pid]/status to extract process metadata. Parses state codes (R, S, Z).',
    mapping: 'Process Manager · parse /proc/[pid]/stat · ProcessInfo struct',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-cinnamon)'
  },
  {
    id: 4,
    icon: Cpu,
    title: 'CPU Scheduling & Context Switching',
    theory: 'The OS switches between processes, saving and restoring their states.',
    implementation: 'Tracks context switches from /proc/stat. Monitors per-core CPU usage showing multitasking efficiency.',
    mapping: 'System Monitoring · get_context_switches() · get_cpu_per_core()',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-espresso)'
  },
  {
    id: 5,
    icon: MemoryStick,
    title: 'Memory Management',
    theory: 'The OS allocates, tracks and reclaims process memory.',
    implementation: 'Reads /proc/meminfo for total, used and available memory. Reads /proc/[pid]/status for per-process memory consumption (VmRSS).',
    mapping: 'System Monitoring · Memory metrics · Process memory tracking',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-mocha)'
  },
  {
    id: 6,
    icon: HardDrive,
    title: 'Virtual Memory & Swap',
    theory: 'The OS extends RAM by paging to disk when physical memory is full.',
    implementation: 'Monitors swap usage from /proc/meminfo. Displays total swap space and current utilization.',
    mapping: 'System Monitoring · get_swap_usage() · SwapTotal/SwapFree',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-umber)'
  },
  {
    id: 7,
    icon: HardDrive,
    title: 'File System & I/O',
    theory: 'The OS manages file operations and disk I/O.',
    implementation: 'Tracks disk read/write operations from /proc/diskstats. Monitors I/O activity per disk device.',
    mapping: 'System Monitoring · get_disk_io() · /proc/diskstats',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-walnut)'
  },
  {
    id: 8,
    icon: Terminal,
    title: 'Command Execution & System Calls',
    theory: 'Processes request OS services through system calls.',
    implementation: 'Executes shell commands via Linux (bash) or Windows (PowerShell). Captures stdout/stderr. Enforces timeout limits.',
    mapping: 'Command Executor · execute_linux_command() · execute_windows_command()',
    files: 'core-engine/src/linux_executor.rs, core-engine/src/windows_executor.rs',
    color: 'var(--brown-clay)'
  },
  {
    id: 9,
    icon: Shield,
    title: 'Security & Risk Assessment',
    theory: 'The OS enforces security policies and prevents harmful operations.',
    implementation: 'AI-powered risk assessment classifies commands (SAFE, MEDIUM, HIGH). Pattern matching detects destructive operations. User confirmation required for risky commands.',
    mapping: 'AI Service · Risk Classifier · Pattern Matchers',
    files: 'core-engine/src/linux_risk.rs, core-engine/src/windows_risk.rs, ai-service/src/intellishell_ai_service/risk.py',
    color: 'var(--brown-rust)'
  },
  {
    id: 10,
    icon: Zap,
    title: 'Inter-Process Communication (IPC)',
    theory: 'Processes communicate through shared mechanisms.',
    implementation: 'WebSocket connections enable real-time communication between frontend, Rust backend and Python AI service. HTTP REST APIs for command execution.',
    mapping: 'WebSocket Server · REST APIs · Multi-service architecture',
    files: 'core-engine/src/server.rs, ai-service/src/intellishell_ai_service/main.py',
    color: 'var(--brown-bronze)'
  },
  {
    id: 11,
    icon: Lock,
    title: 'Concurrency & Synchronization',
    theory: 'The OS coordinates access to shared resources.',
    implementation: 'Async Rust (tokio) manages concurrent connections. Thread-safe data structures prevent race conditions.',
    mapping: 'Async Runtime · Arc<Mutex<T>> · Concurrent WebSocket handlers',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-chestnut)'
  },
  {
    id: 12,
    icon: Clock,
    title: 'System Uptime & Time Management',
    theory: 'The OS tracks time since boot.',
    implementation: 'Reads /proc/uptime to display system uptime in human-readable format.',
    mapping: 'System Monitoring · get_uptime() · /proc/uptime',
    files: 'core-engine/src/server.rs',
    color: 'var(--brown-sienna)'
  },
  {
    id: 13,
    icon: Network,
    title: 'Dual Operating System Support',
    theory: 'Modern systems support multiple execution environments.',
    implementation: 'Automatic OS detection. Separate executors for Linux (bash in WSL2) and Windows (PowerShell). Platform-specific risk patterns and telemetry.',
    mapping: 'OS Detection · Execution Routing · Platform Abstraction',
    files: 'core-engine/src/server.rs, core-engine/src/linux_executor.rs, core-engine/src/windows_executor.rs',
    color: 'var(--brown-amber)'
  },
  {
    id: 14,
    icon: BarChart3,
    title: 'System Monitoring & Telemetry',
    theory: 'The OS provides interfaces to inspect system state.',
    implementation: 'Real-time monitoring of 13+ metrics: CPU (total & per-core), memory, disk, swap, load averages, uptime, context switches, process states. Updates every 2 seconds.',
    mapping: 'Telemetry Service · SystemSnapshot · Periodic polling',
    files: 'core-engine/src/server.rs, core-engine/src/windows_telemetry.rs',
    color: 'var(--brown-coffee)'
  }
];

const OSConceptsShowcase = () => {
  const [selectedConcept, setSelectedConcept] = useState(null);

  return (
    <div className="os-concepts-showcase">
      <div className="showcase-header">
        <h2>Operating System Concepts Implementation</h2>
        <p>Discover the 14 core OS concepts brought to life in IntelliShell</p>
      </div>

      <div className="concepts-grid">
        {osConcepts.map((concept) => {
          const IconComponent = concept.icon;
          return (
            <div
              key={concept.id}
              className={`concept-card ${selectedConcept?.id === concept.id ? 'selected' : ''}`}
              onClick={() => setSelectedConcept(concept)}
              style={{ borderColor: concept.color }}
            >
              <div className="concept-icon" style={{ backgroundColor: concept.color }}>
                <IconComponent size={24} />
              </div>
              <h3>{concept.title}</h3>
            </div>
          );
        })}
      </div>

      {selectedConcept && (
        <div className="concept-detail">
          <div className="detail-header" style={{ borderLeftColor: selectedConcept.color }}>
            <div className="detail-icon" style={{ backgroundColor: selectedConcept.color }}>
              {React.createElement(selectedConcept.icon, { size: 32 })}
            </div>
            <h2>{selectedConcept.title}</h2>
          </div>

          <div className="detail-section">
            <h3>Theory</h3>
            <p>{selectedConcept.theory}</p>
          </div>

          <div className="detail-section">
            <h3>Implementation in IntelliShell</h3>
            <p>{selectedConcept.implementation}</p>
          </div>

          <div className="detail-section">
            <h3>Code Mapping</h3>
            <p className="concept-mapping">{selectedConcept.mapping}</p>
          </div>

          <div className="detail-section">
            <h3>Source Files</h3>
            <code className="concept-files">{selectedConcept.files}</code>
          </div>
        </div>
      )}
    </div>
  );
};

export default OSConceptsShowcase;
