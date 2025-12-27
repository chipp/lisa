mod constants;
mod hci;

use constants::{
    EVT_LE_ADVERTISING_REPORT, EVT_LE_META_EVENT, HCIDEVDOWN, HCIDEVUP, HCI_EVENT_HDR_SIZE,
    HCI_EVENT_PKT, HCI_FILTER, HCI_MAX_EVENT_SIZE, SOL_HCI,
};
use hci::{
    hci_close_dev, hci_filter_set_event, hci_filter_set_ptype, hci_get_route,
    hci_le_set_scan_enable, hci_le_set_scan_parameters, hci_open_dev, htobs, EvtLeMetaEvent,
    HciFilter, LeAdvertisingInfo,
};

use libc::{c_int, c_void};
use log::{debug, error, info, trace};
use tokio::sync::mpsc::{self, Receiver, Sender};

use std::{
    io::Error as IoError,
    mem::MaybeUninit,
    sync::{Arc, Mutex},
};

use crate::{event::parse_event, Event, MacAddr};
use super::{Error, Result};

pub struct Scanner {
    dd: Arc<Mutex<c_int>>,
}

impl super::ScannerTrait for Scanner {
    fn new() -> Result<Scanner> {
        let dev_id = unsafe { hci_get_route(std::ptr::null()) };

        let dd = unsafe { hci_open_dev(dev_id) };
        if dd < 0 {
            return Err(Error::new("could not open device", IoError::last_os_error()));
        }

        unsafe {
            if libc::ioctl(dd, HCIDEVDOWN, dev_id) < 0 {
                hci_close_dev(dd);
                return Err(Error::new("could not down hci device", IoError::last_os_error()));
            }

            if libc::ioctl(dd, HCIDEVUP, dev_id) < 0 {
                hci_close_dev(dd);
                return Err(Error::new("could not up hci device", IoError::last_os_error()));
            }
        }

        Ok(Scanner {
            dd: Arc::from(Mutex::from(dd)),
        })
    }

    fn start_scan(&mut self) -> Result<Receiver<(MacAddr, Event)>> {
        unsafe {
            let dd = *self.dd.lock().unwrap();
            if hci_le_set_scan_parameters(dd, 0x01, htobs(0x0010), htobs(0x0010), 0x00, 0x00, 1000)
                < 0
            {
                hci_close_dev(dd);
                return Err(Error::new(
                    "set scan parameters failed",
                    IoError::last_os_error(),
                ));
            }
        }

        unsafe {
            let dd = *self.dd.lock().unwrap();
            if hci_le_set_scan_enable(dd, 0x01, 0x00, 1000) < 0 {
                hci_close_dev(dd);
                return Err(Error::new("enable scan failed", IoError::last_os_error()));
            }
        }

        info!("started LE scanning...");

        let (tx, rx) = mpsc::channel(1);
        let dd = self.dd.clone();

        std::thread::spawn(move || unsafe { Self::read_events(dd, tx) });

        Ok(rx)
    }
}

impl Scanner {
    unsafe fn stop_scan(dd: c_int) {
        let err = hci_le_set_scan_enable(dd, 0x00, 0x00, 1000);
        if err < 0 {
            error!("Disable scan failed: {}", IoError::last_os_error());
        }

        hci_close_dev(dd);
    }

    unsafe fn read_events(dd: Arc<Mutex<c_int>>, tx: Sender<(MacAddr, Event)>) {
        let dd = *dd.lock().unwrap();

        let mut nf = MaybeUninit::<HciFilter>::zeroed();

        let mut of = MaybeUninit::<HciFilter>::uninit();
        let mut olen = std::mem::size_of::<HciFilter>() as libc::socklen_t;

        if libc::getsockopt(
            dd,
            SOL_HCI,
            HCI_FILTER,
            of.as_mut_ptr() as *mut c_void,
            &mut olen,
        ) < 0
        {
            error!("Could not get socket options: {}", IoError::last_os_error());
            return;
        }

        hci_filter_set_ptype(HCI_EVENT_PKT, nf.as_mut_ptr());
        hci_filter_set_event(EVT_LE_META_EVENT, nf.as_mut_ptr());

        if libc::setsockopt(
            dd,
            SOL_HCI,
            HCI_FILTER,
            nf.as_ptr() as *const c_void,
            std::mem::size_of::<HciFilter>() as u32,
        ) < 0
        {
            error!("Could not set socket options: {}", IoError::last_os_error());
            return;
        }

        let mut buf = [0; HCI_MAX_EVENT_SIZE as usize];
        let mut len;

        loop {
            trace!("reading bluetooh stream");

            let meta: *const EvtLeMetaEvent;
            let info: *const LeAdvertisingInfo;

            len = libc::read(
                dd,
                buf.as_mut_ptr() as *mut c_void,
                std::mem::size_of_val(&buf),
            );

            trace!("read {} bytes", len);

            while len < 0 {
                len = libc::read(
                    dd,
                    buf.as_mut_ptr() as *mut c_void,
                    std::mem::size_of_val(&buf),
                );
            }

            let ptr: *const u8 = buf.as_ptr().offset(1 + HCI_EVENT_HDR_SIZE);
            meta = ptr.cast();

            if (*meta).subevent != EVT_LE_ADVERTISING_REPORT {
                trace!("got meta.subevent {:x}", (*meta).subevent);
                continue;
            }

            let ptr: *const u8 = (*meta).data.as_ptr().offset(1);
            info = ptr.cast();

            let mut event = None;
            let mut offset = 0;
            trace!("info.length: {}", (*info).length);

            while offset < (*info).length {
                trace!("offset: {}", offset);

                let eir_data_len = *(*info).data.as_ptr().offset(offset as isize);

                if eir_data_len == 0 {
                    break;
                }

                if let Some(evt) = Self::read_event((*info).data.as_ptr().offset(offset as isize)) {
                    event = Some(evt);
                    break;
                }
                offset += eir_data_len + 1;
            }

            if let Some(event) = event.as_ref().map(Vec::as_slice).and_then(parse_event) {
                if let Err(_) = tx.blocking_send(event) {
                    debug!("scanner observer has been dropped, cancelling scanning");
                    break;
                }
            }
        }

        if libc::setsockopt(
            dd,
            SOL_HCI,
            HCI_FILTER,
            of.as_ptr() as *const c_void,
            std::mem::size_of_val(&of) as u32,
        ) < 0
        {
            error!(
                "failed to reset socket options: {}",
                IoError::last_os_error()
            );
        }
    }

    fn read_event(data: *const u8) -> Option<Vec<u8>> {
        let len = unsafe { *data };

        let r#type = unsafe { *(data.offset(1)) };

        if r#type == 0x16 {
            let uuid = unsafe { (*data.offset(2).cast::<u16>()).to_le() };
            if uuid == 0xfe95 {
                let slice =
                    unsafe { std::slice::from_raw_parts(data.offset(4), (len - 2) as usize) };
                Some(Vec::from(slice))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        unsafe {
            debug!("Scanner has been dropped");
            let dd = *self.dd.lock().unwrap();
            Scanner::stop_scan(dd);
        }
    }
}
