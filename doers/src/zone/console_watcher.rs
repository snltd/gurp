use crate::zone::constants::READINESS_WAIT_TIMEOUT_EMULATED;
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use common::constants::ZLOGIN_BIN;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

const BUFFER_SIZE: usize = 8192;
const WINDOW_SIZE: usize = 4096;

pub fn wait_for_readiness(zone: &str, uuid: &Uuid) -> anyhow::Result<bool> {
    tracing::info!("Waiting for zone '{zone}' to be ready");

    let console_log_dir = Utf8PathBuf::from("/var/tmp");
    let console_log_file = console_log_dir.join(format!("gurp-bhyve-{uuid}.log"));

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize::default())
        .context("Failed to create PTY")?;

    let mut cmd = CommandBuilder::new(ZLOGIN_BIN);
    cmd.arg("-C");
    cmd.arg(zone);

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to start zlogin")?;

    drop(pair.slave);

    let writer = pair
        .master
        .take_writer()
        .context("Failed to get PTY writer")?;

    let reader = pair
        .master
        .try_clone_reader()
        .context("Failed to get PTY reader")?;

    let (result_tx, result_rx) = mpsc::channel();

    tracing::info!("Logging console output to {console_log_file}");

    thread::spawn(move || {
        let result = monitor_console(reader, writer, &console_log_file);
        let _ = result_tx.send(result);
    });

    let result = result_rx.recv_timeout(READINESS_WAIT_TIMEOUT_EMULATED)?;
    let _ = child.kill();
    let _ = child.wait();

    result
}

fn monitor_console(
    mut reader: Box<dyn Read + Send>,
    mut writer: Box<dyn Write + Send>,
    log_path: &Utf8PathBuf,
) -> anyhow::Result<bool> {
    let mut buf = vec![0u8; BUFFER_SIZE];
    let mut window = Vec::new();
    let mut cr_sent = false;
    let mut login_prompt_seen = false;

    let log_file =
        File::create(log_path).with_context(|| format!("failed to open +w log at {log_path}"))?;
    let mut log_writer = BufWriter::new(log_file);

    let start_time = Instant::now();

    while start_time.elapsed() < READINESS_WAIT_TIMEOUT_EMULATED {
        match reader.read(&mut buf) {
            Ok(0) => {
                tracing::debug!("EOF reached on console");
                break;
            }
            Ok(n) => {
                let data = &buf[..n];

                if let Err(e) = log_writer.write_all(data) {
                    tracing::warn!("Failed to write to console log: {}", e);
                } else {
                    let _ = log_writer.flush();
                }

                if !data.is_empty() {
                    let preview = if data.len() > 100 { &data[..100] } else { data };
                    let preview_str = String::from_utf8_lossy(preview);
                    tracing::debug!("Console data: {:?}", preview_str);
                }

                window.extend_from_slice(data);
                if window.len() > WINDOW_SIZE {
                    window.drain(..window.len() - WINDOW_SIZE);
                }

                if let Some(result) =
                    scan_output(&window, &mut writer, &mut cr_sent, &mut login_prompt_seen)
                {
                    let _ = log_writer.flush();
                    return Ok(result);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) => {
                bail!("Read error: {}", e);
            }
        }

        thread::sleep(Duration::from_millis(50));
    }

    let _ = log_writer.flush();

    bail!("Timeout reached waiting for zone readiness");
}

fn scan_output(
    window: &[u8],
    writer: &mut Box<dyn Write + Send>,
    cr_sent: &mut bool,
    login_prompt_seen: &mut bool,
) -> Option<bool> {
    if find_pattern(window, b"Zone halted") {
        tracing::debug!("Zone halted detected");
        hangup(writer);
        return Some(false);
    }

    if !*cr_sent && find_pattern(window, b"ttyS0") {
        tracing::debug!("ttyS0 detected; sending CR");
        hit_return(writer);
        *cr_sent = true;
    }

    let login_patterns: &[&[u8]] = &[
        b"login:",
        b"Login:",
        b"username:",
        b"Username:",
        b"cloud-init is configuring this system",
    ];

    for pattern in login_patterns {
        if find_pattern(window, pattern) && !*login_prompt_seen {
            tracing::debug!("Login prompt detected",);
            *login_prompt_seen = true;

            thread::sleep(Duration::from_millis(500));
            hangup(writer);
            return Some(true);
        }
    }

    None
}

fn find_pattern(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }

    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn hit_return(writer: &mut Box<dyn Write + Send>) {
    let _ = writer.write_all(b"\r");
    let _ = writer.flush();
}

fn hangup(writer: &mut Box<dyn Write + Send>) {
    let _ = writer.write_all(b"~.\r");
    let _ = writer.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pattern() {
        assert!(find_pattern(b"hello world", b"world"));
        assert!(find_pattern(b"ubuntu login: ", b" login:"));
        assert!(!find_pattern(b"hello", b"world"));
        assert!(find_pattern(b"Zone halted", b"Zone halted"));
    }

    #[test]
    fn test_check_console_patterns_login() {
        let mut writer: Box<dyn Write + Send> = Box::new(Vec::new());
        let mut cr_sent = false;
        let mut login_seen = false;

        let window = b"Welcome to Linux\nubuntu login: ";
        let result = scan_output(window, &mut writer, &mut cr_sent, &mut login_seen);

        assert!(matches!(result, Some(true)));
        assert!(login_seen);
    }

    #[test]
    fn test_check_console_patterns_halted() {
        let mut writer: Box<dyn Write + Send> = Box::new(Vec::new());
        let mut cr_sent = false;
        let mut login_seen = false;

        let window = b"Shutting down\nZone halted\n";
        let result = scan_output(window, &mut writer, &mut cr_sent, &mut login_seen);

        assert!(matches!(result, Some(false)));
    }

    #[test]
    fn test_check_console_patterns_ttys0() {
        let mut writer: Box<dyn Write + Send> = Box::new(Vec::new());
        let mut cr_sent = false;
        let mut login_seen = false;

        let window = b"Starting ttyS0...";
        let result = scan_output(window, &mut writer, &mut cr_sent, &mut login_seen);

        assert!(result.is_none());
        assert!(cr_sent);
    }
}
