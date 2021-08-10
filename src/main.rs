use dirs::config_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub struct Statusdata {
    configpath: String,
}

impl Statusdata {
    fn new() -> Statusdata {
        Statusdata {
            configpath: String::from(config_dir().unwrap().to_str().unwrap()),
        }
    }
}

enum Appletlocation {
    User,
    Global,
}

pub struct Applet {
    time: u32,
    script: String,
    script_path: String,

    location: Appletlocation,
}

impl Applet {
    fn render(&self, data: &Statusdata) -> Option<String> {
        match Command::new("bash")
            .arg("-c")
            .arg(format!(
                "cd && source {} && status_display",
                self.script_path
            ))
            .output()
        {
            Ok(output) => {
                return Some(format!(
                    "^c#ff0000^^f11^{}^f11^",
                    String::from_utf8(output.stdout).unwrap()
                ));
            }
            Err(_) => {}
        };
        None
    }

    fn new(name: String, data: &Statusdata) -> Option<Applet> {
        let mut script_path = PathBuf::from(&data.configpath);
        script_path.push(format!("instantstatus/applets/{}.ist.sh", &name));

        let mut location = Appletlocation::Global;

        if script_path.is_file() {
            location = Appletlocation::User;
        } else {
            script_path =
                PathBuf::from(format!("/usr/share/instantstatus/applets/{}.ist.sh", &name));

            if script_path.is_file() {
            } else {
                eprintln!("applet {} does not exist", &name);
                return None;
            }
        }

        Some(Applet {
            time: 0,
            script: name,
            script_path: String::from(script_path.to_str().unwrap()),
            location,
        })
    }
}

fn main() {
    let data = Statusdata::new();
    let mut confdir = config_dir().unwrap();
    confdir.push("instantstatus/applets");

    if !confdir.exists() {
        std::fs::create_dir_all(confdir).unwrap();
    }

    let tester2 = Applet::new(String::from("hello"), &data).unwrap();
    let tester = tester2.render(&data).unwrap();
    println!("{}", tester)
}
