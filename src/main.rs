extern crate telegram_bot;
use telegram_bot::*;

use std::sync::Mutex;
use std::sync::Arc;

use std::io::prelude::*;
use std::fs::File;

extern crate simple_signal;
use simple_signal::{Signals, Signal};

extern crate clap;
use clap::{Arg, App, AppSettings};

//extern crate rustc_serialize;
//use rustc_serialize::json;

fn load_config() -> Result<String> {
    let mut f = try!(File::open("greeting.txt"));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}


fn main() {

    let matches = App::new("telekey")
                      .version("0.1.0")
                      .author("Kirill Pimenov <kirill@pimenov.cc>")
                      .about("Telegram door opener (in a broad sense)")

                      .arg(Arg::with_name("TELEGRAM_BOT_TOKEN")
                           .long("bot-token")
                           .short("t")
                           .takes_value(true)
                           .required(true)
                           .help("API token of a Telegram bot (please get it from @BotFather)"))

                      .setting(AppSettings::ArgRequiredElseHelp)

                      .get_matches();

    let telegram_bot_token = matches.value_of("TELEGRAM_BOT_TOKEN").unwrap();

    let greeting = Arc::new(Mutex::new(load_config().unwrap_or("Hi".into())));
    let trap_greeting = greeting.clone();

    Signals::set_handler(&[Signal::Hup], move |_signals| {
        let mut greeting = trap_greeting.lock().unwrap();
        *greeting = load_config().unwrap_or("Hi".into());
    });

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
