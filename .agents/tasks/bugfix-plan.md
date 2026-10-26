# Implementation Plan: Fix Command Generation Windows Support & Chart Rendering

**Task**: Fix two critical bugs preventing IntelliShell from working properly on Windows/WSL2
**Context**: Project uses Rust (core-engine on port 3000), Python FastAPI (ai-service on port 8000), React/Vite (dashboard on port 5173)
**Build Commands**: `make check` runs all lints/tests/builds; `cargo run --release` for Rust, `uv run uvicorn ...` for Python, `npm run dev` for dashboard
**Test Strategy**: Manual functional testing using the running services as documented in HOW_TO_RUN_AND_TEST.md

---

## Implementation Steps

- [ ] 1. **Fix AI Command Generation - Add Windows/WSL Path Context**
      
      **What**: Modify SYSTEM_INSTRUCTIONS in main.py to teach Groq about Windows drive mounts under WSL2, proper quoting for spaces, and lowercase drive letters.
      
      **Why**: Currently generates broken commands like `ls /mnt/D/3rd year` when users mention Windows paths; needs to generate `ls "/mnt/d/3rd year"` with proper quoting and lowercase drive letters.
      
      **Files**: `d:\3rd year\os project\IntelliShell\ai-service\src\intellishell_ai_service\main.py`
      
      **Changes**:
      - Lines 95-164: Replace entire SYSTEM_INSTRUCTIONS string with enhanced version
      - Add new section "WSL2 + WINDOWS INTEGRATION" immediately after "CRITICAL RULES"
      - Add examples covering Windows path scenarios with correct /mnt/ mappings
      - Emphasize lowercase drive letters and double-quote wrapping for paths with spaces
      
      **Enhanced SYSTEM_INSTRUCTIONS** (replace lines 95-164):
      ```python
SYSTEM_INSTRUCTIONS = """You are an expert Linux command generator and system administrator. Your job is to translate natural language requests into precise, safe, and effective Linux commands.

CRITICAL RULES:
1. Respond with ONLY a JSON object - no markdown, no code blocks, no extra text
2. The JSON must have exactly these two fields:
   {"command": "<single shell command>", "explanation": "<clear one-sentence explanation>"}
3. Use ONLY real, standard Linux commands that actually exist (bash, coreutils, util-linux, etc.)
4. Generate commands that are safe by default - avoid destructive operations unless explicitly requested
5. Prefer non-destructive alternatives when possible (e.g., ls instead of rm, mv instead of dd)
6. Keep commands simple and focused on the specific task requested
7. Use proper quoting and escaping for file paths with spaces

WSL2 + WINDOWS INTEGRATION:
You are running in WSL2 on a Windows host. This means:
- Windows drives mount at /mnt/<lowercase-letter>/ (e.g., C: drive → /mnt/c/, D: drive → /mnt/d/)
- ALWAYS use lowercase drive letters: /mnt/d/ NOT /mnt/D/
- Paths with spaces MUST be wrapped in double-quotes: "/mnt/d/3rd year/project"
- When users mention "D drive", "C drive", "Windows Documents", "my downloads", etc., translate to WSL mount paths
- Current user's Windows home is typically /mnt/c/Users/$USER/
- Common Windows locations:
  * Documents → /mnt/c/Users/$USER/Documents
  * Downloads → /mnt/c/Users/$USER/Downloads
  * Desktop → /mnt/c/Users/$USER/Desktop
  * D drive → /mnt/d/

WSL PATH EXAMPLES:
User: "list files in D drive 3rd year folder"
{"command": "ls \"/mnt/d/3rd year\"", "explanation": "Lists all files in the '3rd year' directory on D drive"}

User: "show my Windows documents"
{"command": "ls \"/mnt/c/Users/$USER/Documents\"", "explanation": "Lists files in your Windows Documents folder"}

User: "open D drive"
{"command": "ls /mnt/d/", "explanation": "Lists all files and folders on D drive"}

User: "find Word files in os project folder"
{"command": "find \"/mnt/d/3rd year/os project\" -name '*.docx' -o -name '*.doc'", "explanation": "Searches for Word documents in the os project directory"}

COMMAND GENERATION GUIDELINES:
- File operations: Use ls, cat, head, tail, grep, find, cp, mv (avoid rm unless explicitly requested)
- Process management: Use ps, top, kill (with proper signal), systemctl status
- System info: Use df, du, free, uptime, uname, hostname
- Text processing: Use grep, sed, awk, cut, sort, uniq, wc
- Network: Use curl, wget, ping, netstat, ss, ip (avoid iptables/firewall changes)
- Permissions: Use chmod, chown (avoid recursive changes to system directories)
- Package management: Use apt list, dpkg -l (avoid install/remove unless explicitly requested)

SAFETY PRINCIPLES:
- Never generate commands that:
  * Delete system directories (/, /etc, /bin, /usr, /var)
  * Modify critical system files without explicit request
  * Run fork bombs or infinite loops
  * Write to raw disk devices (/dev/sda, etc.)
  * Shutdown/reboot without explicit request
  * Download and execute untrusted code
- For potentially destructive operations, use the safest minimal approach
- Add appropriate flags for safety (e.g., -i for interactive confirmation)

EXAMPLES:
Request: "show me the largest files in my home directory"
{"command": "du -ah ~ | sort -rh | head -20", "explanation": "Lists the 20 largest files/directories in your home folder sorted by size"}

Request: "find all log files modified today"
{"command": "find /var/log -name '*.log' -mtime 0 -type f", "explanation": "Searches /var/log for files with .log extension modified in the last 24 hours"}

Request: "check memory usage"
{"command": "free -h", "explanation": "Displays system memory usage in human-readable format"}

Request: "list PowerPoint files in my os project on D drive"
{"command": "find \"/mnt/d/3rd year/os project\" -name '*.pptx' -o -name '*.ppt'", "explanation": "Finds all PowerPoint presentations in the os project folder on D drive"}

Now respond to the user's request with ONLY the JSON object:"""
      ```
      
      **Verify**: 
      1. Start all three services (Rust, Python, dashboard)
      2. Open http://localhost:5173, go to Assistant page
      3. Test: "list files in D drive 3rd year folder" → should generate `ls "/mnt/d/3rd year"` (lowercase d, quoted)
      4. Test: "show my Windows documents" → should generate `ls "/mnt/c/Users/$USER/Documents"`
      5. Test: "open D drive" → should generate `ls /mnt/d/` (lowercase)
      6. Execute generated command and confirm it works without "No such file or directory" errors
      7. Check that commands with spaces are properly quoted

