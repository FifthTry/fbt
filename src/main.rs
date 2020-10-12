use std::fs::read_dir;

fn main() {
    for entry_res in read_dir("./tests").unwrap() {
        let entry = entry_res.unwrap();
        let file_name_buf = entry.file_name();
        let file_name = file_name_buf.to_str().unwrap();
        if !file_name.starts_with(".") &&
            entry.file_type().unwrap().is_dir()
        {
            println!("Test Case {:?} has full path {:?}",
                     file_name, entry.path());
        }
    }
}
