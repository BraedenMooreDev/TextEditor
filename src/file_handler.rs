use std::{fs::File, io::{Read, Write}, path::PathBuf};
use native_dialog::FileDialog;

pub fn file_new(content_buf: &mut String) {

    content_buf.clear();
}

pub fn file_open(file_buf: &mut Option<File>, content_buf: &mut String) {

    let path_opt: Option<PathBuf>;

    // Result error handling to make sure a valid path was selected to open.
    match FileDialog::new()
        .add_filter("Text Document", &["txt"])
        .show_open_single_file() 
    {
        Ok(p) => { path_opt = p; },
        Err(e) => { panic!("{}", e); }
    }

    let path: PathBuf;

    match path_opt {
        Some(p) => path = p,
        None => return
    }

    // Result error handling to make sure the file can be opened.
    match File::open(path) {
        Ok(f) => { *file_buf = Some(f); },
        Err(e) => { *file_buf = None; eprintln!("Unable to open file, {}", e); }
    }

    // Option error handling to make sure file exists to be able to read it's content.
    match file_buf {
        Some(f) => { content_buf.clear(); f.read_to_string(content_buf).expect("Cannot read from file."); },
        None => { panic!(); }
    }
}

// Having issues saving with permission denied.
pub fn file_save(file_buf: &mut Option<File>, content_buf: &mut String) {

    match file_buf {
        Some(f) => { f.write_all(content_buf.as_bytes()).expect("Problem writing to file."); },
        None => { panic!(); }
    }
}

pub fn file_saveas(file_buf: &mut Option<File>, content_buf: &mut String) {

    let path_opt: Option<PathBuf>;

    // Result error handling to make sure a valid save path was selected.
    match FileDialog::new()
        .add_filter("Text Document", &["txt"])
        .show_save_single_file()
    {
        Ok(p) => { path_opt = p; },
        Err(e) => { panic!("{}", e); }
    }

    let path: PathBuf;

    match path_opt {
        Some(p) => path = p,
        None => return
    }

    match File::create(path) {
        Ok(f) => { *file_buf = Some(f); },
        Err(e) => { panic!("{}", e); }
    }

    match file_buf {
        Some(f) => { f.write_all(content_buf.as_bytes()).expect("Problem writing to file."); },
        None => { panic!(); }
    }
}