---

- [ ] 2. **Fix SystemCharts - Add Self-Polling Fallback**
      
      **What**: Add self-contained polling logic to SystemCharts component so it fetches snapshot data independently when no `currentSnapshot` prop is provided.
      
      **Why**: OverviewPage doesn't pass currentSnapshot prop, causing charts to never render. Making SystemCharts self-sufficient means it works on any page without requiring parent to manage data.
      
      **Files**: `d:\3rd year\os project\IntelliShell\dashboard\src\components\SystemCharts.jsx`
      
      **Changes**:
      - Lines 1-20: Add axios import at top: `import axios from 'axios';`
      - Lines 21-45: Replace existing useEffect with dual-mode logic:
        * If `currentSnapshot` prop exists and is valid, use prop-driven mode (existing behavior for MonitoringPage)
        * If `currentSnapshot` is null/undefined, activate self-polling mode: fetch from http://localhost:3000/snapshot every 2 seconds
      - Add new state `const [selfPolling, setSelfPolling] = useState(!currentSnapshot);`
      - Add cleanup for polling interval on unmount
      
      **Implementation** (replace lines 13-33):
      ```jsx
      export function SystemCharts({ currentSnapshot, className = '' }) {
        // Rolling window of last 20 snapshots (client-side history)
        const [history, setHistory] = useState([]);
        const [selfPolling, setSelfPolling] = useState(!currentSnapshot);
        const maxDataPoints = 20;

        // Self-polling mode: fetch snapshot independently if no prop provided
        useEffect(() => {
          if (currentSnapshot) {
            // Prop-driven mode: parent provides data
            setSelfPolling(false);
            return;
          }

          // Self-polling mode: fetch directly from Rust server
          setSelfPolling(true);
          const RUST_API_URL = 'http://localhost:3000';
          
          const fetchSnapshot = async () => {
            try {
              const response = await axios.get(`${RUST_API_URL}/snapshot`, { timeout: 5000 });
              const snapshot = response.data;
              
              const dataPoint = {
                timestamp: new Date().toLocaleTimeString('en-US', { 
                  hour12: false, 
                  hour: '2-digit', 
                  minute: '2-digit', 
                  second: '2-digit' 
                }),
                cpuUsage: snapshot.cpu_percent || 0,
                memoryUsage: snapshot.memory_used_percent || 0,
                diskUsage: snapshot.disk_used_percent || 0,
                networkRx: (snapshot.network_rx_bytes || 0) / 1024 / 1024, // MB
                networkTx: (snapshot.network_tx_bytes || 0) / 1024 / 1024, // MB
                processCount: snapshot.processes?.length || 0
              };

              setHistory(prev => {
                const updated = [...prev, dataPoint];
                return updated.slice(-maxDataPoints);
              });
            } catch (error) {
              console.error('Failed to fetch snapshot:', error);
            }
          };

          // Fetch immediately, then every 2 seconds
          fetchSnapshot();
          const intervalId = setInterval(fetchSnapshot, 2000);

          return () => clearInterval(intervalId);
        }, [currentSnapshot]);

        // Update history when prop changes (prop-driven mode for MonitoringPage)
        useEffect(() => {
          if (!currentSnapshot?.system || selfPolling) return;

          const dataPoint = {
            timestamp: new Date().toLocaleTimeString('en-US', { 
              hour12: false, 
              hour: '2-digit', 
              minute: '2-digit', 
              second: '2-digit' 
            }),
            cpuUsage: currentSnapshot.system.cpu_usage || 0,
            memoryUsage: currentSnapshot.system.memory_used_percent || 0,
            diskUsage: currentSnapshot.system.disk_used_percent || 0,
            networkRx: (currentSnapshot.system.network_rx_bytes || 0) / 1024 / 1024, // MB
            networkTx: (currentSnapshot.system.network_tx_bytes || 0) / 1024 / 1024, // MB
            processCount: currentSnapshot.processes?.length || 0
          };

          setHistory(prev => {
            const updated = [...prev, dataPoint];
            return updated.slice(-maxDataPoints);
          });
        }, [currentSnapshot, selfPolling]);
      ```
      
      **Verify**:
      1. Start Rust core-engine: `cd core-engine && cargo run --release`
      2. Start dashboard: `cd dashboard && npm run dev`
      3. Open http://localhost:5173 (OverviewPage)
      4. Charts should appear within 4-6 seconds (2 data points at 2-second intervals)
      5. Check all 4 charts show real data: CPU, Memory, Network, Disk
      6. Go to Monitoring page - charts should still work (using prop-driven mode)
      7. Verify charts update every 2 seconds with smooth animation

