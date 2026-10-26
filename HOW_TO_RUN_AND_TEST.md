# How to Run and Test IntelliShell - Complete Guide

**Last Updated**: 2026-10-21  
**Your System**: Windows with WSL2  
**Time Required**: 15-20 minutes

---

## Prerequisites Check

Before starting, verify you have:

1. **WSL2 Ubuntu** installed and working
2. **Rust 1.88+** in WSL: `rustc --version`
3. **Python 3.10+** in WSL: `python3 --version`
4. **Node.js 18+** in Windows: `node --version`
5. **Groq API Key**: You already have this in `.env` file ✅

---

## Part 1: Setup Configuration Files

### Step 1A: Create AI Service .env file

Open WSL terminal and run:

```bash
cd /mnt/d/3rd\ year/os\ project/IntelliShell/ai-service
```

Create the `.env` file:

```bash
cat > .env << 'EOF'
# AI Provider Configuration
AI_PROVIDER=groq
GROQ_API_KEY=your_groq_api_key_here
GROQ_MODEL=llama-3.3-70b-versatile

# Core Engine Integration
CORE_ENGINE_URL=http://localhost:3000

# Timeouts
PROVIDER_TIMEOUT=120
TOKEN_TTL_SECONDS=300
EOF
```

### Step 1B: Create Core Engine .env file

```bash
cd /mnt/d/3rd\ year/os\ project/IntelliShell/core-engine
```

Create the `.env` file:

```bash
cat > .env << 'EOF'
DATABASE_URL=sqlite:./intellishell.db
LOG_LEVEL=info
BIND_ADDRESS=127.0.0.1:3000
EOF
```

---

## Part 2: Start the Services

You need **3 terminal windows** open. Do these in order:

### Terminal 1: Start Rust Core Engine (WSL)

```bash
# Open WSL terminal
cd /mnt/d/3rd\ year/os\ project/IntelliShell/core-engine

# Build and run
cargo run --release
```

**What to look for:**
- Should print: `🚀 IntelliShell Core Engine starting...`
- Should print: `📊 Server running on http://127.0.0.1:3000`
- Should print: `💾 Database initialized at ./intellishell.db`

**If it fails:**
- Check Rust version: `rustc --version` (need 1.88+)
- Try debug mode: `cargo run` (without --release)

---

### Terminal 2: Start Python AI Service (WSL)

Open a **NEW WSL terminal** (keep Terminal 1 running):

```bash
cd /mnt/d/3rd\ year/os\ project/IntelliShell/ai-service

# Install dependencies and run
uv run uvicorn intellishell_ai_service.main:app --host 127.0.0.1 --port 8000
```

**What to look for:**
- Should print: `INFO:     Started server process`
- Should print: `INFO:     Uvicorn running on http://127.0.0.1:8000`
- Should print: `✅ Using Groq provider`

**If it fails:**
- Install uv: `pip install uv`
- Check Python version: `python3 --version` (need 3.10+)
- Verify API key in `.env` file

---

### Terminal 3: Start Frontend (Windows PowerShell)

Open **PowerShell** in Windows (not WSL):

```powershell
cd "d:\3rd year\os project\IntelliShell\dashboard"

# First time only: install dependencies
npm install

# Run dev server
npm run dev
```

**What to look for:**
- Should print: `VITE v5.x.x ready in XXX ms`
- Should print: `➜  Local:   http://localhost:5173/`

**If it fails:**
- Check Node.js: `node --version` (need 18+)
- Delete `node_modules` and run `npm install` again

---

## Part 3: Quick Health Check

Open a **4th terminal** (WSL or PowerShell) and run:

```bash
# Check Rust server
curl http://localhost:3000/health

# Check Python AI service
curl http://localhost:8000/health

# Expected response from both:
{"status":"healthy"}
```

If both return `{"status":"healthy"}`, you're ready to test! 🎉

---

## Part 4: Testing the Application - What to Check

### 🖥️ Open the Application

1. Open your web browser
2. Go to: **http://localhost:5173**
3. You should see the IntelliShell dashboard with Swiss design (brown/terracotta colors)

---

### ✅ Test 1: Overview Page (System Monitoring)

**Where**: Main page when you open the app

**What to check:**

1. **CPU Chart** (top left):
   - Should show a line graph
   - Percentage should be between 0-100%
   - Updates every second with smooth animation

2. **Memory Chart** (top right):
   - Shows RAM usage as percentage
   - Should show realistic value (probably 50-80%)

3. **Disk Usage** (bottom left):
   - Shows disk usage percentage
   - Should match your actual disk usage (use `df -h` in WSL to compare)

