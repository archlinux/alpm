//! Utils for the running of rootless backends.

use std::{path::PathBuf, process::Command, str::FromStr};

use strum::EnumString;
use which::which;

use crate::Error;

/// A VM environment detected by [systemd-detect-virt].
///
/// [systemd-detect-virt]: https://man.archlinux.org/man/systemd-detect-virt.1
#[derive(Clone, Copy, Debug, strum::Display, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum SystemdDetectVirtVm {
    /// QEMU software virtualization, without KVM.
    Qemu,
    /// Linux KVM kernel virtual machine, in combination with QEMU.
    Kvm,
    /// Amazon EC2 Nitro using Linux KVM.
    Amazon,
    /// s390 z/VM.
    Zvm,
    /// VMware Workstation or Server, and related products.
    Vmware,
    /// Hyper-V, also known as Viridian or Windows Server Virtualization.
    Microsoft,
    /// Oracle VM VirtualBox, for legacy and KVM hypervisor.
    Oracle,
    /// IBM PowerVM hypervisor.
    PowerVm,
    /// Xen hypervisor (only domU, not dom0).
    Xen,
    /// Bochs Emulator.
    Bochs,
    /// User-mode Linux.
    Uml,
    /// Parallels Desktop, Parallels Server.
    Parallels,
    /// bhyve FreeBSD hypervisor.
    Bhyve,
    /// QNX hypervisor.
    Qnx,
    /// ACRN hypervisor.
    Acrn,
    /// Apple virtualization framework.
    Apple,
    /// LMHS SRE hypervisor.
    Sre,
    /// Google Compute Engine.
    Google,
}

/// A container environment detected by [systemd-detect-virt].
///
/// [systemd-detect-virt]: https://man.archlinux.org/man/systemd-detect-virt.1
#[derive(Clone, Copy, Debug, strum::Display, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum SystemdDetectVirtContainer {
    /// OpenVZ/Virtuozzo.
    OpenVc,
    /// Linux container implementation by LXC.
    Lxc,
    /// Linux container implementation by libvirt.
    #[strum(serialize = "lxc-libvirt")]
    LxcLibvirt,
    /// Systemd's minimal container implementation (systemd-nspawn).
    #[strum(serialize = "systemd-nspawn")]
    SystemdNspawn,
    /// Docker container manager.
    Docker,
    /// Podman container manager.
    Podman,
    /// Rkt app container runtime.
    Rkt,
    /// Windows Subsystem for Linux.
    Wsl,
    /// proot userspace chroot/bind mount emulation
    Proot,
    /// Pouch container engine.
    Pouch,
}

/// A confidential virtualization technology detected by [systemd-detect-virt].
///
/// [systemd-detect-virt]: https://man.archlinux.org/man/systemd-detect-virt.1
#[derive(Clone, Copy, Debug, strum::Display, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum ConfidentialVirtualizationTechnology {
    /// AMD Secure Encrypted Virtualization (x86_64).
    Sev,
    /// AMD Secure Encrypted Virtualization - Encrypted State (x86_64).
    #[strum(serialize = "sev-es")]
    SevEs,
    /// AMD Secure Encrypted Virtualization - Secure Nested Paging (x86_64).
    #[strum(serialize = "sev-snp")]
    SevSnp,
    /// Intel Trust Domain Extension (x86_64).
    Tdx,
    /// IBM Protected Virtualization (Secure Execution) (s390x).
    Protvirt,
}

/// The output of [systemd-detect-virt].
///
/// [systemd-detect-virt]: https://man.archlinux.org/man/systemd-detect-virt.1
#[derive(Clone, Copy, Debug, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum SystemdDetectVirtOutput {
    /// A confidential virtualization technology.
    #[strum(to_string = "{0}")]
    Cms(ConfidentialVirtualizationTechnology),
    /// A VM environment.
    #[strum(to_string = "{0}")]
    Vm(SystemdDetectVirtVm),
    /// A container environment.
    #[strum(to_string = "{0}")]
    Container(SystemdDetectVirtContainer),
    /// No virtual environment.
    None,
}

impl SystemdDetectVirtOutput {
    /// Evaluate whether the [`SystemdDetectVirtOutput`] requires kernel namespaces.
    pub fn uses_namespaces(&self) -> bool {
        match self {
            Self::Vm(_) | Self::Cms(_) | Self::None => false,
            Self::Container(container) => match container {
                SystemdDetectVirtContainer::OpenVc
                | SystemdDetectVirtContainer::Lxc
                | SystemdDetectVirtContainer::LxcLibvirt
                | SystemdDetectVirtContainer::SystemdNspawn
                | SystemdDetectVirtContainer::Rkt
                | SystemdDetectVirtContainer::Pouch
                | SystemdDetectVirtContainer::Docker
                | SystemdDetectVirtContainer::Podman => true,
                SystemdDetectVirtContainer::Wsl | SystemdDetectVirtContainer::Proot => false,
            },
        }
    }
}

