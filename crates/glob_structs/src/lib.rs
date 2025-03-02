use ipc_channel::ipc::{IpcReceiver, IpcSender};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub enum IpcMsg {
    IpcSender(IpcSender<IpcMsg>),
    IpcReceiver(IpcReceiver<IpcMsg>),
    BookISCaching(String),
    BookToCaching(String),
    ExitSignalForMupdf
}

