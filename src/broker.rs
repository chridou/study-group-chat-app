use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use ws;
use ws::util::Token;

enum Command {
    Connect(ws::Sender),
    Broadcast(Token, String),
    Disconnect(Token),
}

pub struct Broker {
    clients: HashMap<Token, ws::Sender>,
    history: Vec<String>,
}

impl Broker {
    pub fn spawn() -> BrokerHandle {
        let broker = Broker {
            clients: HashMap::new(),
            history: Vec::new(),
        };

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || broker.run(rx));

        BrokerHandle(tx)
    }

    fn run(mut self, rx: mpsc::Receiver<Command>) {
        for cmd in rx {
            match cmd {
                Command::Connect(client) => {
                    println!("Client with id {} connected!", client.token().0);

                    let _ = client.send("--- History start ---");
                    for msg in &self.history {
                        let _ = client.send(msg.clone());
                    }
                    let _ = client.send("--- History end ---");

                    self.clients.insert(client.token(), client);
                },
                Command::Broadcast(id, msg) => {
                    let msg = format!("{} says: {}", id.0, msg);
                    self.history.push(msg.clone());
                    self.send_all(msg);
                },
                Command::Disconnect(id) =>  {
                    println!("Client with id {} disconnected!",id.0);
                    self.clients.remove(&id);
                    self.send_all(format!("{} disconnected!", id.0));
                },
            }
        }
    }

    fn send_all(&mut self, msg: String) {
        for client in self.clients.values() {
            let _ = client.send(msg.clone());
        }
    }
}

#[derive(Clone,Debug)]
pub struct BrokerHandle(mpsc::Sender<Command>);

impl BrokerHandle {
    pub fn connect(&self, sender: ws::Sender) {
        let _ = self.0.send(Command::Connect(sender));
    }

    pub fn broadcast<S: Into<String>>(&self, id: Token, msg: S) {
        let msg = msg.into();

        let _ = self.0.send(Command::Broadcast(id, msg));
    }

    pub fn disconnect(&self, id: Token) {
        let _ = self.0.send(Command::Disconnect(id));
    }
}
