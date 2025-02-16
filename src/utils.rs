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

use std::{fs, io};

/* Isolates the executable from the rest of the command using whitespace detection. */
#[inline]
pub fn isolate_exec(cmd: String) -> String {
    cmd.split_whitespace().next().unwrap_or("").to_owned()
}

/*
 * Takes a full path to a command, eg. /usr/bin/ls and returns the name of the command,
 * without the directory.
 */
#[inline]
pub fn omit_dir_from_cmd(cmd: String) -> String {
    isolate_exec(cmd.rsplit('/').next().unwrap_or(&cmd).to_owned())
}

/*
 * Returns the size of the file specified in KBs
 */
#[inline]
pub fn get_file_size(path: impl Into<String>) -> io::Result<String> {
    let metadata = fs::metadata(path.into())?;
    let size_in_bytes = metadata.len();
    Ok((size_in_bytes as f64 / (1024.0)).round().to_string())
}