impl FromStr for SystemdDetectVirtOutput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Strip any trailing newline
        let s = s.strip_suffix("\n").unwrap_or(s);

        Ok(
            if let Ok(cms) = ConfidentialVirtualizationTechnology::from_str(s) {
                SystemdDetectVirtOutput::Cms(cms)
            } else if let Ok(vm) = SystemdDetectVirtVm::try_from(s) {
                SystemdDetectVirtOutput::Vm(vm)
            } else if let Ok(container) = SystemdDetectVirtContainer::try_from(s) {
                SystemdDetectVirtOutput::Container(container)
            } else if s == "none" {
                SystemdDetectVirtOutput::None
            } else {
                return Err(Error::UnknownSystemdDetectVirtOutput {
                    output: s.to_string(),
                });
            },
        )
    }
}

/// Detects whether currently running in a virtualized or containerized Linux environment.
///
/// Uses [systemd-detect-virt] to detect virtualization and containerization environments and
/// returns its output in the form of a [`SystemdDetectVirtOutput`].
///
/// # Errors
///
/// Returns an error if
///
/// - [systemd-detect-virt] cannot be found,
/// - [systemd-detect-virt] cannot be executed successfully,
/// - or the output of [systemd-detect-virt] does not match any variant of
///   [`SystemdDetectVirtOutput`].
///
/// [systemd-detect-virt]: https://man.archlinux.org/man/systemd-detect-virt.1
pub fn detect_virt() -> Result<SystemdDetectVirtOutput, Error> {
    let command_name = get_command("systemd-detect-virt")?;
    let mut command = Command::new(command_name);
    let output = command
        .output()
        .map_err(|source| crate::Error::CommandExec {
            command: format!("{command:?}"),
            source,
        })?;

    SystemdDetectVirtOutput::from_str(&String::from_utf8_lossy(&output.stdout))
}

/// Returns the path to a `command`.
///
/// Searches for an executable in `$PATH` of the current environment and returns the first one
/// found.
///
/// # Errors
///
/// Returns an error if no executable matches the provided `command`.
pub fn get_command(command: &str) -> Result<PathBuf, Error> {
    which(command).map_err(|source| Error::ExecutableNotFound {
        command: command.to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    /// Ensures that [`SystemdDetectVirtOutput`] is properly roundtripped from/to [`str`].
    #[rstest]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Qemu), "qemu")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Kvm), "kvm")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Amazon), "amazon")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Zvm), "zvm")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Vmware), "vmware")]
    #[case(
        SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Microsoft),
        "microsoft"
    )]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Oracle), "oracle")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::PowerVm), "powervm")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Xen), "xen")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Bochs), "bochs")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Uml), "uml")]
    #[case(
        SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Parallels),
        "parallels"
    )]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Bhyve), "bhyve")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Qnx), "qnx")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Acrn), "acrn")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Apple), "apple")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Sre), "sre")]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Google), "google")]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::OpenVc),
        "openvc"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Lxc),
        "lxc"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::LxcLibvirt),
        "lxc-libvirt"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::SystemdNspawn),
        "systemd-nspawn"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Docker),
        "docker"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Podman),
        "podman"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Rkt),
        "rkt"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Wsl),
        "wsl"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Proot),
        "proot"
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Pouch),
        "pouch"
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::Sev),
        "sev"
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::SevEs),
        "sev-es"
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::SevSnp),
        "sev-snp"
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::Tdx),
        "tdx"
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::Protvirt),
        "protvirt"
    )]
    #[case(SystemdDetectVirtOutput::None, "none")]
    fn systemd_detect_virt_output_serialize_deserialize(
        #[case] output: SystemdDetectVirtOutput,
        #[case] to_string: &str,
    ) -> TestResult {
        assert_eq!(output.to_string(), to_string);
        assert_eq!(SystemdDetectVirtOutput::from_str(to_string)?, output);
        Ok(())
    }

    /// Ensures that [`SystemdDetectVirtOutput::uses_namespaces`] works as intended.
    #[rstest]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Qemu), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Kvm), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Amazon), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Zvm), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Vmware), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Microsoft), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Oracle), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::PowerVm), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Xen), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Bochs), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Uml), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Parallels), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Bhyve), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Qnx), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Acrn), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Apple), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Sre), false)]
    #[case(SystemdDetectVirtOutput::Vm(SystemdDetectVirtVm::Google), false)]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::OpenVc),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Lxc),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::LxcLibvirt),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::SystemdNspawn),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Docker),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Podman),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Rkt),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Wsl),
        false
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Proot),
        false
    )]
    #[case(
        SystemdDetectVirtOutput::Container(SystemdDetectVirtContainer::Pouch),
        true
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::Sev),
        false
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::SevEs),
        false
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::SevSnp),
        false
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::Tdx),
        false
    )]
    #[case(
        SystemdDetectVirtOutput::Cms(ConfidentialVirtualizationTechnology::Protvirt),
        false
    )]
    #[case(SystemdDetectVirtOutput::None, false)]
    fn systemd_detect_virt_output_uses_namespaces(
        #[case] output: SystemdDetectVirtOutput,
        #[case] uses_namespaces: bool,
    ) -> TestResult {
        assert_eq!(output.uses_namespaces(), uses_namespaces);
        Ok(())
    }
}
