use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn default() -> Events {
        Events::with_tick(Duration::from_millis(200))
    }

    pub fn with_tick(tick_duration: Duration) -> Events {
        let (tx, rx) = mpsc::channel();

        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = std::io::stdin();
                for evt in stdin.keys() {
                    match evt {
                        Ok(key) => {
                            if let Err(_) = tx.send(Event::Input(key)) {
                                return;
                            }
                        }
                        Err(_) => {}
                    }
                }
            })
        };

        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(tick_duration);
            })
        };

        Events {
            rx: rx,
            input_handle: input_handle,
            tick_handle: tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
