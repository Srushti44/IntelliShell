use std::ffi::CString;
use std::ptr;

// Runs `command` with `args` inside a new PID + mount namespace, so it
// gets its own isolated process tree (becomes PID 1) and its own /proc
// view -- the exact effect we just proved manually with `unshare`.
//
// This forks: the PARENT waits and returns the child's exit status.
// The CHILD becomes the new namespace's init, mounts a fresh /proc,
// and execs the target command, replacing itself entirely.
pub fn run_isolated(command: &str, args: &[String]) -> Result<i32, String> {
    unsafe {
        // CLONE_NEWPID: the next fork()'d child becomes PID 1 of a new PID
        // namespace (this process itself stays in the OLD namespace, exactly
        // as we saw with the real `unshare` command).
        // CLONE_NEWNS: mount namespace, unshared immediately for THIS
        // process and inherited by the child -- needed so mounting /proc
        // below doesn't affect the real host's /proc.
        let result = libc::unshare(libc::CLONE_NEWPID | libc::CLONE_NEWNS);
        if result != 0 {
            return Err(format!(
                "unshare() failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let pid = libc::fork();
        if pid < 0 {
            return Err(format!(
                "fork() failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        if pid == 0 {
            // ---- Child: this process is PID 1 in the new namespace ----

            let proc_src = CString::new("proc").unwrap();
            let proc_target = CString::new("/proc").unwrap();
            let proc_fstype = CString::new("proc").unwrap();

            let mount_result = libc::mount(
                proc_src.as_ptr(),
                proc_target.as_ptr(),
                proc_fstype.as_ptr(),
                0,
                ptr::null(),
            );
            if mount_result != 0 {
                eprintln!(
                    "Failed to mount isolated /proc: {}",
                    std::io::Error::last_os_error()
                );
                libc::_exit(1);
            }

            // Build argv for execvp: program name first, then args, NULL-terminated
            // (the real C calling convention execvp expects).
            let c_command = CString::new(command).unwrap();
            let mut c_args: Vec<CString> = vec![c_command.clone()];
            for a in args {
                c_args.push(CString::new(a.as_str()).unwrap());
            }
            let mut argv: Vec<*const libc::c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
            argv.push(ptr::null());

            libc::execvp(c_command.as_ptr(), argv.as_ptr());

            // execvp only returns if it FAILED (on success, this process
            // image is replaced entirely and never comes back here).
            eprintln!("execvp failed: {}", std::io::Error::last_os_error());
            libc::_exit(1);
        }

        // ---- Parent: wait for the isolated child to finish ----
        let mut status: libc::c_int = 0;
        libc::waitpid(pid, &mut status, 0);

        if libc::WIFEXITED(status) {
            Ok(libc::WEXITSTATUS(status))
        } else {
            Ok(-1)
        }
    }
}
