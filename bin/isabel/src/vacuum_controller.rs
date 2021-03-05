use std::sync::Arc;
use std::time::Duration;

use elisheba::{Command, CommandResponse};
use log::{error, info};
use tokio::{
    sync::{mpsc, Mutex},
    task,
    time::interval,
};

use crate::{vacuum::Status, FanSpeed, Vacuum};

#[derive(Clone)]
pub struct VacuumController {
    vacuum: Arc<Mutex<Vacuum>>,
}

impl VacuumController {
    pub fn new(vacuum: Vacuum) -> Self {
        Self {
            vacuum: Arc::from(Mutex::from(vacuum)),
        }
    }

    pub async fn handle_vacuum_command(&self, command: Command) -> CommandResponse {
        let mut vacuum = self.vacuum.clone().lock_owned().await;

        let (command, result) = match command {
            Command::Start { rooms } => {
                info!("wants to start cleaning in rooms: {:?}", rooms);
                ("start", vacuum.start(rooms).await)
            }
            Command::Stop => {
                info!("wants to stop cleaning");
                ("stop", vacuum.stop().await)
            }
            Command::GoHome => {
                info!("wants to go home");
                ("go home", vacuum.go_home().await)
            }
            Command::SetWorkSpeed { mode } => {
                info!("wants to set mode {}", mode);
                let mode = FanSpeed::from(mode);
                ("set mode", vacuum.set_fan_speed(mode).await)
            }
        };

        match result {
            Ok(_) => {
                info!("ok {}", command);
                CommandResponse::Ok
            }
            Err(err) => {
                error!("err {} {}", command, err);
                CommandResponse::Failure
            }
        }
    }

    pub fn observe_vacuum_status(&self) -> mpsc::Receiver<Status> {
        let vacuum = self.vacuum.clone();
        let (tx, rx) = mpsc::channel::<Status>(1);

        task::spawn(async move {
            let mut timer = interval(Duration::from_secs(60));

            loop {
                timer.tick().await;

                let mut vacuum = vacuum.clone().lock_owned().await;
                let tx = tx.clone();

                if let Ok(status) = vacuum.status().await {
                    if let Err(_) = tx.send(status).await {
                        info!("status observer has been dropped, aborting timer");
                        break;
                    }
                }
            }
        });

        rx
    }
}
