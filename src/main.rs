extern crate telegram_bot;
use telegram_bot::*;

use std::thread;

//extern crate rustc_serialize;
//use rustc_serialize::json;

fn do_loop(greeting: &str) {
    let api = Api::from_env("TELEGRAM_BOT_TOKEN").unwrap();
    println!("getMe: {:?}", api.get_me());

    let mut listener = api.listener(ListeningMethod::LongPoll(None));

    listener.listen(|u| {
        if let Some(m) = u.message {
            if let MessageType::Text(_) = m.msg {
                try!(api.send_message(
                        m.chat.id(),
                        format!("{}, {}!", greeting, m.from.first_name),
                        None, None, None, None)
                    );
            }
        }

        Ok(ListeningAction::Continue)
    });
}

fn main() {
    let child = thread::spawn(|| {
        do_loop("Hi");
    });
    child.join();
}
