//extern crate rustc_serialize;
extern crate telegram_bot;

use telegram_bot::*;
//use rustc_serialize::json;

fn main() {
    let api = Api::from_env("TELEGRAM_BOT_TOKEN").unwrap();
    println!("getMe: {:?}", api.get_me());

    let mut listener = api.listener(ListeningMethod::LongPoll(None));

    listener.listen(|u| {
        if let Some(m) = u.message {
            if let MessageType::Text(_) = m.msg {
                try!(api.send_message(
                        m.chat.id(),
                        format!("Hi, {}!", m.from.first_name),
                        None, None, None, None)
                    );
            }
        }

        Ok(ListeningAction::Continue)
    });
}
