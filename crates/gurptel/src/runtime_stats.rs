use std::fs;
use std::mem;

#[repr(C)]
struct PsInfo {
    pr_flag: i32,
    pr_nlwp: i32,
    pr_pid: i32,
    pr_ppid: i32,
    pr_pgid: i32,
    pr_sid: i32,
    pr_uid: u32,
    pr_euid: u32,
    pr_gid: u32,
    pr_egid: u32,
    pr_addr: usize,
    pr_size: usize,   // VM size in KB
    pr_rssize: usize, // RSS in KB
}

pub fn rss_bytes() -> Option<usize> {
    let bytes = fs::read("/proc/self/psinfo").ok()?;
    if bytes.len() < mem::size_of::<PsInfo>() {
        return None;
    }
    let info: PsInfo = unsafe { bytes.as_ptr().cast::<PsInfo>().read_unaligned() };
    Some(info.pr_rssize * 1024)
}
