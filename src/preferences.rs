use gtk::{prelude::*, Button, CheckButton, Label, Orientation, Window};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Preferences {
    pub startupdlg: bool,
    pub fullscreen: bool,
}

impl Preferences {
    pub fn load() -> Self {
        let config = confy::load::<Self>("LinuxAppUninstaller", None).unwrap();
        config
    }

    pub fn save(&self) {
        confy::store("LinuxAppUninstaller", None, self).unwrap_or_else(|e| {
            log::error!("Failed to save new settings on the disk: {}", e.to_string());
        })
    }

    pub fn window(&self) -> Window {
        let window = Window::builder()
            .title("Preferences")
            .default_width(240)
            .default_height(160)
            .destroy_with_parent(true)
            .icon_name("settings-configure")
            .build();

        let cnt = gtk::Box::new(Orientation::Vertical, 4);
        let startupdlg = CheckButton::with_label("Show warning on startup");
        let fullscreen = CheckButton::with_label("Start application in fullscreen");
        let savebtn = Button::with_label("Save changes");

        let prefs = Rc::new(RefCell::new(self.clone()));

        startupdlg.set_active(self.startupdlg);
        {
            let prefs = Rc::clone(&prefs);
            startupdlg.connect_toggled(move |s| {
                prefs.borrow_mut().startupdlg = s.is_active();
            });
        }

        fullscreen.set_active(self.fullscreen);
        {
            let prefs = Rc::clone(&prefs);
            fullscreen.connect_toggled(move |s| {
                prefs.borrow_mut().fullscreen = s.is_active();
            });
        }

        {
            let prefs = Rc::clone(&prefs);
            savebtn.connect_clicked(move |_| {
                prefs.borrow().save();
                log::info!("Saved new settings!");
            });
        }

        cnt.set_margin_start(6);
        cnt.set_margin_end(6);
        cnt.set_margin_top(6);
        cnt.set_margin_bottom(6);

        cnt.append(&startupdlg);
        cnt.append(&fullscreen);
        cnt.append(&Label::new(Some(
            "You must restart the application to see the changes.",
        )));
        cnt.append(&savebtn);
        window.set_child(Some(&cnt));

        window
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            startupdlg: true,
            fullscreen: false,
        }
    }
}
