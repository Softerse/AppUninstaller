#![warn(clippy::all)]
// This file is part of Linux Program Uninstaller.
///
/// Linux Uninstaller - A fast, elegant program uninstaller for Linux
///  
/// Copyright (C) 2025 Aggelos Tselios  
///  
/// This program is free software: you can redistribute it and/or modify  
/// it under the terms of the GNU General Public License as published by  
/// the Free Software Foundation, either version 3 of the License, or  
/// (at your option) any later version.  
///  
/// This program is distributed in the hope that it will be useful,  
/// but WITHOUT ANY WARRANTY; without even the implied warranty of  
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the  
/// GNU General Public License for more details.  
///  
/// You should have received a copy of the GNU General Public License  
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.
mod desktop;
mod dialog;
mod error;
mod preferences;
mod purge;
mod utils;

use dialog::Dialog;
use gtk::gdk::{prelude::*, Display};
use gtk::gio::SimpleAction;
#[allow(deprecated)]
use gtk::{glib, AboutDialog, CssProvider, License};
use gtk::{prelude::*, ScrolledWindow};
use gtk::{Application, Builder};
#[allow(unused_imports)]
use log::{error, info, warn};
use preferences::Preferences;
use utils::{isolate_exec, omit_dir_from_cmd};

const STARTUP_MSG: &str = r#"This application is meant to be used for very specific cases, like apps built and installed manually.
It is NOT a replacement for `apt`, `pacman` or any other package manager. In fact, it can cause problems if you use this app to uninstall
apps installed through a package manager.

You are assumed to be responsible enough to understand that DATA WILL BE LOST FOREVER and you CANNOT reverse deletion. Please make sure
you do not just blindly run "Delete app" on everything to free space.

Final warning: Make sure you know what you are doing. The creator is not liable or responsible in any way if something doesn't go the way you
expected, you accidentally uninstall something you shouldn't or dragons jump out of your computer and try to bite you. Again, you should probably not run this
application if your package manager can uninstall the app you want to uninstall."#;

fn main() -> glib::ExitCode {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    let app = Application::builder()
        .application_id("org.sfd.LinuxAppUninstaller")
        .build();

    app.connect_activate(move |app| {
        let pref = Preferences::load();
        let builder = Builder::from_string(include_str!("../ui/window.xml"));
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("../ui/style.css"));
        #[allow(deprecated)]
        gtk::StyleContext::add_provider_for_display(
            &Display::default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let mut apps = desktop::load_entries();
        let window: gtk::ApplicationWindow = builder.object("mainwindow").unwrap_or_else(|| {
            error!("Could not retrieve window object from UI file");
            std::process::exit(-1);
        });
        unsafe {
            if libc::getuid() == 0 {
                warn!("This program should not be run as root");
                let warning = builder.object::<gtk::Box>("warn-root").unwrap();

                warning.set_css_classes(&["warn-root"]);
                warning.set_visible(true);
            }
        }
        window.set_maximized(pref.fullscreen);

        let prefaction = SimpleAction::new("preferences", None);
        let quitaction = SimpleAction::new("quit", None);
        let windowclone = window.clone();
        let aboutaction = SimpleAction::new("about", None);

        quitaction.connect_activate(|_, _| std::process::exit(0));
        aboutaction.connect_activate(move |_, _| {
            AboutDialog::builder()
                .authors(["Aggelos Tselios "])
                .modal(true)
                .transient_for(&windowclone)
                .copyright("2025 The Linux App Uninstaller developers")
                .license_type(License::Gpl30Only)
                .version("0.1.0")
                .build()
                .present();
        });
        app.add_action(&prefaction);
        app.add_action(&quitaction);
        app.add_action(&aboutaction);

        if pref.startupdlg {
            Dialog::new("Warning", STARTUP_MSG, &window).show();
        }

        prefaction.connect_activate(move |_, _| {
            pref.window().present();
        });

        let applist: gtk::Box = builder.object("applist").unwrap_or_else(|| {
            warn!("Failed to retrieve a UI element from the descriptor file");
            std::process::exit(-1);
        });

        let appview = builder
            .object::<ScrolledWindow>("appview")
            .unwrap_or_else(|| {
                warn!("Failed to retrieve a UI element from the descriptor file");
                std::process::exit(-1);
            });

        apps.sort_by(|a, b| a.name.cmp(&b.name));
        for a in apps {
            let blacklisted_execs = [
                "flatpak",
                "xdg-open",
                "systemsettings",
                "cinnamon-settings",
                "gamemoderun",
                "gapplication",
                "java",
            ];
            if blacklisted_execs.iter().any(|x| {
                omit_dir_from_cmd((*x.to_owned()).to_string()) == isolate_exec(a.exec.clone())
            }) {
                log::warn!("Skipping application \"{}\"", a.name);
                continue;
            }

            let button = a.create_button_from_entry();
            let appview = appview.clone();
            button.connect_clicked(move |_| {
                appview.set_visible(true);
                appview.set_child(Some(&a.appview()));
            });

            applist.append(&button);
        }

        window.set_application(Some(app));
        window.present();
    });

    app.run()
}
