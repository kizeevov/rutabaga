use iced::futures::channel::oneshot::{self, Receiver, Sender};
use iced::futures::StreamExt;
use iced_native::{subscription, Subscription};
use itertools::Itertools;
use std::arch::aarch64::vreinterpret_u8_f32;
use std::fs::{self, DirEntry, ReadDir};
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::ops::Not;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub fn clear_folder(process: ClearProcess) -> Subscription<Progress> {
    subscription::unfold(process.clone(), State::Ready(process), move |state| {
        clearing_process(state)
    })
}

async fn clearing_process(state: State) -> (Option<Progress>, State) {
    match state {
        State::Ready(process) => {
            let result = fs::read_dir(&process.path);

            let paths = match result {
                Ok(paths) => paths,
                Err(_) => return (Some(Progress::Errored), State::Finished),
            };

            let paths = paths
                .filter_map(|p| match p {
                    Ok(f) => {
                        if f.path().is_file() {
                            Some(f)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                })
                .collect_vec();

            (
                Some(Progress::Started { total: paths.len() }),
                State::Process {
                    renamed: 0,
                    cleared: 0,
                    process,
                    paths: Arc::new(RwLock::new(paths.into_iter().enumerate())),
                },
            )
        }
        State::Process {
            renamed,
            cleared,
            process,
            paths,
        } => {
            if process.is_canceled() {
                return (Some(Progress::Canceled), State::Finished);
            }

            let (index, dir_entry) = {
                let mut guard = paths.write().await;

                let result = guard.next();
                match result {
                    None => return (Some(Progress::Finished), State::Finished),
                    Some(result) => result,
                }
            };

            if dir_entry.path().is_file().not() {
                return (
                    Some(Progress::Advanced { renamed, cleared }),
                    State::Process {
                        renamed,
                        cleared,
                        process,
                        paths,
                    },
                );
            }

            // println!("fillele {:?}", dir_entry.path());
            // tokio::time::sleep(Duration::from_secs(1)).await;

            let mut renamed = renamed;
            let mut cleared = cleared;

            let old_path = dir_entry.path().clone();
            let mut new_path = dir_entry.path();

            for i in 0.. {
                let name = if i == 0 {
                    format!("File{index}")
                } else {
                    format!("File{index}({i})")
                };

                new_path.set_file_name(name);
                if new_path.exists().not() {
                    break;
                }
            }
            new_path.set_extension("txt");

            let result = tokio::fs::rename(old_path, &new_path).await;
            match result {
                Ok(_) => renamed += 1,
                Err(_) => {
                    return (
                        Some(Progress::Advanced { renamed, cleared }),
                        State::Process {
                            renamed,
                            cleared,
                            process,
                            paths,
                        },
                    )
                }
            }

            match tokio::fs::write(&new_path, &vec![]).await {
                Ok(_) => cleared += 1,
                Err(_) => {
                    return (
                        Some(Progress::Advanced { renamed, cleared }),
                        State::Process {
                            renamed,
                            cleared,
                            process,
                            paths,
                        },
                    )
                }
            }

            (
                Some(Progress::Advanced { renamed, cleared }),
                State::Process {
                    renamed,
                    cleared,
                    process,
                    paths,
                },
            )
        }
        State::Finished => iced::futures::future::pending().await,
    }
}

#[derive(Debug, Clone)]
pub struct ClearProcess {
    path: PathBuf,
    canceled: Arc<AtomicBool>,
}

impl Hash for ClearProcess {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

impl ClearProcess {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            canceled: Default::default(),
        }
    }

    pub fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::SeqCst)
    }

    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started { total: usize },
    Advanced { renamed: usize, cleared: usize },
    Canceled,
    Finished,
    Errored,
}

#[derive(Debug, Clone)]
pub enum State {
    Ready(ClearProcess),
    Process {
        renamed: usize,
        cleared: usize,
        process: ClearProcess,
        paths: Arc<RwLock<std::iter::Enumerate<std::vec::IntoIter<DirEntry>>>>,
    },
    Finished,
}