4. **Network Activity** (bottom right):
   - Shows RX/TX bytes
   - Numbers increase when you download/upload

**What SUCCESS looks like:**
- All 4 charts visible with real data
- Numbers changing over time
- No error messages

**If you see problems:**
- Charts showing 0 or NaN → Rust server not responding
- Charts not updating → WebSocket issue, refresh page
- Missing charts → Frontend build issue

---

### ✅ Test 2: Command Generation & Execution (Most Important!)

**Where**: Click **"Assistant"** in left sidebar

**Test 2A: Safe Command (LOW Risk)**

1. In the text box, type: `list all files in the current directory`
2. Click **"Generate Command"** button
3. Wait 2-3 seconds

**What to check:**
- Should show: `Generated Command: ls -la` (or similar)
- Should show: `Risk Level: LOW` with green badge
- Should show: **"Execute Command"** button appears

4. Click **"Execute Command"** button
5. Wait 1-2 seconds

**What to check:**
- Should show: `✅ Execution Successful` (green badge)
- Should show: `Method: direct`
- Should show: `Exit Code: 0`
- Should show: `stdout:` with actual file listing
- Should show: `Duration: XXXms`

**What SUCCESS looks like:**
```
✅ Execution Successful

Method: direct
Message: Command executed successfully
Exit Code: 0

stdout:
total 48
drwxr-xr-x 12 user user 4096 Oct 21 10:00 .
drwxr-xr-x  8 user user 4096 Oct 21 09:00 ..
-rw-r--r--  1 user user  256 Oct 21 10:00 .env
...

Duration: 245ms
```

---

**Test 2B: Dangerous Command (HIGH Risk - Should Block)**

1. Clear the previous result
2. In the text box, type: `delete everything in my home folder`
3. Click **"Generate Command"** button

**What to check:**
- Might show: `Generated Command: rm -rf ~` or similar
- Should show: `Risk Level: HIGH` or `CRITICAL` with red badge

4. Click **"Execute Command"** button

**What to check:**
- Should show: `🚫 Execution Blocked` (red badge)
- Should show: `Method: blocked`
- Should show message explaining why it was blocked
- Command should NOT actually execute!

**What SUCCESS looks like:**
```
🚫 Execution Blocked

Method: blocked
Message: Command blocked due to HIGH risk level. This command could cause system damage.

This is a safety feature - your system is protected! ✅
```

---

**Test 2C: Medium Risk Command (Sandbox or Graceful Degradation)**

1. Clear the previous result
2. Type: `kill a process with ID 12345`
3. Click **"Generate Command"**
4. Click **"Execute Command"**

**What to check:**
- Should show: `Risk Level: MEDIUM`
- Execution result shows either:
  - `Method: sandboxed` (if running as root) OR
  - `Method: direct` (graceful degradation if not root)
- Should execute but process doesn't exist (that's fine, just testing routing)

---

### ✅ Test 3: Command History

**Where**: Click **"History"** in left sidebar

**What to check:**

1. Should see a table with columns:
   - Command
   - Status
   - Exit Code
   - Timestamp

2. Should see your previous commands:
   - The `ls -la` command with status "completed"
   - The blocked dangerous command with status "blocked"

3. Click **"Refresh"** button:
   - History should reload
   - Same commands should still be there

**What SUCCESS looks like:**
- All executed commands appear in history
- Blocked commands also appear with "blocked" status
- Timestamps are correct
- Can scroll through history

**Test History Persistence:**

1. Go back to Terminal 1 (Rust server)
2. Press `Ctrl+C` to stop the server
3. Run `cargo run --release` again to restart
4. Go back to History page in browser
5. Click "Refresh"

**What to check:**
- History should still be there after restart! ✅
- This proves DEF-001 (history persistence) is fixed

---

### ✅ Test 4: Processes Page

**Where**: Click **"Processes"** in left sidebar

**What to check:**

1. Should see a table of running processes
2. Each process shows:
   - PID (process ID)
   - Name
   - CPU usage
   - Memory usage
   - Status

3. Try searching:
   - Type "node" in search box
   - Should filter to show only Node.js processes

4. Try sorting:
   - Click column headers to sort by PID, CPU, Memory

**What SUCCESS looks like:**
- Shows real system processes
- Processes update periodically
- Search and sort work properly

---

### ✅ Test 5: Monitoring Page

**Where**: Click **"Monitoring"** in left sidebar

**What to check:**

This is like Overview but with more detail:

1. **System Metrics Cards** (top):
   - CPU percentage
   - Memory usage (used/total in GB)
   - Disk usage (used/total in GB)
   - Network stats

2. **Charts** (below):
   - Same 4 charts as Overview page
   - Should show last 60 seconds of data
   - Auto-updates every second

