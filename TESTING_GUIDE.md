# IntelliShell Testing Guide
**Complete Testing Checklist for Enhanced Monitoring System**

---

## PREREQUISITE: Start All Services

### Terminal 1 - Rust Core Engine (WSL/Linux):
```bash
cd "/mnt/d/3rd year/os project 2/IntelliShell/core-engine"
cargo run --release --bin intellishell-server
```
**Expected**: Server starts on http://127.0.0.1:3000

### Terminal 2 - Python AI Service (WSL/Linux):
```bash
cd "/mnt/d/3rd year/os project 2/IntelliShell/ai-service"
export GROQ_API_KEY='your_groq_api_key_here'
export GROQ_MODEL='openai/gpt-oss-120b'
python3 -m uvicorn intellishell_ai_service.main:app --host 127.0.0.1 --port 8000
```
**Expected**: AI service starts on http://127.0.0.1:8000

### Terminal 3 - Frontend (Windows PowerShell OR WSL):
**Windows PowerShell**:
```powershell
cd "d:\3rd year\os project 2\IntelliShell\dashboard"
npm install
npm run dev
```

**OR WSL**:
```bash
cd "/mnt/d/3rd year/os project 2/IntelliShell/dashboard"
npm install
npm run dev
```
**Expected**: Frontend starts on http://localhost:5173

---

## TEST 1: System Overview ✓

**Navigate to**: Home / System Overview section

**Verify**:
- [ ] CPU Usage displays percentage (e.g., "2.5%")
- [ ] Memory Usage displays percentage and bytes (e.g., "8.2% - 630MB / 7.48GB")
- [ ] Active Processes displays count (e.g., "10")
- [ ] Last Updated shows "0s ago" and updates
- [ ] Top Memory Consumers list shows 5 processes with PID, name, state

**Expected Behavior**: All values update every 2 seconds without page refresh

---

## TEST 2: Monitoring Page - Existing Charts ✓

**Navigate to**: Monitoring section

**Verify Existing Charts**:
- [ ] **CPU & Memory Usage Chart**:
  - Shows area chart with two lines (CPU in brown, Memory in terracotta)
  - X-axis shows timestamps
  - Y-axis shows 0-100%
  - Chart updates as new data arrives (rolling window)
  - Tooltip displays on hover

- [ ] **Network Traffic Chart**:
  - Shows line chart with RX and TX
  - X-axis shows timestamps
  - Y-axis shows MB transferred
  - Values increase when network activity happens
  - Both lines visible and updating

- [ ] **Disk Usage & Process Count Chart**:
  - Shows dual-axis chart (Disk % on left, Process count on right)
  - Disk usage line relatively stable
  - Process count may fluctuate
  - Both axes labeled correctly

**Expected Behavior**: 
- "Collecting telemetry data..." message disappears after 4-6 seconds
- All charts display and update smoothly
- NO JavaScript errors in browser console (F12)

---

## TEST 3: Enhanced System Metrics (7+ Metrics) ✓

**Navigate to**: Monitoring → "Comprehensive System Metrics" section

### Metric 1: System Load Averages
- [ ] Three metric cards display: "1 min", "5 min", "15 min"
- [ ] Each shows a decimal number (e.g., "0.25", "0.42", "0.38")
- [ ] Values are reasonable for your system (typically < number of CPU cores)

### Metric 2: Swap Usage
- [ ] Donut chart displays with "Used" and "Free" segments
- [ ] Chart shows swap breakdown
- [ ] Summary below chart shows bytes and percentage
- [ ] **OR** displays "No swap configured" if no swap exists

### Metric 3: CPU Per-Core Usage
- [ ] Bar chart displays with one bar per CPU core
- [ ] X-axis labels show "Core 0", "Core 1", etc.
- [ ] Y-axis shows 0-100%
- [ ] Bars update dynamically showing individual core activity

### Metric 4: Process State Distribution
- [ ] Donut chart displays with 3 segments: Running, Sleeping, Zombie
- [ ] Each segment labeled with count
- [ ] Legend shows colors and names
- [ ] Sleeping processes should be the largest segment

### Metric 5-7: System Statistics Grid
- [ ] **Disk Read (Total)**: Shows cumulative bytes read (e.g., "124.5 GB")
- [ ] **Disk Write (Total)**: Shows cumulative bytes written (e.g., "89.2 GB")
- [ ] **System Uptime**: Shows formatted time (e.g., "2d 5h" or "3h 25m")
- [ ] **Context Switches**: Shows large number (e.g., "1,234,567,890")

