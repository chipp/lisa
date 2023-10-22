use libc::{c_char, c_int, c_uint};

pub const HCI_MAX_EVENT_SIZE: c_int = 260;

pub const SOL_HCI: c_int = 0;

pub const HCI_FILTER: c_int = 2;

pub const HCI_VENDOR_PKT: c_int = 0xff;

pub const HCI_FLT_TYPE_BITS: c_int = 31;
pub const HCI_FLT_EVENT_BITS: c_int = 63;

pub const HCI_EVENT_PKT: c_int = 0x04;

pub const EVT_LE_META_EVENT: c_int = 0x3E;

pub const HCI_EVENT_HDR_SIZE: isize = 2;

// ioctl
// TODO: add arch attributes

const _IOC_WRITE: c_uint = 1;

const fn _ioc(dir: c_uint, r#type: c_char, nr: c_int, size: usize) -> c_int {
    ((dir as c_int) << 30) | ((r#type as c_int) << 8) | (nr as c_int) | ((size as c_int) << 16)
}

const fn _iow<T>(r#type: c_char, nr: c_int) -> c_int {
    _ioc(_IOC_WRITE, r#type, nr, std::mem::size_of::<T>())
}

pub const HCIDEVUP: c_int = _iow::<c_int>(b'H' as c_char, 201);
pub const HCIDEVDOWN: c_int = _iow::<c_int>(b'H' as c_char, 202);

pub const EVT_LE_ADVERTISING_REPORT: u8 = 0x02;
