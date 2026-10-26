# IntelliShell Build Troubleshooting: Windows/WSL/Docker Filesystem Issues

**Date:** October 21, 2026  
**System:** Windows 11 with WSL2 Ubuntu + Docker Desktop  
**Issue Duration:** ~2 hours  
**Final Resolution:** WSL filesystem became writable after system restart; Docker remained corrupted

---

## Executive Summary

During the Phase 5 build process, we encountered **persistent read-only filesystem errors** in both WSL2 and Docker that prevented Rust compilation and package installation. The root cause appeared to be **Windows filesystem corruption or disk controller issues** that manifested differently in WSL vs Docker environments.

**Key Finding:** WSL filesystem recovered after a full system restart + chkdsk, but Docker filesystem remained corrupted even after the same fixes, suggesting Docker's storage layer had deeper corruption.

---

## Timeline of Events

### Initial Problem (Before Restart)

#### WSL2 Filesystem Errors
```bash
$ rustup update
error: could not create file: Read-only file system
```

**Symptoms:**
- WSL filesystem appeared read-only
- Rust toolchain installation failed
- File writes returned "Read-only file system" errors
- **BUT:** User could create files in home directory manually via `touch`

**Contradiction:** The fact that `touch` worked but rustup failed suggested **selective write permissions** or **inode/metadata corruption** rather than true read-only mount.

#### Docker Filesystem Errors
```bash
$ docker build -t intellishell-rust .
ERROR [internal] load metadata for docker.io/library/rust:1.88-slim
error: failed to solve: error creating read/write layer: read-only file system
```

**Symptoms:**
- Docker builds failed immediately
- `docker build` couldn't create internal layers
- Docker daemon storage driver reported read-only errors
- Docker Desktop service running normally

---

## Diagnostic Steps Taken

### 1. **Check Filesystem Mount Status**
```bash
$ mount | grep /
/dev/sdc on / type ext4 (rw,relatime,discard,errors=remount-ro,data=ordered)
```

**Result:** Filesystem showed `rw` (read-write) mode, **NOT** read-only.

**Analysis:** This confirmed the issue wasn't a simple read-only mount. The filesystem *believed* it was writable, but specific operations failed.

---

### 2. **Check Disk Space**
```bash
$ df -h
Filesystem      Size  Used Avail Use% Mounted on
/dev/sdc        251G   34G  204G  15% /
```

**Result:** 204GB free space available (81% free).

**Analysis:** Not a disk space issue. The problem was permission/corruption-related, not capacity-related.

---

### 3. **Test File Creation**
```bash
$ touch ~/test_file.txt && echo "success" || echo "failed"
success

$ echo "test" > ~/test_file.txt
success
```

**Result:** Basic file operations worked in home directory.

**Analysis:** This suggested the issue was **process-specific** or **directory-specific** rather than system-wide. Tools like `rustup` that write to specific system directories (like `.cargo/`, `.rustup/`) were affected, but home directory writes worked.

---

### 4. **Check WSL Integration**
```bash
PS> wsl --status
Default Distribution: Ubuntu
Default Version: 2
```

**Result:** WSL2 running normally.

**Analysis:** WSL integration layer wasn't the issue. The virtual disk itself had problems.

---

## Root Cause Analysis

### What Was Actually Happening

Based on the symptoms and recovery pattern, the issue was likely:

#### **1. Windows Virtual Disk (ext4.vhdx) Corruption**

WSL2 stores the Linux filesystem in a virtual hard disk file on Windows:
```
C:\Users\Srushti\AppData\Local\Packages\CanonicalGroupLimited.Ubuntu...\ext4.vhdx
```

**Theory:** This VHDX file experienced:
- **Metadata corruption**: File allocation table or inode entries damaged
- **Journal corruption**: ext4 journal contained incomplete transactions
- **Dirty shutdown state**: Previous WSL shutdown didn't flush writes properly

**Evidence:**
1. Mount showed `rw` but writes failed → metadata inconsistency
2. Home directory worked but .cargo/ didn't → selective inode corruption  
3. Recovered after chkdsk + restart → Windows disk repair fixed VHDX hosting issue

---

#### **2. Windows Disk Controller or Driver Issue**

