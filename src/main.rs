use std::process::Command;

use niri_ipc::{Event, Request, Response, socket::Socket};

fn annotate(path: String) {
    let mut annotator = Command::new("satty");
    match annotator.arg("--filename").arg(path).output() {
        Ok(out) => {
            println!("{out:?}")
        }
        Err(err) => {
            eprintln!("{err:?}")
        }
    }
}
fn main() -> std::io::Result<()> {
    let mut socket = Socket::connect()?;

    let reply = socket.send(Request::EventStream)?;
    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = socket.read_events();
        while let Ok(event) = read_event() {
            if let Event::ScreenshotCaptured { path: Some(path) } = event {
                annotate(path);
            }
        }
    }

    Ok(())
}
