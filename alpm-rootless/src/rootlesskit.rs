//! A rootless backend that uses [rootlesskit].
//!
//! [rootlesskit]: https://github.com/rootless-containers/rootlesskit

use std::{
    fmt::Display,
    process::{Command, Output},
};

use log::debug;

use crate::{
    Error,
    RootlessBackend,
    RootlessOptions,
    utils::{detect_virt, get_command},
};

/// The mode used for [rootlesskit]'s `--copy-up` option.
///
/// Corresponds to the value passed to the `--copy-up-mode` option.
///
/// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
#[derive(Clone, Copy, Debug, Default, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum CopyUpMode {
    /// The `tmpfs+symlink` mode.
    #[default]
    #[strum(serialize = "tmpfs+symlink")]
    TmpfsAndSymlink,
}

/// The propagation used for [rootlesskit]'s `--copy-up` option.
///
/// Corresponds to the value passed to the `--propagation` option.
///
/// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
#[derive(Clone, Copy, Debug, Default, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Propagation {
    /// The `rprivate` propagation.
    #[default]
    Rprivate,
    /// The `rslave` propagation.
    Rslave,
}

/// A network driver used by [rootlesskit].
///
/// Corresponds to the value passed to the `--net` option.
///
/// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
#[derive(Clone, Copy, Debug, Default, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Net {
    /// The `host` network driver.
    #[default]
    Host,
    /// No (`none`) network driver.
    None,
    /// The `pasta` network driver.
    Pasta,
    /// The `slirp4netns` network driver.
    Slirp4netns,
    /// The `vpnkit` network driver.
    Vpnkit,
    /// The `lxc-user-nic` network driver.
    #[strum(serialize = "lxc-user-nic")]
    LxcUserNic,
}

/// An option that may be on, off or automatic.
#[derive(Clone, Copy, Debug, Default, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Slirp4netnsAutoOption {
    /// The option is on.
    True,
    /// The option is off.
    #[default]
    False,
    /// The option is chosen automatically.
    Auto,
}

/// An option that may be on, off or automatic.
#[derive(Clone, Copy, Debug, Default, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum ReaperAutoOption {
    /// The option is chosen automatically.
    #[default]
    Auto,
    /// The option is on.
    True,
    /// The option is off.
    False,
}

/// A port driver for the non-host network of [rootlesskit].
///
/// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
#[derive(Clone, Copy, Debug, Default, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum PortDriver {
    /// No (`none`) port driver.
    #[default]
    None,
    /// The `implicit` port driver (for [`Net::Pasta`]).
    Implicit,
    /// The `builtin` port driver.
    Builtin,
    /// The `slirp4netns` port driver.
    Slirp4netns,
}

/// The source of subids for [rootlesskit].
///
/// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
#[derive(Clone, Copy, Debug, Default, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum SubIdSource {
    /// The source of subids is chosen automatically.
    #[default]
    Auto,
    /// The subids are retrieved using [getsubids].
    ///
    /// [getsubids]: https://man.archlinux.org/man/getsubids.1
    Dynamic,
    /// The subids are retrieved from `/etc/sub{uid,gid}`.
    Static,
}

