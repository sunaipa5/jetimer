mod timer;

use std::sync::mpsc::{Receiver, channel};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

struct App {
    tray_icon: Option<TrayIcon>,
    rx: Receiver<String>,
    menu_channel: &'static tray_icon::menu::MenuEventReceiver,
    quit_id: tray_icon::menu::MenuId,
}

const ICON_SVG: &str = include_str!("../assets/time-symbolic.svg");

fn load_svg_as_icon() -> Icon {
    let rtree = resvg::usvg::Tree::from_str(ICON_SVG, &resvg::usvg::Options::default())
        .expect("SVG parse error");

    let pixmap_size = rtree.size().to_int_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    resvg::render(
        &rtree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let width = pixmap.width();
    let height = pixmap.height();
    let rgba = pixmap.data().to_vec();

    Icon::from_rgba(rgba, width, height).expect("Icon oluşturma hatası")
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        while gtk::events_pending() {
            gtk::main_iteration();
        }

        while let Ok(msg) = self.rx.try_recv() {
            if msg == "QUIT_NOW" {
                event_loop.exit();
                return;
            } else if let Some(ref mut icon) = self.tray_icon {
                let _ = icon.set_title(Some(msg));
            }
        }

        while let Ok(event) = self.menu_channel.try_recv() {
            if event.id == self.quit_id {
                event_loop.exit();
            }
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(150),
        ));
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    gtk::init().expect("GTK init failed");

    let Some(title) =
        timer::get_zenity_output(&["--entry", "--title=Jetimer", "--text=Type a title"])
    else {
        return;
    };
    let Some(selected) = timer::get_zenity_output(&[
        "--list",
        "--title=Jetimer",
        "--column=Time",
        "--width=400",
        "--height=450",
        "Custom",
        "30s",
        "1m",
        "3m",
        "5m",
        "10m",
        "15m",
        "20m",
        "25m",
        "30m",
        "35m",
        "40m",
        "45m",
        "50m",
        "55m",
        "60m",
    ]) else {
        return;
    };

    let total = match selected.as_str() {
        "Custom" => {
            let Some(custom) =
                timer::get_zenity_output(&["--entry", "--text=Enter time (MM:SS or Seconds)"])
            else {
                return;
            };

            let parts: Vec<&str> = custom.split(':').collect();
            let seconds = if parts.len() == 2 {
                let m = parts[0].parse::<i32>().unwrap_or(-1);
                let s = parts[1].parse::<i32>().unwrap_or(-1);
                if m >= 0 && s >= 0 { m * 60 + s } else { -1 }
            } else {
                custom.parse::<i32>().unwrap_or(-1)
            };

            if seconds <= 0 {
                let _ = std::process::Command::new("zenity")
                    .args([
                        "--error",
                        "--title=Jetimer",
                        "--text=Invalid format! Use MM:SS or total seconds.",
                    ])
                    .status();
                return;
            }
            seconds
        }
        s if s.ends_with('s') => s[..s.len() - 1].parse().unwrap_or(0),
        m if m.ends_with('m') => m[..m.len() - 1].parse::<i32>().unwrap_or(0) * 60,
        _ => 0,
    };

    if total <= 0 {
        return;
    }

    let event_loop = EventLoop::new().unwrap();
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append(&quit_item);
    let quit_id = quit_item.id();

    let tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Jetimer")
            .with_icon(load_svg_as_icon())
            .with_title("Starting...")
            .build()
            .unwrap(),
    );

    let (tx, rx) = channel();
    timer::run_countdown(total, title, tx);

    let mut app = App {
        tray_icon,
        rx,
        menu_channel: tray_icon::menu::MenuEvent::receiver(),
        quit_id: quit_id.clone(),
    };

    event_loop.run_app(&mut app).unwrap();
}
