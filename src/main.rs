use std::fs::{self, read_dir};

fn main() {
    for dirs in read_dir("./tests").unwrap() {
        let entry = dirs.unwrap();
        let file_name = entry.file_name();
        let test_dir = file_name.to_str().unwrap();
        if !test_dir.starts_with(".") && entry.file_type().unwrap().is_dir() {
            println!("current folder {:?}", entry.path());
            let mut input_path = None;
            let mut cmd_toml_path = None;
            for inner_dirs in read_dir(entry.path()).unwrap() {
                let entry1 = inner_dirs.unwrap();
                if entry1.file_type().unwrap().is_dir()
                    && entry1.file_name().to_str().unwrap().contains("input")
                {
                    input_path = Some(entry1.path());
                }
                if entry1.file_name().to_str().unwrap().contains("cmd.toml") {
                    cmd_toml_path = Some(entry1.path());
                }
            }

            if input_path == None || cmd_toml_path == None {
                //Error
                println!("not a valid test case");
            }

            println!("input : {:?}, cmd.toml {:?}", input_path, cmd_toml_path);

            let contents = fs::read_to_string(cmd_toml_path.unwrap())
                .expect("Something went wrong reading the file");
            println!("{:?}", contents);
        }
    }
}
