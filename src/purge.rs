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
use crate::error::Error;
use log::error;
use log::info;
use std::path::PathBuf;

/* Does all the purging for us */
pub struct AppPurger;

impl AppPurger {
    /* Isolates the executable from the rest of the command using whitespace detection. */
    #[inline]
    fn isolate_exec(cmd: String) -> String {
        cmd.split_whitespace().next().unwrap_or("").to_owned()
    }

    #[inline]
    fn find_exec(exec: String) -> Option<PathBuf> {
        match which::which(Self::isolate_exec(exec.clone())) {
            Ok(path) => Some(path),
            Err(e) => {
                error!("Failed to locate \"{}\": {}", exec, e.to_string());
                None
            }
        }
    }

    pub fn purge_app(exec: PathBuf, entry: PathBuf) -> Result<(), Error> {
        let exec = exec.to_string_lossy().to_string();
        match Self::find_exec(exec.clone()) {
            None => Err(Error::ExecNotFound), // show an error dialog here
            Some(ex) => match std::fs::remove_file(&ex) {
                Err(e) => {
                    error!("Failed to remove {}: {}", ex.display(), e.to_string());
                    Err(Error::CouldNotDelete(String::from(ex.to_string_lossy())))
                }
                Ok(()) => {
                    info!("Deleted executable {}", ex.display());
                    match std::fs::remove_file(&entry) {
                        Err(e) => {
                            error!("Failed to remove {}: {}", entry.display(), e.to_string());
                            Err(Error::CouldNotDelete(entry.to_string_lossy().to_string()))
                        }
                        Ok(()) => {
                            info!("Deleted desktop entry {}", entry.display());
                            Ok(())
                        }
                    }
                }
            },
        }
    }
}