**What SUCCESS looks like:**
- All metrics show real data
- Charts animate smoothly
- Numbers match what you see in Task Manager (Windows) or `top` (WSL)

---

### ✅ Test 6: Settings Page

**Where**: Click **"Settings"** in left sidebar

**What to check:**

1. Shows configuration sections:
   - System settings
   - API settings
   - UI preferences

2. Check current values:
   - AI Provider: Should show "Groq"
   - Risk threshold settings

**Note**: Settings changes might not persist (this is a beta limitation - documented in Phase 8 report)

---

## Part 5: Verify All 7 Mandatory Features

Use this checklist to confirm everything works:

- [ ] **Feature 1**: Command execution routing works (LOW → execute, HIGH → block)
- [ ] **Feature 2**: Command history persists after server restart
- [ ] **Feature 3**: AI risk classification works (tested all 5 risk levels)
- [ ] **Feature 4**: Disk monitoring shows real data
- [ ] **Feature 5**: Network monitoring shows RX/TX bytes
- [ ] **Feature 6**: CPU monitoring shows accurate percentage
- [ ] **Feature 7**: Execute button works and shows results

---

## Part 6: Common Issues & Solutions

### Issue: "Connection refused" on http://localhost:3000

**Solution:**
- Check Terminal 1 - is Rust server running?
- Try: `curl http://localhost:3000/health`
- Restart Rust server: `Ctrl+C` then `cargo run --release`

---

### Issue: "Generate Command" button does nothing

**Solution:**
- Check Terminal 2 - is Python AI service running?
- Check browser console (F12) for errors
- Verify Groq API key in `ai-service/.env`
- Test AI service directly: `curl http://localhost:8000/health`

---

### Issue: Charts show 0 or NaN

**Solution:**
- This means Rust server isn't providing telemetry
- Check `/snapshot` endpoint: `curl http://localhost:3000/snapshot`
- If empty, DEF-002/DEF-003/DEF-004 fixes might not be compiled
- Rebuild: `cd core-engine && cargo clean && cargo build --release`

---

### Issue: Execute button doesn't appear

**Solution:**
- This means DEF-005 fix not applied
- Check `dashboard/src/pages/AssistantPage.jsx` has the Execute button code
- Rebuild frontend: `cd dashboard && npm run dev`
- Clear browser cache (Ctrl+Shift+R)

---

### Issue: History is empty after restart

**Solution:**
- This means DEF-001 fix not working
- Check database exists: `ls core-engine/intellishell.db`
- Check database has data: `sqlite3 core-engine/intellishell.db "SELECT * FROM command_history;"`
- If empty, history persistence isn't wired - check `server.rs` has `db.insert_command()` calls

---

## Part 7: Performance Expectations

**Normal Behavior:**

- **Command generation**: 1-3 seconds (Groq API call)
- **Command execution**: 50-500ms (depending on command)
- **History load**: <100ms
- **Telemetry update**: Every 1 second
- **Page load**: <1 second

**If slower:**
- Groq API might be slow (try different model)
- Rust debug mode is slow (use `--release`)
- Network latency (check internet connection)

---

## Part 8: Screenshots to Take (Optional)

If you want to document your testing:

1. **Overview page** - showing all 4 charts with real data
2. **Assistant page** - showing successful command execution with stdout
3. **Assistant page** - showing blocked dangerous command
4. **History page** - showing command history table
5. **Processes page** - showing process list

---

## Summary: What PASS Looks Like

✅ **All 3 servers running** without errors  
✅ **All pages load** and show data  
✅ **Commands generate** using AI  
✅ **Safe commands execute** and show output  
✅ **Dangerous commands block** with safety message  
✅ **History persists** after server restart  
✅ **Charts update** every second with real data  
✅ **No crashes** or unhandled errors  

If you see all of these, **congratulations! Your IntelliShell is working perfectly!** 🎉

---

## Quick Start Commands (Copy-Paste)

### WSL Terminal 1 (Rust):
```bash
cd /mnt/d/3rd\ year/os\ project/IntelliShell/core-engine && cargo run --release
```

### WSL Terminal 2 (Python):
```bash
cd /mnt/d/3rd\ year/os\ project/IntelliShell/ai-service && uv run uvicorn intellishell_ai_service.main:app --host 127.0.0.1 --port 8000
```

### PowerShell Terminal 3 (Frontend):
```powershell
cd "d:\3rd year\os project\IntelliShell\dashboard"; npm run dev
```

### Open Browser:
```
http://localhost:5173
```

---

**Need Help?** Check the logs in each terminal for error messages.

**Ready to test?** Follow Part 2 → Part 3 → Part 4 in order!

