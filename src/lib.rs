pub mod types;

pub fn test_all() -> anyhow::Result<bool> {
    for dir in std::fs::read_dir("./tests")? {
        test_one(dir?)?;
    }

    Ok(true)
}

fn test_one(entry: std::fs::DirEntry) -> anyhow::Result<bool> {
    let file_name = entry.file_name();
    let test_dir = match file_name.to_str() {
        Some(f) => f,
        None => {
            eprintln!("cant convert directory to str: {:?}", entry);
            return Ok(false);
        }
    };

    if !test_dir.starts_with('.') && entry.file_type()?.is_dir() {
        // Not testing fbt as of now
        if test_dir.contains("fbt") {
            return Ok(false);
        }

        println!("current folder {:?}", entry.path());
        let mut input_path = None; // TODO: remove mut, must it be option?
        let mut cmd_toml_path = None; // TODO: remove mut
        for inner_dirs in std::fs::read_dir(entry.path())? {
            let entry1 = inner_dirs?;
            if entry1.file_type()?.is_dir()
                && entry1.file_name().to_str().unwrap_or("").contains("input")
            {
                input_path = Some(entry1.path());
            }
            if entry1
                .file_name()
                .to_str()
                .unwrap_or("")
                .contains("cmd.toml")
            {
                cmd_toml_path = Some(entry1.path());
            }
        }

        if input_path == None || cmd_toml_path == None {
            eprintln!("not a valid test case");
            return Ok(false);
        }

        println!("input: {:?}, cmd.toml {:?}", input_path, cmd_toml_path);

        let contents = std::fs::read_to_string(cmd_toml_path.unwrap())?;
        let test_cmd: crate::types::TestCommand = toml::from_str(&contents)?;
        println!("Command: {:?}", test_cmd);

        let args: Vec<&str> = test_cmd.cmd.split(' ').collect();
        let mut cmd = std::process::Command::new(args[0]);
        cmd.current_dir(input_path.unwrap());
        //will need to add code to handle multiple args
        cmd.arg(args[1]);
        let result = cmd.output()?;
        println!("cmd result {:?}", result);
        if String::from_utf8(result.stdout).unwrap() == test_cmd.stdout.trim()
            && result.status.success()
        {
            println!("Passed");
        } else {
            println!(
                "Failed {:?}",
                String::from_utf8(result.stderr).unwrap().trim()
            );
        }
    }

    Ok(true)
}
