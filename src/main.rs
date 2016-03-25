// Telegram Bot API support
extern crate telegram_bot;
use telegram_bot::*;

// Threading and synchronization
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;

// Reading stuff from files
use std::io::prelude::*;
use std::fs::File;

// Posix signals handling
extern crate simple_signal;
use simple_signal::{Signals, Signal};

// Command line parsing
#[macro_use]
extern crate clap;
use clap::App;

// Logging
#[macro_use]
extern crate log;
extern crate flexi_logger;


fn init_logging(enable_debug: bool) {
  let log_level = if enable_debug {
    Some("telekey=debug".into())
  } else {
    Some("telekey=info".into())
  };
  flexi_logger::init(flexi_logger::LogConfig::new(), log_level).unwrap();
}

fn load_config() -> Result<String> {
    debug!("Loading config");
    let mut f = try!(File::open("greeting.txt"));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    debug!("Successfuly loaded config");
    Ok(s)
}

fn handle_sighup(settings: Arc<Mutex<String>>) {
    let mut settings = settings.lock().unwrap();
    *settings = load_config().unwrap_or(String::new());
    debug!("Config reloaded");
}

fn main() {
    let cli_options_config = load_yaml!("cli.yml");
    let cli_options = App::from_yaml(cli_options_config)
                          .setting(clap::AppSettings::ArgRequiredElseHelp)
                          .get_matches();

    init_logging(cli_options.is_present("debug"));


    let greeting = Arc::new(Mutex::new(load_config().unwrap_or("Hi".into())));

    let trap_greeting = greeting.clone();
    Signals::set_handler(&[Signal::Hup], move |_signals| {
        info!("Got SIGHUP, reloading");
        let greeting = trap_greeting.clone();
        thread::spawn(move || {
            handle_sighup(greeting);
        });
    });


    let telegram_bot_token = cli_options.value_of("TELEGRAM_BOT_TOKEN").unwrap();
    let api = Api::from_token(telegram_bot_token).unwrap();
    println!("getMe: {:?}", api.get_me());

    let mut listener = api.listener(ListeningMethod::LongPoll(None));

    listener.listen(|u| {
        if let Some(m) = u.message {
            if let MessageType::Text(_) = m.msg {
                let greeting = greeting.lock().unwrap();
                try!(api.send_message(
                        m.chat.id(),
                        format!("{}, {}!", *greeting, m.from.first_name),
                        None, None, None, None)
                    );
            }
        }

        Ok(ListeningAction::Continue)
    }).unwrap();
}
