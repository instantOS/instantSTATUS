use dirs::config_dir;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use clap::{App, AppSettings, Arg, ArgMatches};
use toml::Value;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::wrapper::ConnectionExt;

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
    let default_config = include_str!("../default/config.toml");
    let matches = App::new("instantSTATUS")
        .version("0.0.1")
        .author("paperbenni <paperbenni@gmail.com>")
        .about("simple status bar for instantWM")
        .arg(
            Arg::new("write-config-file")
                .short('w')
                .about("write the default configuration file to stdout (-) or to a file")
                .takes_value(true),
        )
        .setting(AppSettings::ColoredHelp)
        .get_matches();

    if matches.is_present("write-config-file") {
        match matches.value_of("write-config-file") {
            Some(value) => {
                if value == "-" {
                    println!("{}", default_config);
                } else {
                    match OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(value)
                        .unwrap()
                        .write(default_config.as_bytes())
                    {
                        Ok(_) => {}
                        Err(_) => {
                            println!("{}", value);
                        }
                    }
                }
            }
            None => {
                println!("{}", default_config);
            }
        }
        return;
    }

    let data = Statusdata::new();

    let mut confdir = config_dir().unwrap();
    confdir.push("instantstatus/applets");

    let mut tomlpath = data.configpath_buf.clone();
    tomlpath.push("instantstatus/config.toml");

    if !confdir.exists() {
        std::fs::create_dir_all(confdir).expect("could not create applet directory");
    }

    let mut use_default_config = false;

    if !tomlpath.is_file() {
        use_default_config = true;
        match OpenOptions::new()
            .write(true)
            .create(true)
            .open(&tomlpath)
            .unwrap()
            .write(default_config.as_bytes())
        {
            Ok(_) => {
                println!(
                    "initialized default config file in {}",
                    &tomlpath.to_str().unwrap()
                );
            }
            Err(_) => {
                eprintln!("Warning: Could not create default config file");
            }
        }
    }

    let tomlcontent: String;

    if use_default_config {
        tomlcontent = String::from(default_config);
    } else {
        tomlcontent = match fs::read_to_string(tomlpath) {
            Ok(content) => content,
            Err(_) => {
                eprintln!("Warning: could not read user config file");
                String::from(default_config)
            }
        }
    }

    // TODO: more useful error message
    let config: Value = tomlcontent.parse().expect("error in config file");

    match config.get("applet") {
        Some(applets) => {
            for i in applets.as_array().unwrap() {
                println!("hello {:?}", i);
            }
        }
        None => {}
    }

    // down here is testing stuff
    let tester2 = Applet::new(String::from("hello"), data).unwrap();
    let tester = tester2.render().unwrap();

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
