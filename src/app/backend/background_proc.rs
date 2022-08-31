use std::ffi::OsStr;
use std::process::{Child, Command};

pub struct ProxyHandler {
    handle: Child,
}

impl ProxyHandler {
    pub fn start<T, S>(command: &str, args: T) -> Self
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let child = Command::new(command).args(args).spawn().unwrap();

        ProxyHandler { handle: child }
    }

    pub fn is_alive(&mut self) -> bool {
        match self.handle.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => true,
        }
    }

    pub fn kill(&mut self) -> bool {
        let r = self.handle.kill().is_ok();
        let w = self.handle.wait().is_ok();

        r && w
    }
}
