use super::constants::{HCI_FLT_EVENT_BITS, HCI_FLT_TYPE_BITS, HCI_VENDOR_PKT};

use libc::{c_int, c_uint};

#[repr(C, packed)]
#[derive(Debug)]
pub struct BdAddr {
    pub b: [u8; 6],
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct HciFilter {
    pub type_mask: u32,
    pub event_mask: [u32; 2],
    pub opcode: u16,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct EvtLeMetaEvent {
    pub subevent: u8,
    pub data: [u8; 0],
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct LeAdvertisingInfo {
    pub evt_type: u8,
    pub bdaddr_type: u8,
    pub bdaddr: BdAddr,
    pub length: u8,
    pub data: [u8; 0],
}

#[link(name = "bluetooth", kind = "static")]
extern "C" {
    pub fn hci_get_route(bdaddr: *const BdAddr) -> c_int;
    pub fn hci_open_dev(dev_id: c_int) -> c_int;
    pub fn hci_close_dev(dd: c_int) -> c_int;

    pub fn hci_le_set_scan_parameters(
        dev_id: c_int,
        r#type: u8,
        interval: u16,
        window: u16,
        own_type: u8,
        filter: u8,
        to: c_int,
    ) -> c_int;

    pub fn hci_le_set_scan_enable(dev_id: c_int, enable: u8, filter_dup: u8, to: c_int) -> c_int;
}

#[inline]
pub fn htobs(value: u16) -> u16 {
    value.to_le()
}

#[inline]
pub fn hci_set_bit(nr: c_int, addr: *mut c_uint) {
    let bitset = unsafe { addr.offset((nr >> 5) as isize).as_mut() }.unwrap();
    *bitset |= 1 << (nr & 31);
}

#[inline]
pub fn hci_filter_set_ptype(t: c_int, f: *mut HciFilter) {
    unsafe {
        let nr = if t == HCI_VENDOR_PKT {
            0
        } else {
            t & HCI_FLT_TYPE_BITS
        };
        hci_set_bit(nr, &mut (&mut *f).type_mask as *mut c_uint);
    }
}

#[inline]
pub fn hci_filter_set_event(e: c_int, f: *mut HciFilter) {
    unsafe {
        hci_set_bit(
            e & HCI_FLT_EVENT_BITS,
            &mut (&mut *f).event_mask[0] as *mut c_uint,
        );
    }
}
