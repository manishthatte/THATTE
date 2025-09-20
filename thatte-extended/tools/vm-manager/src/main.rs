use std::path::PathBuf;
use std::process::Command;
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Cfg {
    #[serde(default)]
    kvm: Kvm,
    machine: Machine,
    paths: Paths,
    #[serde(default)]
    network: Network,
}

#[derive(Debug, Default, Deserialize)]
struct Kvm {
    #[serde(default = "default_true")]
    use_kvm_if_available: bool,
}
fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
struct Machine {
    memory_mb: u64,
    cpus: u32,
}

#[derive(Debug, Deserialize)]
struct Paths {
    kernel: PathBuf,
    initrd: PathBuf,
    disk: PathBuf,
    #[serde(default = "default_share")]
    share_dir: PathBuf,
}
fn default_share() -> PathBuf { PathBuf::from(".") }

#[derive(Debug, Default, Deserialize)]
struct Network {
    #[serde(default = "default_ssh")]
    host_ssh_forward: u16,
}
fn default_ssh() -> u16 { 2222 }

#[derive(Parser, Debug)]
#[command(name = "vm-manager", version)]
struct Cli {
    /// Path to config (TOML)
    #[arg(long, default_value = "configs/driveros.toml")]
    cfg: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Validate configuration
    Check,
    /// Run the VM (executes qemu-system-x86_64)
    Run,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg_text = std::fs::read_to_string(&cli.cfg)
        .with_context(|| format!("reading {}", cli.cfg.display()))?;
    let cfg: Cfg = toml::from_str(&cfg_text).context("parsing config")?;

    match cli.cmd {
        Cmd::Check => check(&cfg),
        Cmd::Run => run(&cfg),
    }
}

fn check(cfg: &Cfg) -> Result<()> {
    for p in [&cfg.paths.kernel, &cfg.paths.initrd, &cfg.paths.disk] {
        if !p.exists() { bail!("missing {}", p.display()); }
    }
    println!("OK: config and files exist.");
    Ok(())
}

fn run(cfg: &Cfg) -> Result<()> {
    check(cfg)?;
    let accel = if cfg.kvm.use_kvm_if_available && std::fs::metadata("/dev/kvm").is_ok() {
        "kvm:tcg"
    } else {
        "tcg"
    };
    let status = Command::new("qemu-system-x86_64")
        .args([
            "-machine", &format!("q35,accel={}", accel),
            "-cpu", "host",
            "-m", &cfg.machine.memory_mb.to_string(),
            "-smp", &cfg.machine.cpus.to_string(),
            "-kernel", &cfg.paths.kernel.display().to_string(),
            "-initrd", &cfg.paths.initrd.display().to_string(),
            "-append", "console=ttyS0 root=/dev/vda1 rw quiet",
            "-drive", &format!("file={},if=virtio,format=raw", cfg.paths.disk.display()),
            "-vga", "std",
            "-serial", "stdio",
            "-fsdev", &format!("local,id=fsdev0,path={},security_model=none", cfg.paths.share_dir.display()),
            "-device", "virtio-9p-pci,fsdev=fsdev0,mount_tag=hostshare",
            "-device", &format!("virtio-net-pci,netdev=n0"),
            "-netdev", &format!("user,id=n0,hostfwd=tcp::{}-:22", cfg.network.host_ssh_forward),
        ])
        .status()
        .context("spawning qemu-system-x86_64")?;
    if !status.success() {
        bail!("qemu exited with {}", status);
    }
    Ok(())
}
