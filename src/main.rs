mod fetch;

use nix::unistd::geteuid;
use std::{fs, io::Write, os::unix::fs::OpenOptionsExt, path::Path};
use tempfile::NamedTempFile;

const PREPEND_HOSTS: &str = r#"127.0.0.1       localhost
255.255.255.255 broadcasthost
::1             localhost"#;

const WORK_DIR: &str = "/tmp/adobe_hosts_blocker";

const PREPEND_HOSTS_PATH: &str = "/tmp/adobe_hosts_blocker/prepend_hosts";

const HOST_FILE: &str = "/etc/hosts";
const TMP_HOSTS_FILE: &str = "/tmp/adobe_hosts_blocker/adobe_block_list.txt";

const BACKUP_TMP_HOSTS_FILE: &str = "/tmp/adobe_hosts_blocker/adobe_block_list.txt.bak";
const BACKUP_HOSTS_FILE: &str = "/tmp/adobe_hosts_blocker/hosts.bak";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !geteuid().is_root() {
        eprintln!("run this script with sudo");
        std::process::exit(1);
    }

    fs::create_dir_all(WORK_DIR)?;

    if !Path::new(PREPEND_HOSTS_PATH).exists() {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o644)
            .open(PREPEND_HOSTS_PATH)?;
        file.write_all(PREPEND_HOSTS.as_bytes())?;
    }

    let prepend: String = fs::read_to_string(PREPEND_HOSTS_PATH)?;

    // fetch list.txt from https://a.dove.isdumb.one/list.txt
    let fetched_list: String = fetch::fetch_list()?;

    if Path::new(TMP_HOSTS_FILE).exists() {
        let current_list: String = fs::read_to_string(TMP_HOSTS_FILE)?;
        if current_list == fetched_list {
            return Ok(());
        }
    }

    if Path::new(HOST_FILE).exists() {
        fs::copy(HOST_FILE, BACKUP_HOSTS_FILE)?;
    }

    if Path::new(TMP_HOSTS_FILE).exists() {
        fs::copy(TMP_HOSTS_FILE, BACKUP_TMP_HOSTS_FILE)?;
    }

    fs::write(TMP_HOSTS_FILE, &fetched_list)?;

    atomic_write_hosts(&prepend, &fetched_list)?;

    flush_dns_cache()?;

    Ok(())
}

fn atomic_write_hosts(prepend: &str, list: &str) -> std::io::Result<()> {
    let mut temp_file = NamedTempFile::new_in("/etc")?;

    temp_file.write_all(prepend.as_bytes())?;
    temp_file.write_all(b"\n\n")?;
    temp_file.write_all(list.as_bytes())?;

    temp_file.as_file().sync_all()?;

    temp_file.persist(HOST_FILE)?;

    let dir = fs::File::open("/etc")?;
    dir.sync_all()?;

    Ok(())
}

fn flush_dns_cache() -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // macOS Big Sur (11) 及更新版本
        let status = Command::new("dscacheutil").arg("-flushcache").status()?;

        if !status.success() {
            eprintln!("Warning: dscacheutil failed with status: {}", status);
        }

        // 重啟 mDNSResponder
        let status = Command::new("killall")
            .args(["-HUP", "mDNSResponder"])
            .status()?;

        if !status.success() {
            eprintln!(
                "Warning: killall mDNSResponder failed with status: {}",
                status
            );
        }

        println!("DNS cache flushed successfully");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // 嘗試 systemd-resolved (現代 Linux 發行版)
        let _ = Command::new("systemctl")
            .args(["restart", "systemd-resolved"])
            .status();

        // 嘗試 nscd (較舊的系統)
        let _ = Command::new("systemctl").args(["restart", "nscd"]).status();

        println!("DNS cache flush attempted");
    }

    Ok(())
}

// crontab every 30 minutes
//
// FLOW START
//
// if !is_root
//      FLOW END
//
// if USER_DEFINED_HOSTS exists, read it and assign to PREPEND_HOSTS
// else, write the PREPEND_HOSTS into USER_DEFINED_HOSTS
//
// fetch lists.txt
//
// if the TMP_HOSTS_FILE exists, compare the existing list.txt
//      if equals, do nothing, EARLY RETURN
//            FLOW END
//      else, backup HOST_FILE(user might use it first time),
//            overwrite into TMP_HOSTS_FILE with lists.txt
// else, write into TMP_HOSTS_FILE
//
// backup the existing HOST_FILE to BACKUP_HOSTS_FILE and TMP_HOSTS_FILE to BACKUP_TMP_HOSTS_FILE
//
// write PREPEND_HOSTS and TMP_HOSTS_FILE(which will be lists.txt in memory) into
// HOST_FILE(/etc/hosts might need privilege)
//
// syscall for clear DNS
//
// FLOW END
