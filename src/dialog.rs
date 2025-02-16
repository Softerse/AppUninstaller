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
use gtk::{prelude::*, ApplicationWindow, Dialog as GtkDialog, Label, ResponseType};

pub struct Dialog {
    obj: GtkDialog,
}

impl Dialog {
    pub fn new(title: &str, msg: &str, parent: &ApplicationWindow) -> Self {
        let obj = GtkDialog::builder()
            .title(title)
            .modal(true)
            .transient_for(parent)
            .focus_visible(true)
            .build();

        let content = obj.content_area();
        obj.add_button("Close", ResponseType::Ok);
        obj.set_title(Some(title));
        content.append(&Label::new(Some(msg)));

        obj.connect_response(|dialog, _| {
            dialog.close();
        });

        Self { obj }
    }

    pub fn new_without_parent(title: &str, msg: &str) -> Self {
        let obj = GtkDialog::builder()
            .title(title)
            .modal(true)
            .focus_visible(true)
            .build();

        let content = obj.content_area();
        obj.add_button("Close", ResponseType::Ok);
        obj.set_title(Some(title));
        content.append(&Label::new(Some(msg)));

        obj.connect_response(|dialog, _| {
            dialog.close();
        });

        Self { obj }
    }

    pub fn show(&self) {
        self.obj.show();
    }
}
