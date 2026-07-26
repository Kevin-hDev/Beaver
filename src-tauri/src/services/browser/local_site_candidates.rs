use super::{local_site_policy::is_internal_port, local_site_types::MAX_LOCAL_CANDIDATES};
use netstat2::{
    AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo, TcpSocketInfo, TcpState,
};
use std::{collections::BTreeMap, net::IpAddr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LocalSiteCandidate {
    pub(super) port: u16,
    pub(super) ipv4: bool,
    pub(super) ipv6: bool,
}

impl LocalSiteCandidate {
    pub(super) fn socket_addresses(self) -> Vec<std::net::SocketAddr> {
        let mut addresses = Vec::with_capacity(2);
        if self.ipv4 {
            addresses.push(std::net::SocketAddr::from(([127, 0, 0, 1], self.port)));
        }
        if self.ipv6 {
            addresses.push(std::net::SocketAddr::from((
                [0, 0, 0, 0, 0, 0, 0, 1],
                self.port,
            )));
        }
        addresses
    }
}

pub(super) fn listening_candidates() -> Result<Vec<LocalSiteCandidate>, ()> {
    let families = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let sockets = netstat2::iterate_sockets_info(families, ProtocolFlags::TCP).map_err(|_| ())?;
    let mut bounded = Vec::with_capacity(MAX_LOCAL_CANDIDATES);
    for socket in sockets {
        bounded.push(socket.map_err(|_| ())?);
        if bounded.len() == MAX_LOCAL_CANDIDATES * 8 {
            break;
        }
    }
    Ok(collect_candidates(bounded))
}

pub(super) fn collect_candidates(
    sockets: impl IntoIterator<Item = SocketInfo>,
) -> Vec<LocalSiteCandidate> {
    let mut ports: BTreeMap<u16, (bool, bool)> = BTreeMap::new();
    for socket in sockets {
        let Some((port, address)) = listening_endpoint(socket) else {
            continue;
        };
        if is_internal_port(port) {
            continue;
        }
        if !ports.contains_key(&port) && ports.len() == MAX_LOCAL_CANDIDATES {
            continue;
        }
        let flags = ports.entry(port).or_default();
        match address {
            IpAddr::V4(_) => flags.0 = true,
            IpAddr::V6(_) => flags.1 = true,
        }
    }
    ports
        .into_iter()
        .map(|(port, (ipv4, ipv6))| LocalSiteCandidate { port, ipv4, ipv6 })
        .collect()
}

fn listening_endpoint(socket: SocketInfo) -> Option<(u16, IpAddr)> {
    let ProtocolSocketInfo::Tcp(TcpSocketInfo {
        local_addr,
        local_port,
        state: TcpState::Listen,
        ..
    }) = socket.protocol_socket_info
    else {
        return None;
    };
    if local_port == 0 || !(local_addr.is_loopback() || local_addr.is_unspecified()) {
        return None;
    }
    Some((local_port, local_addr))
}
