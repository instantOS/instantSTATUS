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
    location: Appletlocation,
}

impl Applet {
    // fn getpath(&self) -> String {
    //     match self.location {
    //         User => {
    //             return String::from(&format("{}"))
    //         }
    //     }
    //     String::from("asd")
    // }

    fn render(&self, data: &Statusdata) -> Option<String> {

        let scriptpath = match &self.location {
            User => format!(
                "{}/instantstatus/applets/{}.ist.sh",
                &data.configpath, &self.script
            ),
            Global => format!(
                "/usr/share/instantstatus/applets/{}.ist.sh",
                &self.script
            ),
        };

        match Command::new("bash")
            .arg("-c")
            .arg(&format!("cd && source {} && status_display", &scriptpath))
            .output()
        {
            Ok(output) => {
                return Some(String::from(str::from_utf8(&output.stdout).unwrap()));
            }
            Err(_) => {}
        };
        None
    }

    fn new(name: String) -> Option<Applet> {
        let mut appletpath = config_dir().unwrap();

        appletpath.push(Path::new(&format!(
            "instantstatus/applets/{}.ist.sh",
            &name
        )));

        println!("{}", appletpath.to_str().unwrap());

        let mut location = Appletlocation::Global;

        if appletpath.is_file() {
            location = Appletlocation::User;
        } else {
            let globalpath = PathBuf::from(&format!(
                "/usr/share/instantstatus/applets/{}.ist.sh",
                &name
            ));
            if globalpath.is_file() {
            } else {
                eprintln!("applet {} does not exist", &name);
                return None;
            }
        }

        Some(Applet {
            time: 0,
            script: name,
            location,
        })
    }
}

fn main() {
    let data = Statusdata::new();
    let mut confdir = config_dir().unwrap();
    confdir.push(Path::new("instantstatus/applets"));

    if !confdir.exists() {
        std::fs::create_dir_all(confdir).unwrap();
    }

    let tester2 = Applet::new(String::from("hello")).unwrap();
    let tester = tester2.render(&data).unwrap();
    println!("{}", tester)
}
