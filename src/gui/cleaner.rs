use iced::futures::channel::oneshot;
use iced::futures::channel::oneshot::Receiver;
use iced_native::{subscription, Subscription};
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub fn clear_folder(path: PathBuf) -> Subscription<Progress> {
    struct SomeWorker;

    subscription::unfold(path.clone(), State::Ready(path), move |state| {
        clearing_process(state)
    })
}

async fn clearing_process(state: State) -> (Option<Progress>, State) {
    match state {
        State::Ready(path) => {
            let k = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            let path = PathBuf::from(format!("{path:?}{k:?}"));

            let (sender, receiver) = oneshot::channel::<()>();

            (
                Some(Progress::Started),
                State::Process {
                    path,
                    canceled: receiver,
                },
            )
        }
        State::Process { path, canceled } => {
            for _ in 0..10 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("test {path:?}");
            }

            (Some(Progress::Finished), State::Finished)
        }
        State::Finished => {
            println!("test2");
            iced::futures::future::pending().await
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct ClearConfig {
//     path: PathBuf,
//     canceled: AtomicBool,
// }
//
// impl ClearConfig {
//     pub fn new(path: PathBuf) -> Self {
//         Self {
//             path,
//             canceled: Default::default(),
//         }
//     }
//
//     pub fn cancel(&self) {
//         self.canceled.store(true, Ordering::SeqCst)
//     }
// }

#[derive(Debug, Clone)]
pub enum Progress {
    Started {
        canceled: Sender<()>,
    },
    Advanced {
        renamed: usize,
        cleared: usize,
        total: usize,
    },
    Finished,
    Errored,
}

#[derive(Debug, Clone)]
pub enum State {
    Ready(PathBuf),
    Process {
        path: PathBuf,
        canceled: Receiver<()>,
    },
    Finished,
}
