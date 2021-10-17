use std::time::Duration;

/// Scan task status for each scanner 
#[derive(Clone, Copy, Debug)]
pub enum ScanStatus {
    Ready,
    Done,
    Timeout,
    Error,
}

/// Type of port scan 
/// 
/// Supports TCP SYN Scan, TCP CONNECT Scan
#[derive(Clone, Copy, Debug)]
pub enum PortScanType {
    SynScan,
    ConnectScan,
}

/// Status of port that responded 
#[derive(Clone, Copy, Debug)]
pub enum PortStatus {
    Open,
    Closed,
    Filtered,
}

/// Information on each port that responded
#[derive(Clone, Copy, Debug)]
pub struct PortInfo {
    pub port: u16,
    pub status: PortStatus,
}

/// Result of HostScanner::run_scan  
#[derive(Clone, Debug)]
pub struct HostScanResult {
    /// List of up host  
    pub up_hosts: Vec<String>,
    /// Time from start to end of scan  
    pub scan_time: Duration,
    /// Scan job status
    pub scan_status: ScanStatus,
}

impl HostScanResult {
    pub fn new() -> HostScanResult {
        HostScanResult{
            up_hosts: vec![],
            scan_time: Duration::from_millis(0),
            scan_status: ScanStatus::Ready,
        }
    }
}

/// Result of PortScanner::run_scan  
#[derive(Clone, Debug)]
pub struct PortScanResult {
    /// List of open port  
    pub ports: Vec<PortInfo>,
    /// Time from start to end of scan  
    pub scan_time: Duration,
    /// Scan job status
    pub scan_status: ScanStatus,
}

impl PortScanResult {
    pub fn new() -> PortScanResult {
        PortScanResult{
            ports: vec![],
            scan_time: Duration::from_millis(0),
            scan_status: ScanStatus::Ready,
        }
    }
}