/// Options for [rootlesskit].
///
/// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RootlesskitOptions {
    /// Whether to use debug mode.
    ///
    /// Corresponds to `rootlesskit`'s `--debug` option.
    pub debug: bool,
    /// A list of filesystems to mount and copy-up the contents from.
    ///
    /// Corresponds to `rootlesskit`'s `--copy-up` option.
    pub copy_up: Vec<String>,
    /// The mode to use for [`RootlesskitOptions::copy_up`].
    ///
    /// Corresponds to `rootlesskit`'s `--copy-up-mode` option.
    pub copy_up_mode: Option<CopyUpMode>,
    /// The propagation to use for [`RootlesskitOptions::copy_up`].
    ///
    /// Corresponds to `rootlesskit`'s `--propagation` option.
    pub propagation: Option<Propagation>,
    /// The network driver to use.
    ///
    /// Corresponds to `rootlesskit`'s `--net` option.
    pub net: Option<Net>,
    /// The MTU to use for the network driver.
    ///
    /// Defaults to `65520` for [`Net::Pasta`] and [`Net::Slirp4netns`], `1500` for all other.
    ///
    /// Corresponds to `rootlesskit`'s `--mtu` option.
    pub mtu: Option<usize>,
    /// The CIDR to use for [`Net::Pasta`] and [`Net::Slirp4netns`].
    ///
    /// Defaults to `10.0.2.0/24` for [`Net::Pasta`] and [`Net::Slirp4netns`].
    ///
    /// Corresponds to `rootlesskit`'s `--cidr` option.
    pub cidr: Option<String>,
    /// The network interface name to use.
    ///
    /// Defaults to `tap0` for [`Net::Pasta`], [`Net::Slirp4netns`] and [`Net::Vpnkit`], `eth0` for
    /// [`Net::LxcUserNic`].
    ///
    /// Corresponds to `rootlesskit`'s `--ifname` option.
    pub ifname: Option<String>,
    /// Whether to prohibit connecting to `127.0.0.1:*` on the host.
    ///
    /// Corresponds to `rootlesskit`'s `--disable-host-loopback` option.
    pub disable_host_loopback: bool,
    /// Whether to detach the network namespaces.
    ///
    /// Corresponds to `rootlesskit`'s `--detach-netns` option.
    pub detach_netns: bool,
    /// An alternative path for the `lxc-user-nic` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--lxc-user-nic-binary` option.
    pub lxc_user_nic_binary: Option<String>,
    /// An alternative name for the `lxc-user-bridge` name.
    ///
    /// Corresponds to `rootlesskit`'s `--lxc-user-nic-bridge` option.
    pub lxc_user_nic_bridge: Option<String>,
    /// An alternative path for the `pasta` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--pasta-binary` option.
    pub pasta_binary: Option<String>,
    /// An alternative path for the `slirp4netns` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--slirp4netns-binary` option.
    pub slirp4netns_binary: Option<String>,
    /// Whether to enable `slirp4netns` sandbox.
    ///
    /// Corresponds to `rootlesskit`'s `--slirp4netns-sandbox` option.
    pub slirp4netns_sandbox: Option<Slirp4netnsAutoOption>,
    /// Whether to enable `slirp4netns` seccomp.
    ///
    /// Corresponds to `rootlesskit`'s `--slirp4netns-seccomp` option.
    pub slirp4netns_seccomp: Option<Slirp4netnsAutoOption>,
    /// An alternative path for the `vpnkit` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--vpnkit-binary` option.
    pub vpnkit_binary: Option<String>,
    /// A port driver to use for the non-host network.
    ///
    /// Corresponds to `rootlesskit`'s `--port-driver` option.
    pub port_driver: Option<PortDriver>,
    /// A list of ports to publish.
    ///
    /// Corresponds to `rootlesskit`'s `-p`/`--publish` option.
    pub publish: Vec<String>,
    /// Whether to create a PID namespace.
    ///
    /// Corresponds to `rootlesskit`'s `--pidns` option.
    pub pidns: bool,
    /// Whether to create a cgroup namespace.
    ///
    /// Corresponds to `rootlesskit`'s `--cgroupns` option.
    pub cgroupns: bool,
    /// Whether to create a UTS namespace.
    ///
    /// Corresponds to `rootlesskit`'s `--utsns` option.
    pub utsns: bool,
    /// Whether to create an IPC namespace.
    ///
    /// Corresponds to `rootlesskit`'s `--ipcns` option.
    pub ipcns: bool,
    /// Whether to enable process reaper.
    ///
    /// Requires [`RootlesskitOptions::pidns`] to be set to `true`.
    ///
    /// Corresponds to `rootlesskit`'s `--reaper` option.
    pub reaper: Option<ReaperAutoOption>,
    /// A cgroup2 subgroup to evacuat processes into.
    ///
    /// Requires [`RootlesskitOptions::pidns`] and [`RootlesskitOptions::cgroupns`] to be set to
    /// `true`.
    ///
    /// Corresponds to `rootlesskit`'s `--evacuate-cgroup2` option.
    pub evacuate_cgroup2: Option<String>,
    /// A state directory to use.
    ///
    /// Corresponds to `rootlesskit`'s `--state-dir` option.
    pub state_dir: Option<String>,
    /// The source of subids.
    ///
    /// Corresponds to `rootlesskit`'s `--subid-source` option.
    pub subid_source: Option<SubIdSource>,
}

