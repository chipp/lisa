use core_bluetooth::central::{CentralEvent, CentralManager};
use core_bluetooth::uuid::Uuid;
use core_bluetooth::ManagerState;

use log::{debug, trace};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{event::parse_event, Event, MacAddr};

pub struct Scanner;

impl super::CommonScanner for Scanner {
    fn new() -> Scanner {
        Scanner
    }

    fn start_scan(&mut self) -> Receiver<(MacAddr, Event)> {
        let (tx, rx) = mpsc::channel(1);

        std::thread::spawn(move || {
            let (central, rx) = CentralManager::new();

            for event in rx {
                let tx = tx.clone();
                Self::handle_event(event, tx, &central);
            }
        });

        rx
    }
}

impl Scanner {
    fn handle_event(event: CentralEvent, tx: Sender<(MacAddr, Event)>, central: &CentralManager) {
        trace!("{:?}", event);

        match event {
            CentralEvent::ManagerStateChanged { new_state }
                if new_state == ManagerState::PoweredOn =>
            {
                central.scan();
            }
            CentralEvent::PeripheralDiscovered {
                advertisement_data, ..
            } => {
                let uuid = Uuid::from_slice(&[0xfe, 0x95]);

                if let Some(event) = advertisement_data
                    .service_data()
                    .get(uuid)
                    .and_then(parse_event)
                {
                    if let Err(_) = tx.blocking_send(event) {
                        debug!("scanner observer has been dropped, cancelling scanning");
                        central.cancel_scan();
                    }
                }
            }
            _ => (),
        }
    }
}
