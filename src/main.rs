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
extern crate glob;
use glob::glob;

// Parsing yaml
extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

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

// Calling shell commands
use std::process::Command;

#[derive(Default,Debug)]
struct Action {
    action: String,
    title: String,
    command: String,
    hidden: bool,
    users: Vec<String>
}

fn init_logging(enable_debug: bool) {
  let log_level = if enable_debug {
    Some("telekey=debug".into())
  } else {
    Some("telekey=info".into())
  };
  flexi_logger::init(flexi_logger::LogConfig::new(), log_level).unwrap();
}

fn parse_action(yaml: &Yaml) -> Action {
    let action_name = yaml["action"].as_str().unwrap_or("<unknown>");
    Action {
        action: action_name.into(),
        title: yaml["title"].as_str().unwrap_or(action_name).into(),
        command: yaml["command"].as_str().unwrap_or("").into(),
        hidden: yaml["hidden"].as_bool().unwrap_or(false),
        users: yaml["users"].as_vec().unwrap_or(&vec![]).iter().map(|x| x.as_str().unwrap_or("").into()).collect()
    }
}

fn load_config(settings: &Arc<Mutex<Vec<Action>>>) {
    debug!("Loading configs");

    let mut new_config: Vec<Action> = vec![];
    if let Ok(paths) = glob("config/*.yml") {
      for file in paths {
        if let Ok(ref file) = file {
          if let Ok(mut f) = File::open(file) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
              if let Ok(yaml) = YamlLoader::load_from_str(&s) {
                let action = parse_action(&yaml[0]);
                debug!("Action {:?} loaded", action);
                new_config.push(action);
              } else {
                error!("File {:?} is not a valid YAML", file);
              }
            } else {
              error!("Failed to read file {:?} contents", file);
            }
          } else {
            error!("Failed to read a file {:?}", file);
          }
        }
      }
      info!("Loaded actions: {:?}", new_config.iter().map(|action| &action.action).collect::<Vec<_>>());
      if let Ok(mut settings) = settings.lock() {
        *settings = new_config;
      } else {
        warn!("Failed to lock mutex on settings!");
      }
    } else {
      error!("No configs found!");
      if let Ok(mut settings) = settings.lock() {
        *settings = vec![];
      } else {
        warn!("Failed to lock mutex on settings!");
      }
    }
}

fn handle_sighup(settings: &Arc<Mutex<Vec<Action>>>) {
    load_config(settings);
    debug!("Config reloaded");
}

fn generate_keyboard(actions: &Vec<Action>) -> Option<ReplyMarkup> {
    Some(ReplyKeyboardMarkup {
        keyboard: vec![actions.iter().map(|a| a.action.clone()).collect()],
        ..Default::default()
    }.into())
}

fn generate_actions_list(actions: &Vec<Action>) -> String {
    let mut buff = String::new();
    for action in actions.iter() {
        buff = format!("{}\n {} - {}", buff, action.action, action.title);
    }
    buff
}

fn call(commandline: &String) -> bool {
    let mut arguments: Vec<&str> = commandline.split_whitespace().collect();
    let command = arguments.remove(0);
    debug!("Calling {:?} with arguments {:?}", command, arguments);
    Command::new(command).args(&arguments).spawn().is_ok()
}

fn handle_telegram(api: &Api, settings: &Arc<Mutex<Vec<Action>>>) {
    let mut listener = api.listener(ListeningMethod::LongPoll(None));

    listener.listen(|u| {
        if let Some(m) = u.message {
            debug!("Got {:?}", m);
            if let MessageType::Text(requested_action) = m.msg {

              if let Some(requested_username) = m.from.username {
                if let Ok(actions) = settings.lock() {
                  let keyboard = generate_keyboard(&actions);

                  if requested_action == "/start" {
                    debug!("Sending welcome message");
                    if api.send_message(
                      m.chat.id(),
                      format!("Привет, {}!\nЯ умею следующее: {}", m.from.first_name, generate_actions_list(&actions)),
                      None, None, None, keyboard.clone()
                      ).is_err() {
                      error!("Failed to send welcome message to {}", m.from.first_name)
                    }
                  } else {
                    for action in actions.iter() {
                      if action.action == requested_action {
                        debug!("Found matching action {:?}", action);
                        if action.users.iter().any(|allowed| *allowed == requested_username) {
                          debug!("Action authorized");
                          let message = if call(&action.command) {
                            format!("{}, {}!", action.title, m.from.first_name)
                          } else {
                            format!("Не удалось {}, {}!", action.title, m.from.first_name)
                          };
                          if api.send_message( m.chat.id(), message, None, None, None, keyboard.clone()).is_err() {
                            error!("Failed to send message to {}", m.from.first_name)
                          }
                        } else {
                          debug!("Action denied; user is not in users list");
                          if api.send_message(
                            m.chat.id(),
                            format!("Пошёл нафиг, {}!", m.from.first_name),
                            None, None, None, keyboard.clone()
                            ).is_err() {
                            error!("Failed to send access denied message to {}", m.from.first_name)
                          }
                        }
                      }
                    }
                  }
                } else {
                  error!("Failed to grab mutex on settings!");
                }
              } else {
                error!("Failed to read username from {:?}", m.from);
              }
            }
        }

        Ok(ListeningAction::Continue)
    }).unwrap();
}

fn main() {
    let cli_options_config = load_yaml!("cli.yml");
    let cli_options = App::from_yaml(cli_options_config)
                          .setting(clap::AppSettings::ArgRequiredElseHelp)
                          .get_matches();

    init_logging(cli_options.is_present("debug"));


    let greeting = Arc::new(Mutex::new(vec![]));
    load_config(&greeting);

    let handler_greeting = greeting.clone();
    Signals::set_handler(&[Signal::Hup], move |_signals| {
        info!("Got SIGHUP, reloading");
        let greeting = handler_greeting.clone();
        thread::spawn(move || {
            handle_sighup(&greeting);
        });
    });


    let telegram_bot_token = cli_options.value_of("TELEGRAM_BOT_TOKEN").unwrap();
    let api = Api::from_token(telegram_bot_token).unwrap();
    info!("Bot connected: {:?}", api.get_me().unwrap());

    let telegram_thread = thread::spawn(move || {
      handle_telegram(&api, &greeting);
    });

    telegram_thread.join().unwrap();
}