impl Display for RootlesskitOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vec().join(" "))
    }
}

impl RootlessOptions for RootlesskitOptions {
    /// Returns the options as a [`String`] [`Vec`].
    fn to_vec(&self) -> Vec<String> {
        let mut options = Vec::new();
        if self.debug {
            options.push("--debug".to_string());
        }
        for option in self.copy_up.iter() {
            options.push("--copy-up".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.copy_up_mode {
            options.push("--copy-up-mode".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.propagation {
            options.push("--propagation".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.net {
            options.push("--net".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.mtu {
            options.push("--mtu".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.cidr.as_ref() {
            options.push("--cidr".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.ifname.as_ref() {
            options.push("--ifname".to_string());
            options.push(option.to_string());
        }
        if self.disable_host_loopback {
            options.push("--disable-host-loopback".to_string());
        }
        if self.detach_netns {
            options.push("--detach-netns".to_string());
        }
        if let Some(option) = self.lxc_user_nic_binary.as_ref() {
            options.push("--lxc-user-nic-binary".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.lxc_user_nic_bridge.as_ref() {
            options.push("--lxc-user-nic-bridge".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.pasta_binary.as_ref() {
            options.push("--pasta-binary".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.slirp4netns_binary.as_ref() {
            options.push("--slirp4netns-binary".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.slirp4netns_sandbox {
            options.push("--slirp4netns-sandbox".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.slirp4netns_seccomp {
            options.push("--slirp4netns-seccomp".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.vpnkit_binary.as_ref() {
            options.push("--vpnkit-binary".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.port_driver {
            options.push("--port-driver".to_string());
            options.push(option.to_string());
        }
        for option in self.publish.iter() {
            options.push("--publish".to_string());
            options.push(option.to_string());
        }
        if self.pidns {
            options.push("--pidns".to_string());
        }
        if self.cgroupns {
            options.push("--cgroupns".to_string());
        }
        if self.utsns {
            options.push("--utsns".to_string());
        }
        if self.ipcns {
            options.push("--ipcns".to_string());
        }
        if let Some(option) = self.reaper {
            options.push("--reaper".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.evacuate_cgroup2.as_ref() {
            options.push("--evacuate-cgroup2".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.state_dir.as_ref() {
            options.push("--state-dir".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.subid_source {
            options.push("--subid-source".to_string());
            options.push(option.to_string());
        }

        options
    }
}

/// A rootless backend for running commands using [rootlesskit].
///
/// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
#[derive(Clone, Debug)]
pub struct RootlesskitBackend(RootlesskitOptions);

impl RootlessBackend<RootlesskitOptions> for RootlesskitBackend {
    type Err = Error;

    /// Creates a new [`RootlesskitBackend`] from [`RootlesskitOptions`].
    fn new(options: RootlesskitOptions) -> Self {
        debug!("Create a new rootlesskit backend with options \"{options}\"");
        Self(options)
    }

    /// Returns the [`RootlesskitOptions`] used by the [`RootlesskitBackend`].
    fn options(&self) -> &RootlesskitOptions {
        &self.0
    }

    /// Runs a command using [rootlesskit] and returns its [`Output`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the [rootlesskit] command cannot be found,
    /// - or the provided `command` cannot be run using [rootlesskit].
    ///
    /// [rootlesskit]: https://github.com/rootless-containers/rootlesskit
    fn run(&self, command: &[&str]) -> Result<Output, Self::Err> {
        {
            let virt = detect_virt()?;
            if virt.uses_namespaces() {
                return Err(Error::NamespacesInContainer { runtime: virt });
            }
        }

        let mut args = self.0.to_vec();
        for command_component in command.iter() {
            args.push(command_component.to_string());
        }

        let command_name = get_command("rootlesskit")?;
        let mut command = Command::new(command_name);
        let command = command.args(args);
        debug!("Run rootless command: {command:?}");

        command
            .output()
            .map_err(|source| crate::Error::CommandExec {
                command: format!("{command:?}"),
                source,
            })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    /// Ensures that [`RootlesskitOptions`] are constructed as [`String`] [`Vec`] properly.
    #[rstest]
    #[case::all_options(
        RootlesskitOptions{
    debug: true,
    copy_up: vec!["/etc".to_string(), "/usr".to_string()],
    copy_up_mode: Some(CopyUpMode::default()),
    propagation: Some(Propagation::default()),
    net: Some(Net::Pasta),
    mtu: Some(1500),
    cidr: Some("10.0.1.0/24".to_string()),
    ifname: Some("tap1".to_string()),
    disable_host_loopback: true,
    detach_netns: true,
    lxc_user_nic_binary: Some("/usr/local/bin/lxc-user-nic".to_string()),
    lxc_user_nic_bridge: Some("lxcbr1".to_string()),
    pasta_binary: Some("/usr/local/bin/pasta".to_string()),
    slirp4netns_binary: Some("/usr/local/bin/slirp4netns".to_string()),
    slirp4netns_sandbox: Some(Slirp4netnsAutoOption::Auto),
    slirp4netns_seccomp: Some(Slirp4netnsAutoOption::Auto),
    vpnkit_binary: Some("/usr/local/bin/vpnkit".to_string()),
    port_driver: Some(PortDriver::Implicit),
    publish: vec![
        "127.0.0.1:8080:80/tcp".to_string(),
        "127.0.0.1:8443:443/tcp".to_string(),
    ],
    pidns: true,
    cgroupns: true,
    utsns: true,
    ipcns: true,
    reaper: Some(ReaperAutoOption::Auto),
    evacuate_cgroup2: Some("testgroup".to_string()),
    state_dir: Some("/var/foo".to_string()),
    subid_source: Some(SubIdSource::Dynamic),
        },
        vec![
            "--debug".to_string(),
            "--copy-up".to_string(),
            "/etc".to_string(),
            "--copy-up".to_string(),
            "/usr".to_string(),
            "--copy-up-mode".to_string(),
            "tmpfs+symlink".to_string(),
            "--propagation".to_string(),
            "rprivate".to_string(),
            "--net".to_string(),
            "pasta".to_string(),
            "--mtu".to_string(),
            "1500".to_string(),
            "--cidr".to_string(),
            "10.0.1.0/24".to_string(),
            "--ifname".to_string(),
            "tap1".to_string(),
            "--disable-host-loopback".to_string(),
            "--detach-netns".to_string(),
            "--lxc-user-nic-binary".to_string(),
            "/usr/local/bin/lxc-user-nic".to_string(),
            "--lxc-user-nic-bridge".to_string(),
            "lxcbr1".to_string(),
            "--pasta-binary".to_string(),
            "/usr/local/bin/pasta".to_string(),
            "--slirp4netns-binary".to_string(),
            "/usr/local/bin/slirp4netns".to_string(),
            "--slirp4netns-sandbox".to_string(),
            "auto".to_string(),
            "--slirp4netns-seccomp".to_string(),
            "auto".to_string(),
            "--vpnkit-binary".to_string(),
            "/usr/local/bin/vpnkit".to_string(),
            "--port-driver".to_string(),
            "implicit".to_string(),
            "--publish".to_string(),
            "127.0.0.1:8080:80/tcp".to_string(),
            "--publish".to_string(),
            "127.0.0.1:8443:443/tcp".to_string(),
            "--pidns".to_string(),
            "--cgroupns".to_string(),
            "--utsns".to_string(),
            "--ipcns".to_string(),
            "--reaper".to_string(),
            "auto".to_string(),
            "--evacuate-cgroup2".to_string(),
            "testgroup".to_string(),
            "--state-dir".to_string(),
            "/var/foo".to_string(),
            "--subid-source".to_string(),
            "dynamic".to_string(),
        ]
    )]
    #[case::default_options(RootlesskitOptions::default(), Vec::new())]
    fn rootlesskit_options_to_vec(
        #[case] options: RootlesskitOptions,
        #[case] to_vec: Vec<String>,
    ) {
        assert_eq!(options.to_vec(), to_vec);
    }

    /// Ensures that [`FakerootOptions`] is returned from [`FakerootBackend::options`].
    #[test]
    fn rootlesskit_backend_options() {
        let backend = RootlesskitBackend::new(RootlesskitOptions::default());
        assert_eq!(backend.options(), &RootlesskitOptions::default());
    }
}
