use std::process::Command;

use niri_ipc::{Event, Request, Response, socket::Socket};
use notify_rust::{CloseReason, Notification};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Config {
    annotator: Annotator,
    uploader: Uploader,
}
#[derive(Deserialize)]
struct Annotator {
    command: String,
    enabled: bool,
    auto: bool,
}
#[derive(Deserialize)]
struct Uploader {
    url: String,
    enabled: bool,
    auto: bool,
}
fn annotate(path: &String, config: &Annotator) {
    let mut annotator = Command::new("sh");
    let command = config.command.replace("%path%", path);
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
                if config.annotator.enabled {
                    if config.annotator.auto {
                        annotate(&path, &config.annotator)
                    } else {
                        // TODO: this blocks which sucks, but me and lily couldn't figure out how threads work
                        // is also probably dependent on mako specific weirdness??? but whatever
                        Notification::new()
                            .summary("Click to annotate")
                            .show()
                            .map(|handler| {
                                handler.on_close(|reason| {
                                    if let CloseReason::Dismissed = reason {
                                        annotate(&path, &config.annotator);
                                    }
                                })
                            })
                            .expect("failed to spawn notification");
                    }
                };
            }
        }
    }

    Ok(())
}
