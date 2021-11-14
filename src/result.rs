use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use std::collections::HashSet;

/// Status of scan task 
#[derive(Clone, Debug)]
pub enum ScanStatus {
    Ready,
    Done,
    Timeout,
    Error,
}

/// Status of the scanned port 
#[derive(Clone, Copy, Debug)]
pub enum PortStatus {
    Open,
    Closed,
    Filtered,
}

/// Information about the scanned host 
#[derive(Clone, Copy, Debug)]
pub struct HostInfo {
    pub ip_addr: IpAddr,
    pub ttl: u8,
}

/// Information about the scanned port 
#[derive(Clone, Copy, Debug)]
pub struct PortInfo {
    pub port: u16,
    pub status: PortStatus,
}

/// Result of host scan 
#[derive(Clone, Debug)]
pub struct HostScanResult {
    pub hosts: Vec<HostInfo>,
    pub scan_time: Duration,
    pub scan_status: ScanStatus,
}

impl HostScanResult {
    pub fn new() -> HostScanResult {
        HostScanResult{
            hosts: vec![],
            scan_time: Duration::from_millis(0),
            scan_status: ScanStatus::Ready,
        }
    }
}

/// Result of port scan
#[derive(Clone, Debug)]
pub struct PortScanResult {
    pub ip_addr: IpAddr,  
    pub ports: Vec<PortInfo>,
    pub scan_time: Duration,
    pub scan_status: ScanStatus,
}

impl PortScanResult {
    pub fn new() -> PortScanResult {
        PortScanResult{
            ip_addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            ports: vec![],
            scan_time: Duration::from_millis(0),
            scan_status: ScanStatus::Ready,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct ScanResult {
    pub host_scan_result: HostScanResult,
    pub port_scan_result: PortScanResult,
    pub ip_set: HashSet<IpAddr>,
    pub port_set: HashSet<u16>,
}

impl ScanResult {
    pub fn new() -> ScanResult {
        ScanResult {
            host_scan_result: HostScanResult::new(),
            port_scan_result: PortScanResult::new(),
            ip_set: HashSet::new(),
            port_set: HashSet::new(),
        }
    }
}
