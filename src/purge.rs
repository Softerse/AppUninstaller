#![allow(deprecated)]

use crate::dialog::Dialog;
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
///
use crate::utils::isolate_exec;
use gtk::prelude::BoxExt;
use gtk::prelude::DialogExt;
use gtk::prelude::GtkWindowExt;
use gtk::Dialog as GtkDialog;
use gtk::Label;
use gtk::ResponseType;
use log::error;
use log::info;
use std::path::PathBuf;

const COMMON_DATA_DIRS: [&str; 6] = [
    "/usr/local/share/",
    "/usr/local/",
    "/usr/share/",
    "/var/lib/",
    "/opt/",
    "/etc/",
];

const LOCAL_DATA_DIRS: [&str; 4] = [
    "/.local/share/",
    "/.", // Some apps save data under $HOME/.<app name>
    "/.var",
    "/.config",
];

/* Does all the purging for us */
pub struct AppPurger;
pub struct AppPurgeProcess {
    app: String,
    headless: bool,
}

impl AppPurger {
    #[inline]
    fn find_exec(exec: String) -> Option<PathBuf> {
        match which::which(isolate_exec(exec.clone())) {
            Ok(path) => Some(path),
            Err(e) => {
                error!("Failed to locate \"{}\": {}", exec, e.to_string());
                None
            }
        }
    }

    pub fn purge_app(appname: String, exec: PathBuf, entry: PathBuf) {
        let exec_path = exec.to_string_lossy();

        if let Some(exec_file) = Self::find_exec(exec_path.to_string()) {
            if let Err(e) = std::fs::remove_file(&exec_file) {
                error!(
                    "Failed to remove {}: {}",
                    exec_file.display(),
                    e.to_string()
                );
            }
        }

        if let Err(e) = std::fs::remove_file(&entry) {
            error!("Failed to remove {}: {}", entry.display(), e.to_string());
        } else {
            info!("Deleted desktop entry {}", entry.display());
        }

        AppPurgeProcess::new(appname, false).try_purge();
    }
}

impl AppPurgeProcess {
    pub fn new(app: String, headless: bool) -> Self {
        Self { app, headless }
    }

    fn found_file_dialog(&self, path: PathBuf) {
        if self.headless { return }
        log::info!("Found possible path at {}", path.display());
        let dialog = GtkDialog::builder()
            .title("Delete data")
            .icon_name("question-symbolic")
            .modal(true)
            .margin_start(4)
            .margin_end(4)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let content = dialog.content_area();
        content.append(&Label::new(Some(&format!(
            "Data for this application has been detected at {}. Delete it?",
            path.display()
        ))));
        dialog.add_button("Yes, delete", ResponseType::Accept);
        dialog.add_button("No, leave it", ResponseType::Cancel);
        dialog.set_default_response(ResponseType::Cancel);

        dialog.connect_response(move |d, response| match response {
            ResponseType::Accept => {
                let result = std::fs::remove_dir_all(path.clone());
                if result.is_ok() {
                    info!("Deleted directory {}", path.display());
                } else {
                    let e = result.unwrap_err();
                    Dialog::new_without_parent(
                        "Error",
                        &format!("Couldn't remove directory {}: {}", path.display(), e),
                    );
                }
                d.close();
            }
            _ => d.close(),
        });
    }

    pub fn find_app_files_global(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for dir in COMMON_DATA_DIRS {
            let path = PathBuf::from(dir).join(self.app.to_lowercase());
            if path.exists() {
                paths.push(path.clone());
                self.found_file_dialog(path);
            }
        }
        paths
    }

    pub fn find_app_files_home(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        /* We can allow this function even though its deprecated because this app is not designed for Windows. */
        #[allow(deprecated)]
        let homedir = std::env::home_dir().unwrap_or_else(||{
            log::error!("Couldn't find the home directory for the user. Crashing because no substitute can be used");
            std::process::abort()
        });

        for dir in LOCAL_DATA_DIRS {
            let path = homedir.join(dir).join(self.app.clone());
            if path.exists() {
                paths.push(path.clone());
                self.found_file_dialog(path);
            }
        }
        paths
    }

    #[inline]
    pub fn try_purge(self) {
        log::info!("Trying global common paths");
        self.find_app_files_global();
        log::info!("Trying local common paths");
        self.find_app_files_home();
    }
}
