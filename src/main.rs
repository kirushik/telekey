extern crate telegram_bot;
use telegram_bot::*;

use std::sync::Mutex;
use std::sync::Arc;

use std::io::prelude::*;
use std::fs::File;

extern crate simple_signal;
use simple_signal::{Signals, Signal};

//extern crate rustc_serialize;
//use rustc_serialize::json;

fn load_config() -> Result<String> {
    let mut f = try!(File::open("greeting.txt"));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}


fn main() {
    let greeting = Arc::new(Mutex::new(load_config().unwrap_or("Hi".into())));
    let trap_greeting = greeting.clone();

    Signals::set_handler(&[Signal::Hup], move |_signals| {
        let mut greeting = trap_greeting.lock().unwrap();
        *greeting = load_config().unwrap_or("Hi".into());
    });

    let api = Api::from_env("TELEGRAM_BOT_TOKEN").unwrap();
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
    });
}
