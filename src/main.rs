use dirs::config_dir;
use std::path::PathBuf;
use std::process::Command;

use x11rb::connection::Connection;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xproto::*;
use x11rb::wrapper::ConnectionExt;
use x11rb::COPY_DEPTH_FROM_PARENT;

pub struct Statusdata {
    configpath: String,
    configpath_buf: PathBuf,
}

impl Statusdata {
    fn new() -> Statusdata {
        let cpath = config_dir().expect("~/.config not existing or not readable");
        Statusdata {
            configpath_buf: cpath.clone(),
            configpath: String::from(cpath.to_str().unwrap()),
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
    fn render(&self) -> Option<String> {
        match Command::new("bash")
            .arg("-c")
            .arg(&format!(
                "cd && source {} && status_display",
                &self.script_path
            ))
            .output()
        {
            Ok(output) => {
                return Some(format!(
                    "^c#ff0000^^f11^{}^f11^",
                    String::from_utf8(output.stdout).unwrap().trim()
                ));
            }
            Err(_) => {}
        };
        None
    }

    fn new(name: String, data: Statusdata) -> Option<Applet> {
        let mut appletpath = data.configpath_buf.clone();

        appletpath.push(&format!("instantstatus/applets/{}.ist.sh", &name));

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

        let scriptpath = match location {
            Appletlocation::User => format!(
                "{}/instantstatus/applets/{}.ist.sh",
                &data.configpath, &name
            ),
            Appletlocation::Global => {
                format!("/usr/share/instantstatus/applets/{}.ist.sh", &name)
            }
        };

        Some(Applet {
            time: 0,
            script: name,
            script_path: scriptpath,
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

    let tester2 = Applet::new(String::from("hello"), data).unwrap();
    let tester = tester2.render().unwrap();

    println!("this is not supposed to show in the window name");
    println!("{}", &tester);

    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    conn.change_property8(
        PropMode::REPLACE,
        root,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        tester.as_bytes(),
    )
    .unwrap();

    conn.flush().unwrap();
}
