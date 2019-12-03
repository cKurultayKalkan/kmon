use crate::kernel::log::KernelLogs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;

/* Terminal events enumerator */
pub enum Event<I> {
    Input(I),
    Kernel(String),
    Tick,
}

/* Terminal events struct */
#[allow(dead_code)]
pub struct Events {
    pub rx: mpsc::Receiver<Event<Key>>,
    input_handler: thread::JoinHandle<()>,
    kernel_handler: thread::JoinHandle<()>,
    tick_handler: thread::JoinHandle<()>,
}

/* Terminal events implementation */
impl Events {
    /**
     * Create a new events instance.
     *
     * @param  refresh_rate
     * @return Events
     */
    pub fn new(refresh_rate: Duration) -> Self {
        /* Create a new asynchronous channel. */
        let (tx, rx) = mpsc::channel();
        /* Handle inputs using stdin stream and sender of the channel. */
        let input_handler = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    match evt {
                        Ok(key) => {
                            tx.send(Event::Input(key)).unwrap();
                        }
                        Err(_) => {}
                    }
                }
            })
        };
        /* Handle kernel logs using 'dmesg' output. */
        let kernel_handler = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                let mut kernel_logs = KernelLogs::new();
                loop {
                    if kernel_logs.update() {
                        tx.send(Event::Kernel(kernel_logs.output.to_string()))
                            .unwrap();
                    }
                    thread::sleep(refresh_rate * 10);
                }
            })
        };
        /* Create a loop for handling events. */
        let tick_handler = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(refresh_rate);
                }
            })
        };
        /* Return events. */
        Self {
            rx,
            input_handler,
            kernel_handler,
            tick_handler,
        }
    }
}

/**
 * Unit tests.
 */
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_events() -> Result<(), failure::Error> {
        let events = Events::new(Duration::from_millis(250));
        match events.rx.recv()? {
            Event::Input(_) => Ok(()),
            Event::Tick => Ok(()),
            Event::Kernel(logs) => {
                if logs.len() > 0 {
                    Ok(())
                } else {
                    Err(failure::err_msg("failed to retrieve kernel logs"))
                }
            }
        }
    }
}