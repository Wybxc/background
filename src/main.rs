#![windows_subsystem = "windows"]

use std::os::windows::process::CommandExt;
use std::process::Child;
use std::sync::OnceLock;

use argh::FromArgs;
use serde::{Deserialize, Serialize};
use tray_icon::menu::{Menu, MenuEvent, MenuEventReceiver, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};

/// Run a command in background.
#[derive(FromArgs)]
struct Args {
    /// configuration json file.
    #[argh(positional)]
    config: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    executable: String,
    args: Vec<String>,
    icon: Option<String>,
}

fn create_tray(config: &Config) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let mut tray = TrayIconBuilder::new()
        .with_tooltip(&config.executable)
        .with_menu(Box::new({
            let menu = Menu::new();
            menu.append_items(&[
                &MenuItem::with_id("Restart", "Restart", true, None),
                &MenuItem::with_id("Quit", "Quit", true, None),
            ])?;
            menu
        }));
    if let Some(icon) = &config.icon {
        tray = tray.with_icon(Icon::from_path(icon, None)?);
    }
    let tray = tray.build()?;
    Ok(tray)
}

fn spawn_child(config: &Config) -> Result<Child, Box<dyn std::error::Error>> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let child = std::process::Command::new(&config.executable)
        .args(&config.args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;

    Ok(child)
}

fn handle_event(
    config: &Config,
    menu_channel: &MenuEventReceiver,
    process: &mut Option<Child>,
    elwt: &EventLoopWindowTarget<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    if process.is_none() && !elwt.exiting() {
        *process = Some(spawn_child(config)?);
    }

    if let Ok(event) = menu_channel.try_recv() {
        match event.id().0.as_str() {
            "Restart" => {
                if let Some(mut process) = process.replace(spawn_child(config)?) {
                    process.kill()?;
                }
            }
            "Quit" => {
                if let Some(mut process) = std::mem::take(process) {
                    process.kill()?;
                }
                elwt.exit();
            }
            _ => {}
        }
    }
    Ok(())
}

fn start(args: Args) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string(args.config)?;
    let config: Config = serde_json::from_str(&config)?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let tray = create_tray(&config)?;

    let mut process = None;
    let menu_channel = MenuEvent::receiver();
    let error = OnceLock::new();
    event_loop.run(|_, elwt| {
        if let Err(e) = handle_event(&config, menu_channel, &mut process, elwt) {
            error.set(e).unwrap();
            elwt.exit();
        }
    })?;

    if let Some(e) = error.into_inner() {
        return Err(e);
    }

    Ok(tray)
}

fn main() {
    let args = argh::from_env::<Args>();

    if let Err(e) = start(args) {
        rfd::MessageDialog::new()
            .set_title("Error")
            .set_description(e.to_string())
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
}
