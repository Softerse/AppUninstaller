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
use crate::purge::AppPurger;
use freedesktop_desktop_entry::DesktopEntry as FdoDesktopEntry;
use gtk::{prelude::*, Align, Dialog as GtkDialog, ResponseType};
use gtk::{Button, Image, Label};
use log::error;
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

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
        let dltapp = Button::builder()
            .label("Delete Application (!)")
            .css_classes(vec!["destructive-action"])
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

        dltapp.connect_clicked(move |_|{
            let choice = GtkDialog::builder()
                .icon_name("question")
                .title("Confirm action")
                .modal(true)
                .build();

            let content = choice.content_area();
            content.set_halign(Align::Center);
            content.set_valign(Align::Center);
            content.append(&Label::new(Some("Are you sure you wish to delete this application?")));
            choice.add_button("No, close", ResponseType::Close);
            choice.add_button(&format!("Yes, delete {}", name), ResponseType::Accept).set_css_classes(&[ "destructive-action" ]);
            choice.set_default_response(ResponseType::Close);

            let exec = exec.clone();
            let entry = entry.clone();
            choice.connect_response(move |choice, response|{
                choice.close();
                match response {
                    ResponseType::Accept => {
                        let delete_result = AppPurger::purge_app(exec.clone(), entry.clone());
                        if delete_result.is_ok() {
                            Dialog::new_without_parent("Success", "Application deleted successfully. Any residual files (eg. in /usr/share/) must be deleted manually.").show();
                        } else {
                            let e = delete_result.unwrap_err();
                            Dialog::new_without_parent("Error!", &format!("Could not delete this application.\nThe reported error was: '{}'", e.to_string())).show();
                        }
                    }
                    _ => ()
                }
            });

            choice.show();
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
        view.append(&dltapp);

        view
    }
}

pub fn load_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();
    let entry_dirs = [
        format!(
            "{}/.local/share/applications/",
            std::env::var("HOME").unwrap()
        ),
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
    ];

    for dir in entry_dirs.iter() {
        if let Ok(entries_iter) = fs::read_dir(dir) {
            for entry in entries_iter.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    || !entry.file_name().to_string_lossy().ends_with(".desktop")
                {
                    continue;
                }

                let path = entry.path();
                if let Ok(fdo_entry) = FdoDesktopEntry::from_path(&path, None::<&[String]>) {
                    let exec = fdo_entry.exec().unwrap_or_default();
                    let name = fdo_entry.name(&["en_US"]).unwrap_or_default();
                    let icon_path = fdo_entry.icon().unwrap_or_default();
                    let description = fdo_entry
                        .comment(&["en_US"])
                        .unwrap_or(Cow::Borrowed("None"));

                    entries.push(DesktopEntry::new(
                        name.to_string(),
                        exec.to_string(),
                        Some(icon_path.to_string()),
                        description.to_string(),
                        path.to_string_lossy().to_string(),
                    ));
                }
            }
        } else {
            error!("Error reading directory: {}", dir);
        }
    }
    entries
}
