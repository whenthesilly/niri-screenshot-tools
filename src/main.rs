use std::process::Command;

use niri_ipc::{Event, Request, Response, socket::Socket};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Config {
    annotator: Annotator,
    uploader: Uploader,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct Annotator {
    command: String,
    enabled: bool,
    auto: bool,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct Uploader {
    url: String,
    enabled: bool,
}
fn annotate(path: String, config: &Annotator) {
    let mut annotator = Command::new("sh");
    let command = config.command.replace("%path%", &path);
    match annotator.arg("-c").arg(command).output() {
        Ok(out) => {
            println!("{out:?}")
        }
        Err(err) => {
            eprintln!("{err:?}")
        }
    }
}

fn main() -> std::io::Result<()> {
    let config = std::fs::read_to_string(
        pathexpand::expand("${XDG_CONFIG_HOME:~/.config}/niri-screenshot-tools/config.toml")
            .unwrap(),
    )
    .expect("Please write a config file (temporary)");
    let config = toml::from_str::<Config>(&config).expect("unable to parse config");
    let mut socket = Socket::connect()?;

    let reply = socket.send(Request::EventStream)?;
    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = socket.read_events();
        while let Ok(event) = read_event() {
            if let Event::ScreenshotCaptured { path: Some(path) } = event {
                annotate(path, &config.annotator);
            }
        }
    }

    Ok(())
}
