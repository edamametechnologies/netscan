use crate::{tcp, ipv4, ethernet};
use crate::packet::EndPoints;
use crate::status::ScanStatus;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::net::Ipv4Addr;
use pnet::datalink::MacAddr;
use pnet::packet::Packet;
use std::time::{Duration, Instant};

pub struct TcpOptions {
    pub sender_mac: MacAddr,
    pub target_mac: MacAddr,
    pub src_ip: Ipv4Addr,
    pub dst_ip: Ipv4Addr,    
    pub src_port: u16,
    pub min_port_num: u16,
    pub max_port_num: u16,
    pub scan_type: tcp::PortScanType,
    pub timeout: Duration,
}

pub fn scan_ports(interface: &pnet::datalink::NetworkInterface, tcp_options: &TcpOptions) -> (Vec<String>, ScanStatus)
{
    let mut result = vec![];
    let stop: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let open_ports = Arc::new(Mutex::new(vec![]));
    let scan_status: Arc<Mutex<ScanStatus>> = Arc::new(Mutex::new(ScanStatus::Ready));
    let (mut tx, mut rx) = match pnet::datalink::channel(&interface, Default::default()) {
        Ok(pnet::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };
    rayon::join(|| send_tcp_packet(&mut tx, &tcp_options, &stop),
                || receive_packets(&mut rx, &tcp_options, &open_ports, &stop, &scan_status)
    );
    for port in open_ports.lock().unwrap().iter(){
        result.push(port.to_string());
    }
    return (result, *scan_status.lock().unwrap());
}

fn build_packet(tcp_options: &TcpOptions, tmp_packet: &mut [u8], target_port: u16){
    // Setup Ethernet header
    let mut eth_header = pnet::packet::ethernet::MutableEthernetPacket::new(&mut tmp_packet[..ethernet::ETHERNET_HEADER_LEN]).unwrap();
    ethernet::build_ethernet_packet(&mut eth_header, tcp_options.sender_mac, tcp_options.target_mac, ethernet::EtherType::Ipv4);
    // Setup IP header
    let mut ip_header = pnet::packet::ipv4::MutableIpv4Packet::new(&mut tmp_packet[ethernet::ETHERNET_HEADER_LEN..(ethernet::ETHERNET_HEADER_LEN + ipv4::IPV4_HEADER_LEN)]).unwrap();
    ipv4::build_ipv4_packet(&mut ip_header, tcp_options.src_ip, tcp_options.dst_ip, ipv4::IpNextHeaderProtocol::Tcp);
    // Setup TCP header
    let mut tcp_header = pnet::packet::tcp::MutableTcpPacket::new(&mut tmp_packet[(ethernet::ETHERNET_HEADER_LEN + ipv4::IPV4_HEADER_LEN)..]).unwrap();
    tcp::build_tcp_packet(&mut tcp_header, tcp_options.src_ip, tcp_options.src_port, tcp_options.dst_ip, target_port, &tcp_options.scan_type);
}

fn send_tcp_packet(tx: &mut Box<dyn pnet::datalink::DataLinkSender>, tcp_options: &TcpOptions, stop: &Arc<Mutex<bool>>) {
    for i in tcp_options.min_port_num..tcp_options.max_port_num + 1 {
        thread::sleep(time::Duration::from_millis(1));
        tx.build_and_send(1, 66, &mut |packet: &mut [u8]| {
            build_packet(&tcp_options, packet, i);
        });
    }
    *stop.lock().unwrap() = true;
}

fn receive_packets(
    rx: &mut Box<dyn pnet::datalink::DataLinkReceiver>, 
    tcp_options: &TcpOptions, 
    open_ports: &Arc<Mutex<Vec<String>>>, 
    stop: &Arc<Mutex<bool>>, 
    scan_status: &Arc<Mutex<ScanStatus>>) {
    let start_time = Instant::now();
    loop {
        match rx.next() {
            Ok(frame) => {
                let frame = pnet::packet::ethernet::EthernetPacket::new(frame).unwrap();
                match frame.get_ethertype() {
                    pnet::packet::ethernet::EtherTypes::Ipv4 => {
                        ipv4_handler(&frame, &tcp_options, &open_ports);
                    },
                    pnet::packet::ethernet::EtherTypes::Ipv6 => {
                        ipv6_handler(&frame, &tcp_options, &open_ports);
                    }
                    _ => {
                        //println!("Not a ipv4 or ipv6");
                    }
                }
            },
            Err(e) => {
                panic!("Failed to read: {}", e);
            }
        }
        if *stop.lock().unwrap(){
            *scan_status.lock().unwrap() = ScanStatus::Done;
            break;
        }
        if Instant::now().duration_since(start_time) > tcp_options.timeout {
            *scan_status.lock().unwrap() = ScanStatus::Timeout;
            break;
        }
    }
}

fn ipv4_handler(ethernet: &pnet::packet::ethernet::EthernetPacket, tcp_options: &TcpOptions, open_ports: &Arc<Mutex<Vec<String>>>) {
    if let Some(packet) = pnet::packet::ipv4::Ipv4Packet::new(ethernet.payload()){
        match packet.get_next_level_protocol() {
            pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet, &tcp_options, &open_ports);
            },
            pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                udp_handler(&packet, &tcp_options, &open_ports);
            },
            _ => {
                //println!("Not a tcp or a udp packet");
            }
        }
    }
}

fn ipv6_handler(ethernet: &pnet::packet::ethernet::EthernetPacket, tcp_options: &TcpOptions, open_ports: &Arc<Mutex<Vec<String>>>) {
    if let Some(packet) = pnet::packet::ipv6::Ipv6Packet::new(ethernet.payload()){
        match packet.get_next_header() {
            pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet, &tcp_options, &open_ports);
            },
            pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                udp_handler(&packet, &tcp_options, &open_ports);
            },
            _ => {
                //println!("Not a tcp or a udp packet");
            }
        }
    }
}

fn tcp_handler(packet: &dyn EndPoints, tcp_options: &TcpOptions, open_ports: &Arc<Mutex<Vec<String>>>) {
    let tcp = pnet::packet::tcp::TcpPacket::new(packet.get_payload());
    if let Some(tcp) = tcp {
        append_packet_info(packet, &tcp, &tcp_options, &open_ports);
    }
}

fn udp_handler(packet: &dyn EndPoints, tcp_options: &TcpOptions, open_ports: &Arc<Mutex<Vec<String>>>) {
    let udp = pnet::packet::udp::UdpPacket::new(packet.get_payload());
    if let Some(udp) = udp {
        append_packet_info(packet, &udp, &tcp_options, &open_ports);
    }
}

fn append_packet_info(_l3: &dyn EndPoints, l4: &dyn EndPoints, tcp_options: &TcpOptions, open_ports: &Arc<Mutex<Vec<String>>>) {
    if l4.get_destination() == tcp_options.src_port.to_string() {
        if !open_ports.lock().unwrap().contains(&l4.get_source()){
            open_ports.lock().unwrap().push(l4.get_source());
        }
    }
}
