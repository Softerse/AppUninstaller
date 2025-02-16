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
mod purge;

use dialog::Dialog;
use gtk::glib;
use gtk::{prelude::*, ListBox, ScrolledWindow};
use gtk::{Application, Builder};
#[allow(unused_imports)]
use log::{error, info, warn};

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

    app.connect_activate(|app| {
        let builder = Builder::from_string(include_str!("../ui/window.xml"));
        let apps = desktop::load_entries();
        let window: gtk::ApplicationWindow = builder.object("mainwindow").unwrap_or_else(|| {
            error!("Could not retrieve window object from UI file");
            std::process::exit(-1);
        });
        Dialog::new("Warning", STARTUP_MSG, &window).show();

        let applist: ListBox = builder.object("applist").unwrap_or_else(|| {
            warn!("Failed to retrieve a UI element from the descriptor file");
            std::process::exit(-1);
        });

        let appview = builder
            .object::<ScrolledWindow>("appview")
            .unwrap_or_else(|| {
                warn!("Failed to retrieve a UI element from the descriptor file");
                std::process::exit(-1);
            });

        for a in apps {
            
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