---

- [ ] 3. **Fix OverviewPage - Pass Snapshot to SystemCharts (Backup Fix)**
      
      **What**: Add useSystemSnapshot hook to OverviewPage and pass snapshot data to SystemCharts as prop, providing a second independent data path.
      
      **Why**: Belt-and-suspenders approach - even though SystemCharts now self-polls, having OverviewPage also fetch means React Query's caching and error handling are available.
      
      **Files**: `d:\3rd year\os project\IntelliShell\dashboard\src\pages\OverviewPage.jsx`
      
      **Changes**:
      - Line 6: Add import: `import { useSystemSnapshot } from '../hooks/useSystemSnapshot';`
      - Line 10: Add hook call: `const { data: snapshot } = useSystemSnapshot();`
      - Line 27: Change `<SystemCharts />` to `<SystemCharts currentSnapshot={snapshot} />`
      
      **Implementation** (replace lines 1-13 and line 27):
      ```jsx
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
      ```
      
      And line 27 change to:
      ```jsx
      <SystemCharts currentSnapshot={snapshot} />
      ```
      
      **Verify**:
      1. Restart dashboard: `npm run dev`
      2. Open http://localhost:5173
      3. Charts should appear even faster now (React Query cache + 2s self-polling)
      4. Open browser DevTools Network tab - should see /snapshot requests every 5 seconds (React Query) OR every 2 seconds (self-polling fallback)
      5. Charts should smoothly update with no "Collecting telemetry" message staying forever

---

- [ ] 4. **Fix MonitoringPage - Reduce Polling Interval**
      
      **What**: Reduce useSystemSnapshot refetch interval from 5 seconds to 2 seconds for faster chart updates.
      
      **Why**: 5-second polling means 10+ seconds to get 2 data points; 2-second polling matches SystemCharts' self-polling rate and provides smoother real-time feel.
      
      **Files**: `d:\3rd year\os project\IntelliShell\dashboard\src\hooks\useSystemSnapshot.js`
      
      **Changes**:
      - Line 9: Change `refetchInterval: 5000,` to `refetchInterval: 2000,`
      - Line 10: Change `staleTime: 2000,` to `staleTime: 1000,`
      
      **Implementation** (replace lines 7-12):
      ```javascript
      export function useSystemSnapshot() {
        return useQuery({
          queryKey: ['system-snapshot'],
          queryFn: () => rustApi.getSnapshot(),
          refetchInterval: 2000, // Refresh every 2 seconds (was 5000)
          retry: 2,
          staleTime: 1000, // Consider data stale after 1 second
        });
      }
      ```
      
      **Verify**:
      1. Restart dashboard if needed
      2. Open http://localhost:5173/monitoring
      3. Watch the "Current Resource Usage" metric cards - should update every 2 seconds
      4. Charts should build history faster (2 points in 4 seconds instead of 10 seconds)
      5. CPU usage should feel more "live" and responsive to actual load changes

