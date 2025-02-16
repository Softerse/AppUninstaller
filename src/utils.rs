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
