use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::stream;
use iced::Subscription;
use stratum_ipc::IpcClient;

use crate::app::Message;

/// Returns an Iced subscription that connects to stratum-wm's IPC socket and
/// forwards every received message into the Iced event loop.
///
/// Automatically reconnects with a 2-second delay when the connection drops.
pub fn ipc_subscription() -> Subscription<Message> {
    struct IpcSub;

    Subscription::run_with_id(
        std::any::TypeId::of::<IpcSub>(),
        stream::channel(64, |mut sender: mpsc::Sender<Message>| async move {
            loop {
                match IpcClient::connect().await {
                    Ok(mut client) => loop {
                        match client.recv().await {
                            Ok(msg) => {
                                if sender.try_send(Message::IpcEvent(msg)).is_err() {
                                    return;
                                }
                            }
                            Err(_) => break,
                        }
                    },
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }),
    )
}