**Expected Behavior**: All 7+ metrics display real data, NO "0" or "undefined" values

---

## TEST 4: Current Resource Snapshot ✓

**Navigate to**: Monitoring → "Current Resource Snapshot" section

**Verify Metric Cards**:
- [ ] CPU Usage %
- [ ] Memory Used % (with subtitle showing bytes)
- [ ] Disk Used % (with subtitle showing bytes)
- [ ] Active Processes (number)
- [ ] Network RX (formatted bytes)
- [ ] Network TX (formatted bytes)
- [ ] System Load (1m) (decimal number)
- [ ] Swap Used % (with subtitle or "No swap")

**Expected Behavior**: All 8 cards display and match values from other sections

---

## TEST 5: Data Synchronization ✓

**Open**: System Overview AND Monitoring Page (side by side if possible)

**Verify**:
- [ ] CPU percentage matches in both sections
- [ ] Memory percentage matches in both sections
- [ ] Process count matches in both sections
- [ ] Both sections update at the same time (within 2 seconds)
- [ ] Network RX/TX values match

**Expected Behavior**: Perfect synchronization, same data source, same update interval

---

## TEST 6: Dynamic Updates ✓

**Action**: Generate some system activity:
```bash
# In WSL, create CPU load
yes > /dev/null &
# Note the PID, e.g., 12345
```

**Verify**:
- [ ] CPU usage increases in System Overview
- [ ] CPU usage increases in Monitoring charts
- [ ] CPU per-core chart shows activity on one or more cores
- [ ] Process count may increase by 1

**Action**: Stop the CPU load:
```bash
kill 12345  # Replace with actual PID
```

**Verify**:
- [ ] CPU usage decreases
- [ ] Charts reflect the decrease

**Expected Behavior**: Real-time response to actual system activity

---

## TEST 7: Dual-OS Feature (Linux ⇔ Windows) ✓

**Navigate to**: AI Command Assistant

**Verify UI**:
- [ ] Two OS selection buttons visible: 🐧 Linux and 🪟 Windows
- [ ] One button is highlighted (active state)
- [ ] Clicking switches between operating systems

### Linux Test:
1. [ ] Select **Linux**
2. [ ] Enter: "list all files"
3. [ ] Generated command should be Linux syntax: `ls -la` or similar
4. [ ] Risk level displays (SAFE/MEDIUM/HIGH)
5. [ ] Explanation references the specific command
6. [ ] Execute button available

### Windows Test:
1. [ ] Select **Windows**
2. [ ] Enter: "list all files"
3. [ ] Generated command should be Windows syntax: `Get-ChildItem -Force` or similar
4. [ ] Risk level displays (SAFE/MEDIUM/HIGH)
5. [ ] Explanation references the specific command
6. [ ] Execute button available

**Expected Behavior**: Commands are OS-specific, NOT the same for both

---

## TEST 8: Risk Classification Accuracy ⚠️

**Navigate to**: AI Command Assistant

### SAFE Commands (should be classified as SAFE):
- [ ] Linux: `pwd`
- [ ] Linux: `whoami`
- [ ] Linux: `ls -la`
- [ ] Windows: `Get-Location`
- [ ] Windows: `Get-ChildItem`

### MEDIUM RISK Commands:
- [ ] Linux: `rm file.txt`
- [ ] Linux: `chmod 777 file`
- [ ] Linux: `mkdir test`
- [ ] Windows: `Remove-Item file.txt`
- [ ] Windows: `New-Item -ItemType Directory test`

### HIGH RISK Commands:
- [ ] Linux: `rm -rf /tmp/*`
- [ ] Linux: `sudo rm -rf /`
- [ ] Windows: `Remove-Item C:\\ -Recurse -Force`
- [ ] Windows: `Format-Volume C:`

**Expected Behavior**: 
- HIGH RISK commands should be blocked or require strong confirmation
- Explanations should be personalized, NOT generic
- Each explanation should reference the SPECIFIC command and its dangers

**⚠️ Known Issue to Verify**: Previously reported that "everything is classified as SAFE" - this should now be fixed with rule-based patterns.

---

## TEST 9: Process Manager (End-to-End) ⚠️

**Navigate to**: Process Manager

### View Processes:
- [ ] Process list displays
- [ ] Shows PID, Name, State, Memory, CPU for each process
- [ ] List is not empty
- [ ] Top 100 processes shown

### Search by Name:
1. [ ] Type "bash" in search box
2. [ ] List filters to show only bash processes
3. [ ] Clear search → full list returns

