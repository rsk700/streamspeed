use std::io::{self, Read, Write};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::collections::HashMap;

struct ReadableSpeed {
    speed: f64,
    title: String,
}

impl ReadableSpeed {
    fn from_bytes(speed: f64) -> ReadableSpeed {
        if speed < 0.0 {
            ReadableSpeed {
                speed: 0.0,
                title: "B/s".to_string(),
            }
        } else if speed < 1024.0 {
            ReadableSpeed {
                speed: speed,
                title: "B/s".to_string(),
            }
        } else if speed < 1024_f64.powi(2) {
            ReadableSpeed {
                speed: speed / 1024.0,
                title: "KiB/s".to_string(),
            }
        } else if speed < 1024_f64.powi(3) {
            ReadableSpeed {
                speed: speed / 1024_f64.powi(2),
                title: "MiB/s".to_string(),
            }
        } else if speed < 1024_f64.powi(4) {
            ReadableSpeed {
                speed: speed / 1024_f64.powi(3),
                title: "GiB/s".to_string(),
            }
        } else if speed < 1024_f64.powi(5) {
            ReadableSpeed {
                speed: speed / 1024_f64.powi(4),
                title: "TiB/s".to_string(),
            }
        } else if speed < 1024_f64.powi(6) {
            ReadableSpeed {
                speed: speed / 1024_f64.powi(5),
                title: "PiB/s".to_string(),
            }
        } else if speed < 1024_f64.powi(7) {
            ReadableSpeed {
                speed: speed / 1024_f64.powi(6),
                title: "EiB/s".to_string(),
            }
        } else if speed < 1024_f64.powi(8) {
            ReadableSpeed {
                speed: speed / 1024_f64.powi(7),
                title: "ZiB/s".to_string(),
            }
        } else {
            ReadableSpeed {
                speed: speed / 1024_f64.powi(8),
                title: "YiB/s".to_string(),
            }
        }
    }
}

fn show_speed(start: Instant, ops: Arc<Mutex<HashMap<u64, usize>>>) {
    loop {
        thread::sleep(Duration::from_secs(1));
        {
            let ops = match ops.lock() {
                Result::Ok(ops) => ops,
                Result::Err(_) => return,
            };
            let total_spent = Instant::now().duration_since(start).as_secs();
            // speed calculated based on last 10 seconds
            let mut start_stat: u64 = 0;
            if total_spent >= 10 {
                start_stat = total_spent - 10;
            }
            let duration = total_spent - start_stat;
            let mut speed: f64 = 0.0;

            if duration > 0 {
                let mut size: usize = 0;
                for (spent, current_size) in ops.iter() {
                    if *spent >= start_stat {
                        size += *current_size;
                    }
                }
                let size: f64 = size as f64;
                let duration: f64 = duration as f64;
                speed = size / duration;
            }
            let readable = ReadableSpeed::from_bytes(speed);
            writeln!(&mut std::io::stderr(),
                     "{:.2} {}",
                     readable.speed,
                     readable.title).unwrap();
        }
    }
}

fn count_bytes(rx: Receiver<usize>) {
    let start = Instant::now();
    let ops: Arc<Mutex<HashMap<u64, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let ops_clone = ops.clone();
    thread::spawn(move || {
        show_speed(start, ops_clone);
    });

    loop {
        let mut size: usize;
        match rx.recv() {
            Result::Ok(s) => size = s,
            Result::Err(_) => return,
        }
        let spent = (Instant::now() - start).as_secs();
        {
            let mut ops = match ops.lock() {
                Result::Ok(ops) => ops,
                Result::Err(_) => return,
            };
            size += *ops.entry(spent).or_insert(0);
            ops.insert(spent, size);
        }
    }
}

fn main() {
    let mut buf = [0; 4096];
    let mut std_in = io::stdin();
    let mut std_out = io::stdout();
    let (tx, rx) = channel();
    thread::spawn(move || {
        count_bytes(rx);
    });
    loop {
        let size = std_in.read(&mut buf).unwrap();
        if size == 0 {
            break;
        }
        std_out.write(&buf[0..size]).unwrap();
        // send does not blocks
        tx.send(size).unwrap();
    }
}
