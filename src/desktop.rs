#![allow(deprecated)]

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
use crate::dialog::Dialog;
use crate::purge::{AppPurgeProcess, AppPurger};
use crate::utils;
use freedesktop_desktop_entry::DesktopEntry as FdoDesktopEntry;
use gtk::{prelude::*, Align, Dialog as GtkDialog, ResponseType};
use gtk::{Button, Image, Label};
use log::error;
use rayon::prelude::*;
use std::borrow::Cow;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub full_path: String,
}

impl DesktopEntry {
    #[inline]
    pub fn new(
        name: String,
        exec: String,
        icon_path: Option<String>,
        description: String,
        full_path: String,
    ) -> Self {
        Self {
            name,
            exec,
            description,
            icon_path,
            full_path,
        }
    }

    pub fn create_button_from_entry(&self) -> Button {
        let button = Button::new();
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let icon = Image::from_icon_name(
            &self
                .icon_path
                .clone()
                .unwrap_or("question-symbolic".to_string()),
        );
        let label = Label::new(Some(&self.name));

        label.set_halign(gtk::Align::Center);
        container.append(&icon);
        container.append(&label);
        button.set_child(Some(&container));

        button
    }

    pub fn appview(&self) -> gtk::Box {
        let view = gtk::Box::new(gtk::Orientation::Vertical, 16);
        let title = Label::new(None);
        let exec_widget = Label::new(Some(&format!("Command: {}", self.exec)));
        let desc = Label::new(Some(&format!("Description: {}", self.description.clone())));
        let full = Label::new(Some(&format!("Desktop entry path: {}", self.full_path)));
        let filesize = Label::new(Some(&format!(
            "Size on disk: {}KB",
            utils::get_file_size(
                which::which(self.exec.clone())
                    .unwrap_or_default()
                    .to_string_lossy()
            )
            .unwrap_or("0".to_string())
        )));
        let openbtn = Button::with_label("Open externally");
        let dltapp = Button::builder()
            .label("Delete Application (!)")
            .css_classes(vec!["destructive-action"])
            .build();
        #[cfg(debug_assertions)]
        let opendata = Button::builder()
            .label("Open Data Folder(s)")
            .tooltip_text(r#"Only available on debug builds. Opens the directory that AppUninstaller thinks belongs to this application. Useful to see how accurate is the internal algorithm for detection.
            Note that if many directories are found only the last one detected will be used."#)
            .build();

        /* Apparently we can't just use markup directly, we need to set it manually here. */
        title.set_markup(&format!(
            "<b><span size='xx-large'>App: {}</span></b>",
            self.name
        ));
        title.set_halign(Align::Start);
        exec_widget.set_halign(Align::Start);
        desc.set_halign(Align::Start);

        let name = self.name.clone();
        let exec = PathBuf::from(self.exec.clone());
        let entry = PathBuf::from(self.full_path.clone());
        let entry_c = entry.clone();

        openbtn.connect_clicked(move |_| {
            open::that_detached(format!("file://{}", entry_c.to_string_lossy())).unwrap_or_else(
                |e| {
                    Dialog::new_without_parent(
                        "Error!",
                        &format!("Could not open file '{}': {}.", entry_c.display(), e),
                    );
                    log::error!("Failed to open {}: {}", entry_c.display(), e)
                },
            )
        });
        openbtn.set_tooltip_text(Some("Opens the desktop entry using your system's preconfigured application. Useful if you want to modify something in it."));
        openbtn.set_sensitive(OpenOptions::new().read(true).open(&self.full_path).is_ok());

        dltapp.connect_clicked(move |_| {
            let choice = GtkDialog::builder()
                .icon_name("question")
                .title("Confirm action")
                .modal(true)
                .build();

            let content = choice.content_area();
            content.set_halign(Align::Center);
            content.set_valign(Align::Center);
            content.append(&Label::new(Some(
                "Are you sure you wish to delete this application?",
            )));
            choice.add_button("No, close", ResponseType::Close);
            choice
                .add_button(&format!("Yes, delete {}", name), ResponseType::Accept)
                .set_css_classes(&["destructive-action"]);
            choice.set_default_response(ResponseType::Close);

            let exec = exec.clone();
            let entry = entry.clone();
            let name = name.clone();
            choice.connect_response(move |choice, response| {
                choice.close();
                if response == ResponseType::Accept {
                    AppPurger::purge_app(name.clone(), exec.clone(), entry.clone())
                }
            });

            choice.show();
        });

        let name = self.name.clone();
        #[cfg(debug_assertions)]
        opendata.connect_clicked(move |b| {
            let dir_g = AppPurgeProcess::new(name.clone(), true).find_app_files_global();
            let dir_l = AppPurgeProcess::new(name.clone(), true).find_app_files_home();
            if !dir_g.is_empty() {
                let dir = dir_g.last().unwrap();
                open::that_detached(dir).unwrap_or_else(|e| {
                    log::error!(
                        "Couldn't open directory {}: {}",
                        dir.display(),
                        e.to_string()
                    );
                });
            }

            if !dir_l.is_empty() {
                let dir = dir_l.last().unwrap();
                open::that_detached(dir).unwrap_or_else(|e| {
                    log::error!(
                        "Couldn't open directory {}: {}",
                        dir.display(),
                        e.to_string()
                    );
                });
            }

            if dir_g.is_empty() && dir_l.is_empty() {
                b.set_label("No directories were found in the system.");
            }
        });

        view.set_margin_start(16);
        view.set_margin_end(16);
        view.set_margin_top(16);
        view.set_margin_bottom(16);
        view.set_halign(Align::Start);
        view.append(&title);
        view.append(&exec_widget);
        view.append(&desc);
        view.append(&full);
        view.append(&filesize);

        /* The brackets aren't needed here, it's just for readability. */
        {
            let c = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            c.append(&openbtn);
            #[cfg(debug_assertions)]
            c.append(&opendata);
            c.append(&dltapp);
            view.append(&c);
        }

        view
    }
}

pub fn load_entries() -> Vec<DesktopEntry> {
    let entry_dirs = [
        format!(
            "{}/.local/share/applications/",
            std::env::var("HOME").unwrap()
        ),
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
    ];

    let entries = Mutex::new(Vec::new()); // Protects access to entries

    // Process directories in parallel
    entry_dirs.par_iter().for_each(|dir| {
        if let Ok(entries_iter) = fs::read_dir(dir) {
            entries_iter
                .flatten()
                .filter(|entry| {
                    // Filter for only .desktop files
                    entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                        && entry.file_name().to_string_lossy().ends_with(".desktop")
                })
                .for_each(|entry| {
                    let path = entry.path();
                    if let Ok(fdo_entry) = FdoDesktopEntry::from_path(&path, None::<&[String]>) {
                        let exec = fdo_entry.exec().unwrap_or_default();
                        let name = fdo_entry.name(&["en_US"]).unwrap_or_default();
                        let icon_path = fdo_entry.icon().unwrap_or_default();
                        let description = fdo_entry
                            .comment(&["en_US"])
                            .unwrap_or(Cow::Borrowed("None"));

                        // Lock and collect entries safely
                        let mut entries = entries.lock().unwrap();
                        entries.push(DesktopEntry::new(
                            name.to_string(),
                            exec.to_string(),
                            Some(icon_path.to_string()),
                            description.to_string(),
                            path.to_string_lossy().to_string(),
                        ));
                    }
                });
        } else {
            error!("Error reading directory: {}", dir);
        }
    });

    // Return the collected entries after all parallel work is done
    entries.into_inner().unwrap()
}
