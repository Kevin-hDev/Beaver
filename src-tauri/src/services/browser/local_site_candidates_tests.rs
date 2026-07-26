use super::{local_site_candidates::collect_candidates, local_site_types::MAX_LOCAL_CANDIDATES};
use netstat2::{ProtocolSocketInfo, SocketInfo, TcpSocketInfo, TcpState};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn keeps_only_local_listeners_and_merges_address_families() {
    let sockets = vec![
        socket(IpAddr::V4(Ipv4Addr::LOCALHOST), 8_000, TcpState::Listen),
        socket(IpAddr::V6(Ipv6Addr::LOCALHOST), 8_000, TcpState::Listen),
        socket(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            8_001,
            TcpState::Listen,
        ),
        socket(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            8_002,
            TcpState::Established,
        ),
        socket(IpAddr::V4(Ipv4Addr::LOCALHOST), 11_434, TcpState::Listen),
    ];

    let candidates = collect_candidates(sockets);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].port, 8_000);
    assert!(candidates[0].ipv4 && candidates[0].ipv6);
}

#[test]
fn candidate_collection_is_bounded_and_stably_sorted() {
    let sockets = (0..MAX_LOCAL_CANDIDATES + 20).rev().map(|offset| {
        socket(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            20_000 + offset as u16,
            TcpState::Listen,
        )
    });

    let candidates = collect_candidates(sockets);

    assert_eq!(candidates.len(), MAX_LOCAL_CANDIDATES);
    assert!(candidates
        .windows(2)
        .all(|pair| pair[0].port < pair[1].port));
}

fn socket(address: IpAddr, port: u16, state: TcpState) -> SocketInfo {
    SocketInfo {
        protocol_socket_info: ProtocolSocketInfo::Tcp(TcpSocketInfo {
            local_addr: address,
            local_port: port,
            remote_addr: match address {
                IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            },
            remote_port: 0,
            state,
        }),
        associated_pids: Vec::new(),
        #[cfg(target_os = "linux")]
        inode: 0,
        #[cfg(target_os = "linux")]
        uid: 0,
    }
}
