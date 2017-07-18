#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate ws;

use std::sync::{Mutex, Arc};
use std::thread;
use std::path::{PathBuf, Path};
use rocket::*;
use rocket::response::NamedFile;

use std::time::Instant;


struct Handler {
    clients: Arc<Mutex<Vec<ws::Sender>>>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl ws::Handler for Handler {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.messages.lock().unwrap().push(msg.to_string());
        for client in &*self.clients.lock().unwrap() {
            let _ = client.send(msg.clone());
        }
        Ok(())
    }
}

struct ChatServer {
    clients: Arc<Mutex<Vec<ws::Sender>>>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl ChatServer {
    pub fn launch() -> Self {
        let clients = Arc::new(Mutex::new(Vec::new()));
        let messages = Arc::new(Mutex::new(Vec::new()));

        let clients1 = clients.clone();
        let messages1 = messages.clone();
        thread::spawn(move || {
            let clients = clients1;
            let messages = messages1;
            ws::listen("localhost:8001", |sender| {
                let _ = sender.send(format!("{:?} - History:", Instant::now()));
                {
                    let messages = messages.lock().unwrap().clone();
                    for message in messages {
                        let _ = sender.send(message);
                    }
                }
                clients.lock().unwrap().push(sender);
                Handler {
                    clients: clients.clone(),
                    messages: messages.clone(),
                }
            }).unwrap()
        });

        ChatServer {
            clients: clients,
            messages: messages,
        }
    }
}


#[get("/")]
fn home(state: State<ChatServer>) -> NamedFile {
    NamedFile::open("assets/index.html").unwrap()
}

#[get("/test")]
fn test(state: State<ChatServer>) -> String {
    unimplemented!()
}


#[get("/assets/<file..>")]
fn assets(file: PathBuf) -> Option<NamedFile> {
    let path = Path::new("assets").join(file);
    NamedFile::open(path).ok()
}

fn main() {
    println!("Hello, world!");

    let server = ChatServer::launch();


    let _ = rocket::ignite()
        .manage(server)
        .mount("/", routes![home, assets, test])
        .launch();
}