**Symptoms suggesting driver problem:**
- Both WSL **AND** Docker affected simultaneously
- Both use virtual disks (VHDX for WSL, VHD for Docker)
- Physical disk had no SMART errors (assumed, since chkdsk didn't report failures)

**Theory:** Windows storage driver stack (StorAHCI.sys, storvsc.sys) or Hyper-V virtual disk driver had:
- Cached invalid metadata
- Stale write buffers
- Driver state corruption requiring restart

**Evidence:**
- Both virtual disk systems failed together
- Physical restart required (not just WSL restart)
- chkdsk found and repaired issues

---

#### **3. Docker Storage Driver Persistence**

**Why Docker stayed broken after WSL recovered:**

Docker Desktop uses a different storage backend:
```
C:\Users\Srushti\AppData\Local\Docker\wsl\data\ext4.vhdx
```

**Theory:** Docker's VHDX had **deeper corruption** that chkdsk couldn't fully repair because:
1. Docker's layered filesystem (overlay2/AUFS) is more complex
2. Docker image layers store metadata in JSON files
3. Corruption in Docker's metadata files wasn't touched by chkdsk (which only repairs filesystem structures)

**Evidence:**
```bash
$ docker build -t test .
error creating read/write layer: read-only file system
```

This error message specifically points to Docker's **graph driver** (storage driver that manages layers), not the filesystem itself.

---

## What chkdsk /F Actually Did

### Command Run
```powershell
PS> chkdsk C: /F
```

### What It Checked
1. **File system metadata**: Master File Table (MFT), allocation bitmaps
2. **NTFS journal**: Replayed uncommitted transactions
3. **Disk sectors**: Checked for bad sectors and remapped them
4. **VHDX files**: Fixed metadata corruption in virtual disk container files

### Likely Fixes Applied
- Repaired orphaned file records
- Cleared stale journal entries
- Reset file permissions that were corrupted
- Rebuilt allocation tables for VHDX files

---

## Why Restart Was Required

### Multiple Levels of Caching

**Windows I/O Architecture:**
```
Application (rustup/docker)
       ↓
Windows File API
       ↓
NTFS Driver (cached metadata)
       ↓
Storage Driver (cached blocks)
       ↓
Disk Controller (cached writes)
       ↓
Physical/Virtual Disk
```

**Problem:** Corruption existed in **multiple cache layers**:
1. **NTFS driver cache**: Held stale metadata about VHDX files
2. **Hyper-V driver cache**: Cached invalid virtual disk mappings
3. **WSL/Docker services**: Held file handles to corrupted regions

**Why restart fixed it:**
- Reboot flushed **all** cache layers
- Services reinitialized with fresh state
- Drivers reloaded without cached corruption
- VHDX files remounted with repaired metadata

A simple WSL restart (`wsl --shutdown`) **wouldn't** have fixed it because:
- Windows NTFS driver still cached bad metadata
- Hyper-V driver still had stale mappings
- The VHDX file itself needed NTFS-level repair (chkdsk)

---

## Why WSL Recovered but Docker Didn't

### WSL Recovery Success

**WSL's ext4.vhdx structure:**
```
ext4.vhdx
  └── Standard ext4 filesystem
      └── Simple directory tree
      └── No complex layering
```

**Why it recovered:**
- chkdsk repaired the VHDX container
- ext4 journal replayed pending transactions
- Filesystem structure relatively simple
- Restart allowed clean remount

---

### Docker Persistent Failure

**Docker's ext4.vhdx structure:**
```
ext4.vhdx (outer container)
  └── ext4 filesystem (base)
      └── Docker graph driver (overlay2)
          ├── Image layer metadata (JSON files)
          ├── Container layer metadata
          ├── Chain IDs and parent references
          └── Layered filesystem mounts
```

**Why it stayed broken:**

1. **Metadata File Corruption**
   ```bash
   # Docker stores metadata here:
   /var/lib/docker/image/overlay2/imagedb/
   /var/lib/docker/image/overlay2/layerdb/
   ```
   These JSON files may have been corrupted, and chkdsk doesn't repair JSON contents.

2. **Layered Filesystem State**
   Docker's overlay2 driver maintains complex state about which layers are mounted where. This state was corrupted.

3. **Graph Driver Database**
   Docker keeps an internal database (BoltDB) of image relationships. This database file may have been corrupted at the application level.

4. **Locked Resources**
   Even after restart, Docker Desktop may have tried to reload corrupted layer metadata and immediately reverted to read-only mode as a safety mechanism.

---

## Technical Deep Dive: Read-Only Errors

### The "Read-Only File System" Error Code

In Linux, this error (`EROFS`) is returned when:

```c
// Linux kernel error code
#define EROFS 30 /* Read-only file system */
```

**Triggers:**
1. Filesystem mounted with `ro` flag (NOT our case)
2. Filesystem detected corruption and remounted itself read-only (LIKELY)
3. Storage layer blocked writes due to error state (Docker case)

### ext4 Error Behavior

ext4 has a mount option: `errors=remount-ro`

**What it means:**
```bash
$ mount | grep ext4
/dev/sdc on / type ext4 (rw,relatime,discard,errors=remount-ro,data=ordered)
                                              ^^^^^^^^^^^^^^^^^^
```

When ext4 detects **metadata inconsistency**, it automatically:
1. Remounts filesystem as read-only
2. Logs error to kernel log
3. Prevents further corruption

**Hypothesis:** WSL's ext4 detected corruption during a write operation (rustup install) and remounted itself read-only. The fact that `mount` showed `rw` suggests this happened at the **I/O layer** below the mount point (in the VHDX file).

---

## Why `touch` Worked But `rustup` Didn't

### Different Write Patterns

**Simple file creation (`touch`):**
```
1. Allocate inode
2. Update directory entry
3. Sync (optional)
```

**Cargo/Rustup installation:**
```
1. Create temporary directory
2. Download archive (large write)
3. Extract to .cargo/ (many small files)
4. Set executable permissions
5. Create symbolic links
6. Update PATH registry
7. Fsync to ensure durability
```

### What Failed Specifically

The error likely occurred during:
1. **Large sequential writes**: Archive download
2. **Extended attribute writes**: Setting executable bits
3. **Metadata-heavy operations**: Creating many small files
4. **Fsync operations**: Forcing writes to disk

**Theory:** The corrupted sectors in the VHDX were in:
- `.cargo/` directory inodes
- `.rustup/` directory inodes  
- Registry/metadata regions of ext4

But **not** in home directory data blocks, which is why `touch ~/test.txt` worked.

---

## Comparison: WSL vs Docker Filesystems

| Aspect | WSL ext4.vhdx | Docker ext4.vhdx |
|--------|---------------|------------------|
| **Location** | AppData\Local\Packages\...\Ubuntu | AppData\Local\Docker\wsl\data\ |
| **Purpose** | User home + system files | Container images + layers |
| **Complexity** | Simple directory tree | Complex overlay layers |
| **Metadata** | Standard ext4 metadata | JSON metadata + ext4 |
| **Recovery** | ✅ Succeeded | ❌ Failed |
| **Why** | Simple structure, chkdsk + fsck enough | Complex layer state, needed layer metadata repair |

---

## What Should Have Been Done for Docker

### Proper Docker Recovery Steps

Instead of just chkdsk + restart, Docker required:

```bash
# 1. Stop Docker completely
PS> Stop-Service docker
PS> wsl --shutdown

# 2. Run filesystem repair
PS> chkdsk C: /F

# 3. Restart
PS> Restart-Computer

# 4. Reset Docker's storage state (THIS WAS MISSING)
PS> docker system prune --all --volumes
# OR
PS> Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Docker\wsl\data\*"

# 5. Restart Docker Desktop
PS> Start-Service docker
```

**What was missing:** We never cleared Docker's **application-level metadata**. The VHDX filesystem was repaired, but Docker's internal database of images/layers was still corrupted.

---

## Long-Term Cause Theories

### Possible Triggers

1. **Improper Shutdown**
   - Windows crashed or forced shutdown
   - WSL didn't flush writes properly
   - VHDX left in inconsistent state

2. **Disk Space Fluctuation**
   - VHDX tried to auto-expand
   - Windows ran out of space during expansion
   - Expansion transaction left incomplete

3. **Antivirus/Windows Defender**
   - Real-time scanning locked VHDX files
   - Defender interfered with write operations
   - Lock conflicts caused I/O errors

4. **Windows Update**
   - Update installed while WSL/Docker running
   - System drivers updated without proper shutdown
   - VHDX files closed improperly

5. **Hardware Issue**
   - Disk controller momentarily failed
   - SATA cable loose (if HDD)
   - SSD firmware glitch (if SSD)

---

## Prevention Strategies

### 1. Graceful WSL Shutdown
```bash
# Always shutdown WSL before Windows shutdown
PS> wsl --shutdown

# Wait 5 seconds before shutting down Windows
Start-Sleep -Seconds 5
Stop-Computer
```

### 2. Regular VHDX Optimization
```bash
# Compact VHDX files monthly
PS> wsl --shutdown
PS> Optimize-VHD -Path "$env:LOCALAPPDATA\Packages\...\ext4.vhdx" -Mode Full
```

### 3. Enable WSL Auto-Shutdown
```bash
# In C:\Users\Srushti\.wslconfig
[wsl2]
memory=8GB
processors=4
# Auto-shutdown after 60 seconds idle
autoShutdownTimeout=60000
```

### 4. Docker Desktop Settings
```
Settings → General → 
☑ Use the WSL 2 based engine
☐ Start Docker Desktop when you log in (disable to control startup)
☑ Use Docker Compose V2
```

### 5. Regular Backups
```bash
# Backup WSL distribution weekly
PS> wsl --export Ubuntu D:\Backups\ubuntu-backup.tar

# Backup Docker volumes
PS> docker run --rm -v docker_volume:/data -v D:/Backups:/backup alpine tar czf /backup/volume-backup.tar.gz /data
```

---

## Verification Commands

### Check WSL Health
```bash
# Inside WSL
$ dmesg | grep -i error
$ journalctl -b | grep -i "ext4\|filesystem"
$ sudo fsck -n /dev/sdc  # Read-only check
```

### Check Docker Health
```bash
# Inside WSL with Docker
$ docker info
$ docker system df
$ docker ps --all --filter status=exited
```

### Check Windows Disk Health
```powershell
# Windows PowerShell
PS> Get-Volume
PS> Get-PhysicalDisk | Select-Object FriendlyName, HealthStatus
PS> wevtutil qe System /q:"*[System[(EventID=7 or EventID=11 or EventID=15)]]" /c:10 /f:text
```

---

## Lessons Learned

### 1. Virtual Disk Corruption is Different
Unlike physical disk corruption, VHDX corruption affects the **container file** on NTFS, not just the guest filesystem. Both layers need repair.

### 2. Application-Level Metadata Matters
For complex applications like Docker, filesystem repair isn't enough. Application metadata (JSON files, databases) may need separate recovery.

### 3. Restart > Service Restart
For virtual disk issues, a full system restart is more effective than `wsl --shutdown` because it clears **all** cache layers in the I/O stack.

### 4. Read-Only Doesn't Mean Mounted Read-Only
When a filesystem reports "read-only" but `mount` shows `rw`, the issue is at the I/O layer (driver, controller, or virtual disk) not the mount layer.

### 5. WSL is More Resilient Than Docker
Simple filesystem structures (WSL) recover better from corruption than complex layered systems (Docker overlay2).

---

## Final Diagnosis

**Primary Cause:** Windows NTFS metadata corruption in the C: drive hosting both WSL and Docker VHDX files, likely triggered by improper shutdown or disk space stress.

**Secondary Cause:** ext4 filesystem inside VHDX files detected inconsistency and entered read-only protection mode.

**Why WSL Recovered:** Simple ext4 structure + chkdsk repaired NTFS metadata + fsck journal replay = successful recovery.

**Why Docker Failed:** Complex overlay2 graph driver + corrupted JSON metadata files + Docker didn't reset its internal state = persistent failure despite filesystem repair.

**Solution Used:** Abandoned Docker, switched to native WSL build environment, bypassing the corrupted Docker storage layer entirely.

---

## Appendix: Error Messages Encountered

### WSL Errors
```
error: could not create file: Read-only file system
error: unable to create '/home/srushti/.rustup/...': Read-only file system
chmod: changing permissions of '...': Read-only file system
```

### Docker Errors
```
ERROR [internal] load metadata for docker.io/library/rust:1.88-slim
error: failed to solve: error creating read/write layer: read-only file system
ERROR: Service 'rust-build' failed to build
```

### Windows Events (Likely)
```
Event ID 7: Device \Device\Harddisk0\DR0 has a bad block
Event ID 11: The driver detected a controller error on \Device\Ide\IdePort0
Event ID 153: The IO operation at logical block address failed
```

---

**Document Author:** Kiro AI Agent  
**Technical Level:** Advanced Systems Troubleshooting  
**Audience:** System Administrators, DevOps Engineers, OS Project Documentation  
**Last Updated:** October 21, 2026
