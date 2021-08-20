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

enum AppletAction {
    Display,
    Button1,
    Button2,
    Button3,
    Scrollup,
    Scrolldown,
}

enum AppletType {
    Script,
    Directory,
}

pub struct Applet {
    time: u32,
    script: String,
    path: String,
    location: Appletlocation,
    // boolean for presence of a display file
    has_display: bool,
    has_button1: bool,
    has_button2: bool,
    has_button3: bool,
    has_scrollup: bool,
    has_scrolldown: bool,
    applettype: AppletType,
}

enum StringColor {
    Color0,
    Color1,
    Color2,
    Color3,
    Color4,
    Color5,
    Color6,
    Color7,
    Color8,
    Color9,
    Color10,
    Color11,
    Color12,
    Color13,
    Color14,
    Color15,
    Custom(String),
}

struct StringOptions {
    color: StringColor,
}

impl StringOptions {
    fn new() -> StringOptions {
        StringOptions {
            color: StringColor::Color0,
        }
    }
    fn render(&self, content: &str) -> String {
        if content == "" {
            return String::from("");
        } else {
            return format!("^c#ff0000^^f11^{}^f11^", content.trim());
        }
    }
}

impl Applet {
    // output applet content with markup as string

    fn new(name: String, data: Statusdata) -> Option<Applet> {
        let mut appletpath = data.configpath_buf.clone();

        appletpath.push(&format!("instantstatus/applets/{}", &name));

        let mut location = Appletlocation::Global;

        if appletpath.exists() {
            location = Appletlocation::User;
        } else {
            let globalpath = PathBuf::from(&format!("/usr/share/instantstatus/applets/{}", &name));
            if !globalpath.exists() {
                eprintln!("applet {} does not exist", &name);
                return None;
            }
        }

        let scriptpath = match location {
            Appletlocation::User => format!("{}/instantstatus/applets/{}", &data.configpath, &name),
            Appletlocation::Global => {
                format!("/usr/share/instantstatus/applets/{}", &name)
            }
        };

        // check if scriptpath is a directory or a file
        if PathBuf::from(&scriptpath).is_file() {
            eprintln!("file based applets are not supported yet");
            return None;
        }

        let mut has_display = false;
        let mut has_button1 = false;
        let mut has_button2 = false;
        let mut has_button3 = false;
        let mut has_scrollup = false;
        let mut has_scrolldown = false;
        println!("scriptpath {}", &scriptpath);

        for i in fs::read_dir(&scriptpath).unwrap() {
            let filename = &String::from(i.unwrap().file_name().to_str().unwrap());

            has_display = has_display || filename == "display";
            has_button1 = has_button1 || filename == "button1";
            has_button2 = has_button2 || filename == "button2";
            has_button3 = has_button3 || filename == "button3";
            has_scrollup = has_scrollup || filename == "scrollup";
            has_scrolldown = has_scrolldown || filename == "scrolldown";
        }

        if !has_display {
            eprintln!("warning: applet {} does not have a display file", &name);
        }

        Some(Applet {
            time: 0,
            script: name,
            path: scriptpath,
            location,
            has_display,
            has_button1,
            has_button2,
            has_button3,
            has_scrollup,
            has_scrolldown,
            applettype: AppletType::Directory,
        })
    }

    fn render(&self) -> Option<String> {
        if self.has_display {
            println!(
                "{}",
                format!(" display lalala {}/{}", &self.path, "display")
            );
            match Command::new(&format!("{}/{}", &self.path, "display")).output() {
                Ok(output) => {
                    return Some(
                        StringOptions::new().render(&String::from_utf8(output.stdout).unwrap()),
                    );
                }
                Err(_) => {}
            }
        } else {
            return Some(StringOptions::new().render(&self.script.clone()));
        }
        None
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

    println!("{}", format!("^d^^f11^{}", tester));
    println!("tester {}", tester);
    conn.change_property8(
        PropMode::REPLACE,
        root,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        format!("^d^^f11^{}", tester).as_bytes(),
    )
    .unwrap();

    conn.flush().unwrap();
}
