extern crate telegram_bot;
use telegram_bot::*;

use std::thread;
use std::sync::{Arc, Mutex};

use std::borrow::Cow;

use std::io::prelude::*;
use std::fs::File;

//extern crate rustc_serialize;
//use rustc_serialize::json;

fn do_loop(greeting: Arc<Mutex<String>>) {
    let api = Api::from_env("TELEGRAM_BOT_TOKEN").unwrap();
    println!("getMe: {:?}", api.get_me());

    let mut listener = api.listener(ListeningMethod::LongPoll(None));

    listener.listen(|u| {
        if let Some(m) = u.message {
            if let MessageType::Text(_) = m.msg {
                let hi = greeting.lock().unwrap();
                try!(api.send_message(
                        m.chat.id(),
                        format!("{}, {}!", *hi, m.from.first_name),
                        None, None, None, None)
                    );
            }
        }

        Ok(ListeningAction::Continue)
    });
}

fn read_data() -> Result<String> {
    let mut f = try!(File::open("greeting.txt"));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

fn main() {
    let greeting = Arc::new(Mutex::new(read_data().unwrap_or("Hi".into())));

    let child = thread::spawn(|| {
        do_loop(greeting);
    });
    child.join();
}
