use tun_tap;
use std::io;
use std::collections::HashMap;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}
fn main() -> io::Result<()>{
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();

    // Set up virtual network interface (tun0) with the tun mode
    // buf - buffer with size 1504: MTU(1500 octets) + 4 (flag, proto)
    let mut nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    let nic_name = nic.name();
    eprintln!("Listening on interface: {:?}", nic_name);

    // loops and receives packets
    loop{
        // Layer 3
        let nbytes = nic.recv(&mut buf[..])?;
        // let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        // let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        
        // if eth_proto != 0x0800 {
        //     // If Layer 3 protocol is not 0x0800 (IPv4), then ignore and continue
        //     continue;
        // }

        // Layer 4
        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]){
            Ok(ip_header) => {
                let src = ip_header.source_addr();
                let dst = ip_header.destination_addr();
                let _payload_len = ip_header.payload_len();

                if ip_header.protocol() != 0x06{
                    // If Layer 4 protocol is not 0x06 (TCP), then ignore and continue
                    continue
                }

                // Parse TCP Header from the buffer
                // 4 bytes (flag, proto) + Ip_Header_Length() bytes .. end of buffer
                match etherparse::TcpHeaderSlice::from_slice(&buf[ip_header.slice().len()..]){
                    Ok(tcp_header) => {
                        use std::collections::hash_map::Entry;
                        // Get the starting index of data
                        let data = ip_header.slice().len() + tcp_header.slice().len();

                        match connections.entry(
                            Quad { src: (src, tcp_header.source_port()), 
                                        dst: (dst, tcp_header.destination_port())
                            })
                        {
                                Entry::Occupied(mut c) => {
                                    c.get_mut().on_packet(&mut nic, ip_header, tcp_header, &buf[data..nbytes])?;
                                }
                                Entry::Vacant(mut e) => {
                                    if let Some(c) = tcp::Connection::accept(&mut nic, ip_header, tcp_header, &buf[data..nbytes])?{
                                        e.insert(c);
                                    }
                                }
                        }                        
                    }
                    Err(e) => {
                        eprintln!("Unexpected TCP Header: {:?}", e);
                    }
                }
            },
            Err(e) => {
                eprintln!("Ignoring unexpected packet: {:?}", e);
            }
        };



    };
}