### Search by PID:
1. [ ] Find a PID from the list (e.g., 1234)
2. [ ] Type the PID in search box
3. [ ] Only that process shows

### Create Test Process:
```bash
# In WSL terminal
sleep 300 &
echo $!  # Note the PID
```

### Find and Terminate:
1. [ ] Search for "sleep" process
2. [ ] Process appears in list
3. [ ] Click "Terminate" button
4. [ ] Confirmation dialog appears
5. [ ] Confirm termination
6. [ ] Process disappears from list
7. [ ] Verify in terminal: `ps aux | grep sleep` shows nothing

### Protected Process Test:
1. [ ] Try to terminate PID 1 (init/systemd)
2. [ ] Should be REJECTED with error message
3. [ ] Try to terminate the IntelliShell backend process
4. [ ] Should be REJECTED with error message

**Expected Behavior**: All operations work smoothly, protected processes cannot be terminated

---

## TEST 10: Browser Console Check ✓

**Action**: Open browser DevTools (F12)

**Verify**:
- [ ] NO red errors in Console tab
- [ ] NO failed network requests in Network tab
- [ ] All API calls to `localhost:3000/snapshot` succeed (Status 200)
- [ ] API responses contain all expected fields (load_1min, swap_total_bytes, cpu_cores, etc.)

**Expected Behavior**: Clean console, no errors, all API calls successful

---

## ACCEPTANCE CHECKLIST

### Core Requirements:
- [ ] System Overview displays live metrics
- [ ] Monitoring section synchronized with System Overview
- [ ] All charts are genuinely live (NO dummy data)
- [ ] Minimum 7 distinct Linux metrics implemented (we have 13)
- [ ] Appropriate visualizations for each metric type
- [ ] All metrics update dynamically every 2 seconds
- [ ] Network, Wi-Fi, disk telemetry preserved
- [ ] Dual-OS support working (Linux WSL2 + Windows host)

### Risk Assessment:
- [ ] SAFE commands classified correctly
- [ ] MEDIUM RISK commands classified correctly
- [ ] HIGH RISK commands classified correctly
- [ ] Explanations are personalized (not generic)
- [ ] Explanations reference specific commands
- [ ] Execution confirmation workflow works

### Process Manager:
- [ ] Live process retrieval working
- [ ] Search and filtering functional
- [ ] Process termination works
- [ ] Protected processes cannot be terminated
- [ ] Process list updates after termination

### No Regressions:
- [ ] Existing functionality preserved
- [ ] No compilation errors
- [ ] No runtime errors
- [ ] All pages load correctly
- [ ] Navigation works smoothly

---

## TROUBLESHOOTING

### If Charts Don't Display:
1. Check browser console for errors
2. Verify backend is running: `curl http://localhost:3000/health`
3. Verify snapshot endpoint: `curl http://localhost:3000/snapshot?os=linux`
4. Clear browser cache and reload (Ctrl+F5)

### If Metrics Show 0 or Undefined:
1. Check if running in WSL (required for Linux metrics)
2. Verify `/proc` filesystem is accessible in WSL
3. Check backend logs for telemetry collection errors
4. Verify SystemSnapshot structure matches frontend expectations

### If AI Commands Don't Generate:
1. Verify AI service is running on port 8000
2. Check Groq API key is set correctly
3. Check AI service logs for errors
4. Verify network connectivity to Groq API

### If Process Manager Doesn't Work:
1. Verify `/proc` filesystem access
2. Check backend `/processes` endpoint: `curl http://localhost:3000/processes`
3. Verify process termination endpoint works
4. Check for permission errors in backend logs

---

## SUCCESS CRITERIA

✅ **PASS** if:
- All 13 metrics display real data
- All charts update dynamically every 2 seconds
- System Overview and Monitoring are synchronized
- Dual-OS commands generate correctly
- Risk classifications are accurate
- Process Manager operations work
- NO errors in browser console
- NO regressions in existing features

⚠️ **NEEDS FIX** if:
- Any metrics show 0, undefined, or "No data"
- Charts don't update or show static data
- Synchronization issues between sections
- Risk classifications incorrect (everything SAFE)
- Process Manager fails to terminate processes
- Errors in browser console
- Services crash or fail to start

---

## REPORT RESULTS

After testing, document:
1. Which tests PASSED ✅
2. Which tests FAILED ❌
3. Any errors encountered
4. Screenshots of working features
5. Any unexpected behavior

**All features have been implemented. Testing is the final verification step!**

