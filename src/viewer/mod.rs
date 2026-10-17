//! Portable loopback-only browser client. Scientific state and stepping remain in Rust.
pub mod app;
mod http;
use http::Context;
use std::{
    io,
    net::TcpListener,
    sync::{atomic::Ordering, mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub struct Server {
    listener: TcpListener,
    context: Arc<Context>,
    idle: Duration,
}
impl Server {
    pub fn bind(port: u16, idle: Duration) -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        listener.set_nonblocking(true)?;
        let host = format!("127.0.0.1:{}", listener.local_addr()?.port());
        let context = Arc::new(Context {
            app: Mutex::new(app::App::new().map_err(io::Error::other)?),
            stop: Default::default(),
            activity: Mutex::new(Instant::now()),
            token: token()?,
            origin: format!("http://{host}"),
            host,
        });
        Ok(Self {
            listener,
            context,
            idle,
        })
    }
    pub fn url(&self) -> String {
        format!("{}/{}/", self.context.origin, self.context.token)
    }
    pub fn serve(self) -> io::Result<()> {
        let (tx, rx) = mpsc::sync_channel(8);
        let rx = Arc::new(Mutex::new(rx));
        let mut workers = Vec::new();
        for _ in 0..4 {
            let rx = rx.clone();
            let context = self.context.clone();
            workers.push(thread::spawn(move || loop {
                let stream = match rx.lock() {
                    Ok(r) => r.recv(),
                    Err(_) => break,
                };
                match stream {
                    Ok(s) => http::handle(s, &context),
                    Err(_) => break,
                }
            }));
        }
        let mut last = Instant::now();
        let mut failure = None;
        while !self.context.stop.load(Ordering::Relaxed) {
            let now = Instant::now();
            let elapsed = now.duration_since(last);
            last = now;
            if let Ok(mut app) = self.context.app.lock() {
                app.tick(elapsed);
            } else {
                failure = Some(io::Error::other("simulation lock poisoned"));
                break;
            }
            if self
                .context
                .activity
                .lock()
                .map(|t| t.elapsed() > self.idle)
                .unwrap_or(true)
            {
                break;
            }
            for _ in 0..16 {
                match self.listener.accept() {
                    Ok((stream, _)) => match tx.try_send(stream) {
                        Ok(()) => {}
                        Err(mpsc::TrySendError::Full(mut s)) => http::busy(&mut s),
                        Err(_) => {
                            failure = Some(io::Error::other("worker pool closed"));
                            break;
                        }
                    },
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => {
                        failure = Some(e);
                        break;
                    }
                }
            }
            if failure.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        drop(tx);
        for worker in workers {
            let _ = worker.join();
        }
        if let Some(e) = failure {
            Err(e)
        } else {
            Ok(())
        }
    }
}
fn token() -> io::Result<String> {
    let mut bytes = [0u8; 16];
    #[cfg(windows)]
    {
        #[link(name = "bcrypt")]
        unsafe extern "system" {
            fn BCryptGenRandom(
                algorithm: *mut std::ffi::c_void,
                buffer: *mut u8,
                length: u32,
                flags: u32,
            ) -> i32;
        }
        // OS-provided CSPRNG, no persistent secret or external library required.
        if unsafe { BCryptGenRandom(std::ptr::null_mut(), bytes.as_mut_ptr(), 16, 2) } != 0 {
            return Err(io::Error::other("OS random generator unavailable"));
        }
    }
    #[cfg(not(windows))]
    {
        use std::io::Read;
        std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    }
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
