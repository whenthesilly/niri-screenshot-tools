use niri_ipc::{Event, Request, Response, socket::Socket};
use notify_rust::{CloseReason, Notification};
use serde::Deserialize;
use std::{path::Path, process::Command};

#[derive(Deserialize)]
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
fn annotate(path: &str, config: &Annotator) {
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

fn upload(client: &reqwest::blocking::Client, url: &str, image: &[u8]) {
    let res = client.put(url).body(image.to_owned()).send();
    println!("{res:?}");
} // TODO: give this post functionality also to work with other destinations

fn main() {
    let config = std::fs::read_to_string(
        pathexpand::expand("${XDG_CONFIG_HOME:~/.config}/niri-screenshot-tools/config.toml")
            .unwrap(),
    )
    .expect("Please write a config file (temporary)"); // TODO: write a default config and save it to a file instead
    let config = toml::from_str::<Config>(&config).expect("unable to parse config");
    let mut socket = Socket::connect().expect("Failed to connect to niri socket");
    let client = reqwest::blocking::Client::new();

    let reply = socket.send(Request::EventStream).unwrap();
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
                                    if matches!(reason, CloseReason::Dismissed) {
                                        annotate(&path, &config.annotator);
                                    }
                                })
                            })
                            .expect("failed to spawn notification");
                    }
                };
                if config.uploader.enabled {
                    let image = std::fs::read(&path).expect("Unable to read screenshot path");
                    let filename = Path::new(&path).file_name().unwrap().to_str().unwrap(); //this is disgusting but its 1am
                    let url = config.uploader.url.replace("%name%", filename);
                    if config.uploader.auto {
                        upload(&client, &url, &image)
                    } else {
                        // TODO: this also blocks
                        Notification::new()
                            .summary("Click to upload")
                            .show()
                            .map(|handler| {
                                handler.on_close(|reason| {
                                    if matches!(reason, CloseReason::Dismissed) {
                                        upload(&client, &url, &image);
                                    }
                                })
                            })
                            .expect("failed to spawn notification");
                    }
                }
            }
        }
    }
}
