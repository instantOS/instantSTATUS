use dirs::config_dir;
use std::path::{Path, PathBuf};

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
    fn render(&self) {
        print!("^f{}^", 11);
        print!("{}", self.script);
    }

    fn new(name: String) -> Option<Applet> {
        let mut appletpath = config_dir().unwrap();
        appletpath.push(Path::new(&format!("applets/{}.ist.sh", &name)));
        
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
    let mut confdir = config_dir().unwrap();
    confdir.push(Path::new("instantstatus"));

    if !confdir.exists() {
        std::fs::create_dir_all(confdir).unwrap();
    }

    Applet::new(String::from("hello")).unwrap().render();
}
