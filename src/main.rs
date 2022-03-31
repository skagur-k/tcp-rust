use tun_tap;
use std::io;
fn main() -> io::Result<()>{
    // Set up virtual network interface (tun0) with the tun mode
    // buf - buffer with size 1504: MTU(1500 octets) + 4 (flag, proto)
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    let nic_name = nic.name();
    eprintln!("Listening on interface: {:?}", nic_name);

    // loops and receives packets
    loop{
        let nbytes = nic.recv(&mut buf[..])?;
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let proto = u16::from_be_bytes([buf[2], buf[3]]);

        
        if proto != 0x0800 {
            // If protocol # is not 0x0800 (IPv4), then ignore and continue
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]){
            Ok(packet) => {
                eprintln!("Read {} bytes [FLAGS: {}, PROTO: {:#06x}]: {:x?} ", nbytes - 4, flags, proto, packet.to_header());
            },
            Err(e) => {
                eprintln!("Ignoring unexpected packet: {:?}", e);
            }
        };



    };
}