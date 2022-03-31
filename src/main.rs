use tun_tap;
use std::io;
use std::collections::HashMap;
use std::net::Ipv4Addr;

mod tcp;
struct Quad{
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}
fn main() -> io::Result<()>{
    let mut connections: HashMap<Quad, tcp::State> = Default::default();

    // Set up virtual network interface (tun0) with the tun mode
    // buf - buffer with size 1504: MTU(1500 octets) + 4 (flag, proto)
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    let nic_name = nic.name();
    eprintln!("Listening on interface: {:?}", nic_name);

    // loops and receives packets
    loop{
        // Layer 3
        let nbytes = nic.recv(&mut buf[..])?;
        let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        
        if eth_proto != 0x0800 {
            // If Layer 3 protocol is not 0x0800 (IPv4), then ignore and continue
            continue;
        }

        // Layer 4
        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]){
            Ok(packet) => {
                let src = packet.source_addr();
                let dst = packet.destination_addr();
                let proto = packet.protocol();
                let _payload_len = packet.payload_len();

                if proto != 0x06{
                    // If Layer 4 protocol is not 0x06 (TCP), then ignore and continue
                    continue
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[4+packet.slice().len()..]){
                    Ok(p) => {
                        eprintln!("SRC: {} -> DST: {} [{} bytes of TCP to Port: {}] ", src, dst, p.slice().len(), p.destination_port());
                    },
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