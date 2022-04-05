use std::io;

enum State{
    Closed,
    Listen,
    SyncRcvd,
    // Estab,
}
//Transmission Control Block
pub struct Connection{
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
}


/// State of the Send Sequence Space (RFC 793 S3.2)
///
///      1         2          3          4
/// ----------|----------|----------|----------
///       SND.UNA    SND.NXT    SND.UNA
///                             +SND.WND
///
/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers of unacknowledged data
/// 3 - sequence numbers allowed for new data transmission
/// 4 - future sequence numbers which are not yet allowed
///
///             Send Sequence Space (RFC 793 S3.2)
struct SendSequenceSpace{
    /// send unacknowledged
    una: u32,
    /// send next
    nxt: u32,
    /// send window
    wnd: u16,
    /// segment sequence number used for last window update
    wl1: usize,
    /// segment acknowledgement number used for last window update
    wl2: usize,
    /// Initial send sequence number
    iss: u32,
    /// send urgent pointer
    up: bool,
}

struct RecvSequenceSpace{
    /// receive next 
    nxt: u32,
    /// receive window
    wnd: u16,
    /// initial receive sequence number
    irs: u32,
    /// receive urgent pointer
    up: bool,
}

impl Connection {
    pub fn accept<'a> (nic: &mut tun_tap::Iface, ip_header: etherparse::Ipv4HeaderSlice<'a>, tcp_header: etherparse::TcpHeaderSlice<'a>, data: &'a [u8] ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];
        let mut unwritten = &mut buf[..];
        
        if !tcp_header.syn(){
            // Only expecting SYN Packet
            return Ok(None);
        }

        let iss = 0;
        let mut c = Connection {
            state: State::SyncRcvd,
            // keep track of sender info
            send: SendSequenceSpace{
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0
            },
            // decide on what to send the other side
            recv: RecvSequenceSpace{
                nxt: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                irs: tcp_header.sequence_number(),
                up: false,
            }
        };
       
        // start establishing connection - create new TCP Header
        let mut syn_ack = etherparse::TcpHeader::new(tcp_header.destination_port(), tcp_header.source_port(), c.send.iss ,c.send.wnd);

        // The next byte we're expected to get from the other side
        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;

        // create IPv4 Hheader
        let mut ip = etherparse::Ipv4Header::new(syn_ack.header_len(), 64, etherparse::IpNumber::Tcp, ip_header.destination_addr().octets(), ip_header.source_addr().octets());


        // Manual checksum calculation;; kernel does this for us
        syn_ack.checksum = syn_ack.calc_checksum_ipv4(&ip, &[]).expect("Failed to compute checksum");

        let unwritten = {
            let mut unwritten = &mut buf[..];
            eprintln!("IP bytes: {:?}", unwritten.len());
            ip.write(&mut unwritten);
            eprintln!("IP bytes: {:?}", unwritten.len());
            syn_ack.write(&mut unwritten);
            eprintln!("SYN_ACK bytes: {:?}", unwritten.len());
            unwritten.len()
        };  
        nic.send(&buf[..buf.len()-unwritten])?;
        Ok(Some(c))
    }

    pub fn on_packet<'a> (&mut self, nic: &mut tun_tap::Iface, ip_header: etherparse::Ipv4HeaderSlice<'a>, tcp_header: etherparse::TcpHeaderSlice<'a>, data: &'a [u8] ) -> io::Result<()> {
        Ok(())
    }
}
