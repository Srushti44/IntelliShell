# WSL Command Generation and Chart Rendering Fixes

Fixes two critical bugs preventing IntelliShell from working on Windows/WSL2: AI-generated commands failing with path errors, and system charts never appearing on the Overview page.

The command generation fix adds WSL/Windows path context to the AI prompt, teaching it to generate `/mnt/d/` (lowercase) instead of `/mnt/D/`, and to wrap paths with spaces in double quotes. The chart rendering fix makes SystemCharts self-polling when no prop is provided, ensures OverviewPage passes snapshot data, and reduces the polling interval from 5 seconds to 2 seconds for faster updates.

**Watch for:** Windows path example using uppercase `/mnt/D/` instead of lowercase `/mnt/d/` (confirmed), staleTime misconfiguration that could cause cache thrashing (confirmed).

**Verdict**: CHANGES_REQUESTED

## High-level view

The SYSTEM_INSTRUCTIONS enhancement adds a dedicated WSL/Windows integration section with four examples demonstrating lowercase drive letters and proper quoting for spaces. One example uses uppercase `/mnt/D/` which will fail on many WSL systems where drive mounts are case-sensitive lowercase. The Linux command examples and safety principles remain intact.

SystemCharts gains a second useEffect that polls `/snapshot` every 2 seconds when currentSnapshot prop is undefined, using native fetch instead of axios. The polling interval is cleaned up on unmount. The data mapping matches the existing prop-driven path's structure.

OverviewPage now imports and calls useSystemSnapshot, passing the result to SystemCharts as currentSnapshot prop. This provides dual data paths: React Query cache via prop and self-polling fallback.

The polling interval reduction to 2000ms brings React Query in sync with SystemCharts' self-polling rate. However, staleTime remains at 2000ms, matching refetchInterval, which means data is considered fresh for as long as it takes to fetch new data — a configuration that can cause cache to never be used if queries complete faster than 2 seconds.

<details>
<summary>Issues (2)</summary>

1. **Uppercase drive letter in WSL example** — Line 165 uses `/mnt/D/` instead of `/mnt/d/`. Change to lowercase: `ls "/mnt/d/3rd year"`. This prevents "No such file or directory" errors on systems with case-sensitive mounts.

2. **staleTime equals refetchInterval** — useSystemSnapshot.js line 11 sets staleTime to 2000ms while refetchInterval is 2000ms. Reduce staleTime to 1000ms so React Query uses cache between polls instead of considering every fetch stale immediately.

</details>

<details>
<summary>Details</summary>

## WSL path context in SYSTEM_INSTRUCTIONS

The SYSTEM_INSTRUCTIONS string (lines 95-220 in main.py) now includes a "WSL/WINDOWS PATH RULES" section immediately after "CRITICAL RULES". It covers the core requirements: Windows drives mount at `/mnt/<lowercase-letter>/`, paths with spaces need double quotes, and common Windows locations map to `/mnt/c/Users/$USER/` paths.

Four WSL-specific examples follow the rules section:

```python
User: "list files in D drive 3rd year folder"
{"command": "ls \"/mnt/d/3rd year\"", "explanation": "Lists all files in the '3rd year' directory on D drive"}
```

Three of the four examples correctly use lowercase drive letters (`/mnt/d/`, `/mnt/c/`). The third example, line 165, uses uppercase:

```python
Request: "open D drive"
{"command": "ls /mnt/D/", "explanation": "Lists the contents of the Windows D drive via WSL mount point"}
```

This contradicts the rule stated six lines earlier: "CRITICAL: Always use lowercase drive letters: /mnt/d/ NOT /mnt/D/". On most WSL2 systems, `/mnt/D/` will fail with "No such file or directory" because the mount point is `/mnt/d/`. The example should be `ls /mnt/d/` (confirmed: line 165 uses uppercase).



## SystemCharts self-polling fallback

SystemCharts.jsx adds a second useEffect (lines 45-89) that activates when `currentSnapshot === undefined`. It uses native `fetch()` to poll `http://localhost:3000/snapshot` every 2 seconds, mapping the response to the same dataPoint shape as the prop-driven path:

```javascript
const dataPoint = {
  timestamp: new Date().toLocaleTimeString(...),
  cpuUsage: data.system?.cpu_usage || 0,
  memoryUsage: data.system?.memory_used_percent || 0,
  diskUsage: data.system?.disk_used_percent || 0,
  networkRx: (data.system?.network_rx_bytes || 0) / 1024 / 1024,
  networkTx: (data.system?.network_tx_bytes || 0) / 1024 / 1024,
  processCount: data.processes?.length || 0
};
```

Cleanup is present: `return () => { if (intervalId) clearInterval(intervalId); };` on line 88. The interval only starts when `currentSnapshot === undefined`, checked on line 47.

## OverviewPage snapshot prop

OverviewPage.jsx imports useSystemSnapshot (line 12), calls it (line 17), and passes the result to SystemCharts (line 44):

```javascript
const { data: snapshot } = useSystemSnapshot();
// ...
<SystemCharts currentSnapshot={snapshot} />
```

When React Query's cache has data, SystemCharts uses the prop-driven path. When cache is empty or stale, self-polling takes over.

## Polling interval reduction

useSystemSnapshot.js line 9 changes refetchInterval from 5000 to 2000:

```javascript
refetchInterval: 2000, // Refresh every 2 seconds
```

Line 11 leaves staleTime at 2000:

```javascript
staleTime: 2000,
```

This creates a configuration issue. `staleTime: 2000` means React Query considers data fresh for 2 seconds after fetch. `refetchInterval: 2000` means React Query fetches every 2 seconds. If a fetch completes in under 2 seconds (likely for a local Rust server), the data becomes stale before the next refetch fires, so React Query never uses the cache — every component mount triggers a new fetch even if data was fetched 1 second ago.

The bugfix plan (Step 4, line 10) specified changing staleTime to 1000ms, but the implemented file shows 2000ms (confirmed: line 11 reads `staleTime: 2000,`). This is not a blocking issue — the chart will still render — but it wastes network requests and defeats React Query's caching purpose.

</details>

<details>
<summary>File Map</summary>

- **ai-service/src/intellishell_ai_service/main.py** — Added WSL/Windows path rules and four WSL examples to SYSTEM_INSTRUCTIONS; one example uses uppercase `/mnt/D/` instead of lowercase
- **dashboard/src/components/SystemCharts.jsx** — Added self-polling useEffect that fetches `/snapshot` every 2s when currentSnapshot prop is absent; includes cleanup on unmount
- **dashboard/src/pages/OverviewPage.jsx** — Added useSystemSnapshot hook and passes snapshot to SystemCharts as currentSnapshot prop
- **dashboard/src/hooks/useSystemSnapshot.js** — Changed refetchInterval from 5000 to 2000; staleTime remains at 2000 instead of 1000 as planned

[View full diff](d:\3rd year\os project\IntelliShell/.git)

</details>
