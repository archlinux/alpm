//! A rootless backend that uses [rootlesskit].
//!
//! [rootlesskit]: https://github.com/rootless-containers/rootlesskit

use std::{
    fmt::Display,
    path::PathBuf,
    process::{Command, Output},
};

use log::debug;

use crate::{
    Error,
    RootlessBackend,
    RootlessOptions,
    utils::{detect_virt, get_command},
};

/// The `rootlesskit` `--debug` option.
const ARG_DEBUG: &str = "--debug";
/// The `rootlesskit` `--copy-up` option.
const ARG_COPY_UP: &str = "--copy-up";
/// The `rootlesskit` `--copy-up-mode` option.
const ARG_COPY_UP_MODE: &str = "--copy-up-mode";
/// The `rootlesskit` `--propagation` option.
const ARG_PROPAGATION: &str = "--propagation";
/// The `rootlesskit` `--net` option.
const ARG_NET: &str = "--net";
/// The `rootlesskit` `--mtu` option.
const ARG_MTU: &str = "--mtu";
/// The `rootlesskit` `--cidr` option.
const ARG_CIDR: &str = "--cidr";
/// The `rootlesskit` `--ifname` option.
const ARG_IFNAME: &str = "--ifname";
/// The `rootlesskit` `--disable-host-loopback` option.
const ARG_DISABLE_HOST_LOOPBACK: &str = "--disable-host-loopback";
/// The `rootlesskit` `--ipv6` option.
const ARG_IPV6: &str = "--ipv6";
/// The `rootlesskit` `--detach-netns` option.
const ARG_DETACH_NETNS: &str = "--detach-netns";
/// The `rootlesskit` `--lxc-user-nic-binary` option.
const ARG_LXC_USER_NIC_BINARY: &str = "--lxc-user-nic-binary";
/// The `rootlesskit` `--lxc-user-nic-bridge` option.
const ARG_LXC_USER_NIC_BRIDGE: &str = "--lxc-user-nic-bridge";
/// The `rootlesskit` `--pasta-binary` option.
const ARG_PASTA_BINARY: &str = "--pasta-binary";
/// The `rootlesskit` `--slirp4netns-binary` option.
const ARG_SLIRP4NETNS_BINARY: &str = "--slirp4netns-binary";
/// The `rootlesskit` `--slirp4netns-sandbox` option.
const ARG_SLIRP4NETNS_SANDBOX: &str = "--slirp4netns-sandbox";
/// The `rootlesskit` `--slirp4netns-seccomp` option.
const ARG_SLIRP4NETNS_SECCOMP: &str = "--slirp4netns-seccomp";
/// The `rootlesskit` `--vpnkit-binary` option.
const ARG_VPNKIT_BINARY: &str = "--vpnkit-binary";
/// The `rootlesskit` `--port-driver` option.
const ARG_PORT_DRIVER: &str = "--port-driver";
/// The `rootlesskit` `--publish` option.
const ARG_PUBLISH: &str = "--publish";
/// The `rootlesskit` `--pidns` option.
const ARG_PIDNS: &str = "--pidns";
/// The `rootlesskit` `--cgroupns` option.
const ARG_CGROUPNS: &str = "--cgroupns";
/// The `rootlesskit` `--utsns` option.
const ARG_UTSNS: &str = "--utsns";
/// The `rootlesskit` `--ipcns` option.
const ARG_IPCNS: &str = "--ipcns";
/// The `rootlesskit` `--reaper` option.
const ARG_REAPER: &str = "--reaper";
/// The `rootlesskit` `--evacuate-cgroup2` option.
const ARG_EVACUATE_CGROUP2: &str = "--evacuate-cgroup2";
/// The `rootlesskit` `--state-dir` option.
const ARG_STATE_DIR: &str = "--state-dir";
/// The `rootlesskit` `--subid-source` option.
const ARG_SUBID_SOURCE: &str = "--subid-source";

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
#[derive(Clone, Copy, Debug, strum::Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum AutoOption {
    /// The option is on.
    True,

    /// The option is off.
    False,

    /// The option is chosen automatically.
    Auto,
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

    /// Whether to enable IPv6 routing.
    ///
    /// Requires `net` to either be set to [`Net::Pasta`] or [`Net::Slirp4netns`].
    ///
    /// Corresponds to `rootlesskit`'s `--ipv6` option.
    pub ipv6: bool,

    /// Whether to detach the network namespaces.
    ///
    /// Corresponds to `rootlesskit`'s `--detach-netns` option.
    pub detach_netns: bool,

    /// An alternative path for the `lxc-user-nic` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--lxc-user-nic-binary` option.
    pub lxc_user_nic_binary: Option<PathBuf>,

    /// An alternative name for the `lxc-user-bridge` name.
    ///
    /// Corresponds to `rootlesskit`'s `--lxc-user-nic-bridge` option.
    pub lxc_user_nic_bridge: Option<String>,

    /// An alternative path for the `pasta` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--pasta-binary` option.
    pub pasta_binary: Option<PathBuf>,

    /// An alternative path for the `slirp4netns` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--slirp4netns-binary` option.
    pub slirp4netns_binary: Option<PathBuf>,

    /// Whether to enable `slirp4netns` sandbox.
    ///
    /// Corresponds to `rootlesskit`'s `--slirp4netns-sandbox` option.
    pub slirp4netns_sandbox: Option<AutoOption>,

    /// Whether to enable `slirp4netns` seccomp.
    ///
    /// Corresponds to `rootlesskit`'s `--slirp4netns-seccomp` option.
    pub slirp4netns_seccomp: Option<AutoOption>,

    /// An alternative path for the `vpnkit` binary.
    ///
    /// Corresponds to `rootlesskit`'s `--vpnkit-binary` option.
    pub vpnkit_binary: Option<PathBuf>,

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
    pub reaper: Option<AutoOption>,

    /// A cgroup2 subgroup to evacuate processes into.
    ///
    /// Requires [`RootlesskitOptions::pidns`] and [`RootlesskitOptions::cgroupns`] to be set to
    /// `true`.
    ///
    /// Corresponds to `rootlesskit`'s `--evacuate-cgroup2` option.
    pub evacuate_cgroup2: Option<String>,

    /// A state directory to use.
    ///
    /// Corresponds to `rootlesskit`'s `--state-dir` option.
    pub state_dir: Option<PathBuf>,

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
    ///
    /// # Notes
    ///
    /// All [`PathBuf`] options are represented using [`PathBuf::to_string_lossy`].
    fn to_vec(&self) -> Vec<String> {
        let mut options = Vec::new();
        if self.debug {
            options.push(ARG_DEBUG.to_string());
        }
        for option in self.copy_up.iter() {
            options.push(ARG_COPY_UP.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.copy_up_mode {
            options.push(ARG_COPY_UP_MODE.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.propagation {
            options.push(ARG_PROPAGATION.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.net {
            options.push(ARG_NET.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.mtu {
            options.push(ARG_MTU.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.cidr.as_ref() {
            options.push(ARG_CIDR.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.ifname.as_ref() {
            options.push(ARG_IFNAME.to_string());
            options.push(option.to_string());
        }
        if self.disable_host_loopback {
            options.push(ARG_DISABLE_HOST_LOOPBACK.to_string());
        }
        if self.ipv6 {
            options.push(ARG_IPV6.to_string());
        }
        if self.detach_netns {
            options.push(ARG_DETACH_NETNS.to_string());
        }
        if let Some(option) = self.lxc_user_nic_binary.as_ref() {
            options.push(ARG_LXC_USER_NIC_BINARY.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.lxc_user_nic_bridge.as_ref() {
            options.push(ARG_LXC_USER_NIC_BRIDGE.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.pasta_binary.as_ref() {
            options.push(ARG_PASTA_BINARY.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.slirp4netns_binary.as_ref() {
            options.push(ARG_SLIRP4NETNS_BINARY.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.slirp4netns_sandbox {
            options.push(ARG_SLIRP4NETNS_SANDBOX.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.slirp4netns_seccomp {
            options.push(ARG_SLIRP4NETNS_SECCOMP.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.vpnkit_binary.as_ref() {
            options.push(ARG_VPNKIT_BINARY.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.port_driver {
            options.push(ARG_PORT_DRIVER.to_string());
            options.push(option.to_string());
        }
        for option in self.publish.iter() {
            options.push(ARG_PUBLISH.to_string());
            options.push(option.to_string());
        }
        if self.pidns {
            options.push(ARG_PIDNS.to_string());
        }
        if self.cgroupns {
            options.push(ARG_CGROUPNS.to_string());
        }
        if self.utsns {
            options.push(ARG_UTSNS.to_string());
        }
        if self.ipcns {
            options.push(ARG_IPCNS.to_string());
        }
        if let Some(option) = self.reaper {
            options.push(ARG_REAPER.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.evacuate_cgroup2.as_ref() {
            options.push(ARG_EVACUATE_CGROUP2.to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.state_dir.as_ref() {
            options.push(ARG_STATE_DIR.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.subid_source {
            options.push(ARG_SUBID_SOURCE.to_string());
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
    fn run(&self, cmd: &[&str]) -> Result<Output, Self::Err> {
        {
            let virt = detect_virt()?;
            if virt.uses_namespaces() {
                return Err(Error::NamespacesInContainer { runtime: virt });
            }
        }

        let command_name = get_command("rootlesskit")?;
        let mut command = Command::new(command_name);

        // Add all options to rootless as arguments.
        if self.0.debug {
            command.arg(ARG_DEBUG);
        }
        for option in self.0.copy_up.iter() {
            command.arg(ARG_COPY_UP);
            command.arg(option);
        }
        if let Some(option) = self.0.copy_up_mode {
            command.arg(ARG_COPY_UP_MODE);
            command.arg(option.to_string());
        }
        if let Some(option) = self.0.propagation {
            command.arg(ARG_PROPAGATION);
            command.arg(option.to_string());
        }
        if let Some(option) = self.0.net {
            command.arg(ARG_NET);
            command.arg(option.to_string());
        }
        if let Some(option) = self.0.mtu {
            command.arg(ARG_MTU);
            command.arg(option.to_string());
        }
        if let Some(option) = self.0.cidr.as_ref() {
            command.arg(ARG_CIDR);
            command.arg(option);
        }
        if let Some(option) = self.0.ifname.as_ref() {
            command.arg(ARG_IFNAME);
            command.arg(option);
        }
        if self.0.disable_host_loopback {
            command.arg(ARG_DISABLE_HOST_LOOPBACK);
        }
        if self.0.ipv6 {
            command.arg(ARG_IPV6);
        }
        if self.0.detach_netns {
            command.arg(ARG_DETACH_NETNS);
        }
        if let Some(option) = self.0.lxc_user_nic_binary.as_ref() {
            command.arg(ARG_LXC_USER_NIC_BINARY);
            command.arg(option);
        }
        if let Some(option) = self.0.lxc_user_nic_bridge.as_ref() {
            command.arg(ARG_LXC_USER_NIC_BRIDGE);
            command.arg(option);
        }
        if let Some(option) = self.0.pasta_binary.as_ref() {
            command.arg(ARG_PASTA_BINARY);
            command.arg(option);
        }
        if let Some(option) = self.0.slirp4netns_binary.as_ref() {
            command.arg(ARG_SLIRP4NETNS_BINARY);
            command.arg(option);
        }
        if let Some(option) = self.0.slirp4netns_sandbox {
            command.arg(ARG_SLIRP4NETNS_SANDBOX);
            command.arg(option.to_string());
        }
        if let Some(option) = self.0.slirp4netns_seccomp {
            command.arg(ARG_SLIRP4NETNS_SECCOMP);
            command.arg(option.to_string());
        }
        if let Some(option) = self.0.vpnkit_binary.as_ref() {
            command.arg(ARG_VPNKIT_BINARY);
            command.arg(option);
        }
        if let Some(option) = self.0.port_driver {
            command.arg(ARG_PORT_DRIVER);
            command.arg(option.to_string());
        }
        for option in self.0.publish.iter() {
            command.arg(ARG_PUBLISH);
            command.arg(option);
        }
        if self.0.pidns {
            command.arg(ARG_PIDNS);
        }
        if self.0.cgroupns {
            command.arg(ARG_CGROUPNS);
        }
        if self.0.utsns {
            command.arg(ARG_UTSNS);
        }
        if self.0.ipcns {
            command.arg(ARG_IPCNS);
        }
        if let Some(option) = self.0.reaper {
            command.arg(ARG_REAPER);
            command.arg(option.to_string());
        }
        if let Some(option) = self.0.evacuate_cgroup2.as_ref() {
            command.arg(ARG_EVACUATE_CGROUP2);
            command.arg(option);
        }
        if let Some(option) = self.0.state_dir.as_ref() {
            command.arg(ARG_STATE_DIR);
            command.arg(option);
        }
        if let Some(option) = self.0.subid_source {
            command.arg(ARG_SUBID_SOURCE);
            command.arg(option.to_string());
        }

        // Add input cmd as arguments to rootlesskit.
        for command_component in cmd.iter() {
            command.arg(command_component);
        }

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
    ipv6: true,
    detach_netns: true,
    lxc_user_nic_binary: Some(PathBuf::from("/usr/local/bin/lxc-user-nic")),
    lxc_user_nic_bridge: Some("lxcbr1".to_string()),
    pasta_binary: Some(PathBuf::from("/usr/local/bin/pasta")),
    slirp4netns_binary: Some(PathBuf::from("/usr/local/bin/slirp4netns")),
    slirp4netns_sandbox: Some(AutoOption::Auto),
    slirp4netns_seccomp: Some(AutoOption::Auto),
    vpnkit_binary: Some(PathBuf::from("/usr/local/bin/vpnkit")),
    port_driver: Some(PortDriver::Implicit),
    publish: vec![
        "127.0.0.1:8080:80/tcp".to_string(),
        "127.0.0.1:8443:443/tcp".to_string(),
    ],
    pidns: true,
    cgroupns: true,
    utsns: true,
    ipcns: true,
    reaper: Some(AutoOption::Auto),
    evacuate_cgroup2: Some("testgroup".to_string()),
    state_dir: Some(PathBuf::from("/var/foo")),
    subid_source: Some(SubIdSource::Dynamic),
        },
        vec![
            ARG_DEBUG.to_string(),
            ARG_COPY_UP.to_string(),
            "/etc".to_string(),
            ARG_COPY_UP.to_string(),
            "/usr".to_string(),
            ARG_COPY_UP_MODE.to_string(),
            "tmpfs+symlink".to_string(),
            ARG_PROPAGATION.to_string(),
            "rprivate".to_string(),
            ARG_NET.to_string(),
            "pasta".to_string(),
            ARG_MTU.to_string(),
            "1500".to_string(),
            ARG_CIDR.to_string(),
            "10.0.1.0/24".to_string(),
            ARG_IFNAME.to_string(),
            "tap1".to_string(),
            ARG_DISABLE_HOST_LOOPBACK.to_string(),
            ARG_IPV6.to_string(),
            ARG_DETACH_NETNS.to_string(),
            ARG_LXC_USER_NIC_BINARY.to_string(),
            "/usr/local/bin/lxc-user-nic".to_string(),
            ARG_LXC_USER_NIC_BRIDGE.to_string(),
            "lxcbr1".to_string(),
            ARG_PASTA_BINARY.to_string(),
            "/usr/local/bin/pasta".to_string(),
            ARG_SLIRP4NETNS_BINARY.to_string(),
            "/usr/local/bin/slirp4netns".to_string(),
            ARG_SLIRP4NETNS_SANDBOX.to_string(),
            "auto".to_string(),
            ARG_SLIRP4NETNS_SECCOMP.to_string(),
            "auto".to_string(),
            ARG_VPNKIT_BINARY.to_string(),
            "/usr/local/bin/vpnkit".to_string(),
            ARG_PORT_DRIVER.to_string(),
            "implicit".to_string(),
            ARG_PUBLISH.to_string(),
            "127.0.0.1:8080:80/tcp".to_string(),
            ARG_PUBLISH.to_string(),
            "127.0.0.1:8443:443/tcp".to_string(),
            ARG_PIDNS.to_string(),
            ARG_CGROUPNS.to_string(),
            ARG_UTSNS.to_string(),
            ARG_IPCNS.to_string(),
            ARG_REAPER.to_string(),
            "auto".to_string(),
            ARG_EVACUATE_CGROUP2.to_string(),
            "testgroup".to_string(),
            ARG_STATE_DIR.to_string(),
            "/var/foo".to_string(),
            ARG_SUBID_SOURCE.to_string(),
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

    /// Ensures that [`RootlesskitOptions`] is returned from [`RootlesskitBackend::options`].
    #[test]
    fn rootlesskit_backend_options() {
        let backend = RootlesskitBackend::new(RootlesskitOptions::default());
        assert_eq!(backend.options(), &RootlesskitOptions::default());
    }
}
