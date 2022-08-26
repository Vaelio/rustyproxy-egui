use std::process::{Command, Child};
use std::ffi::OsStr;

pub struct ProxyHandler {
    handle: Child,
}

impl ProxyHandler {
    pub fn start<T, S>(command: &str, args: T) -> Self
    where 
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let child = Command::new(command)
            .args(args)
            .spawn()
            .unwrap();


        ProxyHandler {
            handle: child,
        }
    }

    pub fn kill(&mut self) -> bool {
        self.handle.kill().is_ok()
    }
}