---

- [ ] 5. **End-to-End Verification - Both Bugs Fixed**
      
      **What**: Complete functional test covering both bug fixes in realistic usage scenarios.
      
      **Why**: Confirms fixes work together in production-like workflow, not just in isolation.
      
      **Files**: No code changes - testing only
      
      **Test Scenario A - Windows Path Commands**:
      1. Start all services: Rust (port 3000), Python (port 8000), Dashboard (port 5173)
      2. Open http://localhost:5173/assistant
      3. Enter: "list all files in D drive 3rd year folder"
      4. Click "Generate Command"
      5. **Expected**: Generated command is `ls "/mnt/d/3rd year"` (lowercase d, quoted path)
      6. Click "Execute Command"
      7. **Expected**: Command executes successfully, shows file listing in stdout (no "No such file or directory")
      8. Try: "show Word documents in os project on D drive"
      9. **Expected**: Generates `find "/mnt/d/3rd year/os project" -name '*.docx' -o -name '*.doc'`
      10. Execute and confirm it works
      
      **Test Scenario B - Chart Rendering on All Pages**:
      1. Open http://localhost:5173 (Overview page)
      2. **Expected**: Within 6 seconds, all 4 charts appear (CPU, Memory, Network, Disk)
      3. **Expected**: Charts show real data (not 0 or NaN)
      4. **Expected**: Charts update every 2 seconds with smooth animation
      5. **Expected**: No "Collecting telemetry data..." message stuck forever
      6. Go to http://localhost:5173/monitoring
      7. **Expected**: Charts still work, metric cards update every 2 seconds
      8. Go back to Overview - charts should continue updating without reset
      
      **Test Scenario C - Cross-Feature Integration**:
      1. On Overview page, watch CPU chart baseline
      2. Go to Assistant page, generate and execute: "list processes using most CPU"
      3. Execute the command
      4. Go back to Overview page
      5. **Expected**: CPU chart shows slight spike from command execution
      6. **Expected**: Network chart shows activity if command triggered network I/O
      7. Go to History page - executed command should appear in history
      
      **Success Criteria**:
      - ✅ All Windows path commands generate correct /mnt/<lowercase>/ paths with quotes
      - ✅ Commands execute without "No such file or directory" errors
      - ✅ Charts render on Overview page within 6 seconds
      - ✅ Charts render on Monitoring page within 6 seconds
      - ✅ All charts show real, changing data (CPU 0-100%, Memory 0-100%, Network >0 MB, Disk 0-100%)
      - ✅ Charts update smoothly every 2 seconds
      - ✅ No console errors in browser DevTools
      - ✅ No "Collecting telemetry" placeholder stuck forever
      - ✅ System feels responsive and "live"
      
      **If any test fails**:
      - Check browser console for JavaScript errors
      - Check Rust server terminal for HTTP request logs
      - Check Python server terminal for generation errors
      - Use `curl http://localhost:3000/snapshot` to verify Rust server returns valid data
      - Use `curl http://localhost:8000/health` to verify Python server is running
      - Clear browser cache (Ctrl+Shift+R) and retry

---

## Summary

This plan fixes two critical bugs:

1. **BUG 1 (Command Generation)**: Enhanced SYSTEM_INSTRUCTIONS with WSL2/Windows path context, teaching Groq to generate proper /mnt/ paths with lowercase drive letters and double-quote wrapping for spaces.

2. **BUG 2 (Chart Rendering)**: Three-layer fix:
   - SystemCharts gains self-polling ability (works standalone on any page)
   - OverviewPage explicitly passes snapshot prop (React Query benefits)
   - Reduced polling interval to 2 seconds (faster, smoother updates)

**Why This Approach**:
- Step 1 targets root cause: AI doesn't know about WSL mount conventions
- Steps 2-4 provide defense-in-depth: charts work even if one data path fails
- Step 5 ensures fixes integrate properly end-to-end

**Total Estimated Time**: 1-2 hours (mostly testing and verification)

**Risk**: Low - changes are additive (new instructions, new polling logic) without breaking existing functionality
