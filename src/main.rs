use anyhow::Result as anyhowResult;
use crossterm::{
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, Result,
};
use crossterm::event::{read, Event};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thrussh::server::{Auth, Session};
use thrussh::*;
use thrussh_keys::key::{Name, SignatureHash};

mod keys;
mod term;
mod utils;

const SSH_SERVER_KEY_PATH: &str = "./config/ssh_server_key";

#[tokio::main]
async fn main() {
    if !Path::new(SSH_SERVER_KEY_PATH).exists() {
        println!("Generating SSH server key...");
        let key_pair = keys::generate_rsa(4096);
        let private_key = key_pair.private_key_to_pem().unwrap();
        fs::write(SSH_SERVER_KEY_PATH, private_key).unwrap();
    }

    let rsa_pair = keys::load_rsa_pem(SSH_SERVER_KEY_PATH);
    let server_keys = thrussh_keys::key::KeyPair::RSA {
        key: rsa_pair,
        hash: SignatureHash::SHA2_256,
    };
    let fingerprint = server_keys.clone_public_key().fingerprint();

    let mut server_config = thrussh::server::Config::default();
    server_config.connection_timeout = Some(std::time::Duration::from_secs(120));
    server_config.auth_rejection_time = std::time::Duration::from_secs(3);
    server_config.keys.push(server_keys);
    server_config.server_id = String::from("SSH-2.0-Cloud Console v0.1.0");
    server_config.auth_banner = Some(include_str!("../config/auth_banner.txt"));
    server_config.preferred.key = &[Name("rsa-sha2-256")];
    let server_config = Arc::new(server_config);

    let server = Server {
        // client_pubkey,
        addrs: Arc::new(Mutex::new(HashMap::new())),
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 0,
    };

    println!("Server fingerprint: {}", fingerprint);
    thrussh::server::run(server_config, "0.0.0.0:2222", server)
        .await
        .unwrap();
}

#[derive(Clone)]
struct Server {
    // client_pubkey: Arc<thrussh_keys::key::PublicKey>,
    addrs: Arc<Mutex<HashMap<usize, Option<std::net::SocketAddr>>>>,
    clients: Arc<Mutex<HashMap<(usize, ChannelId), thrussh::server::Handle>>>,
    id: usize,
}

impl server::Server for Server {
    type Handler = Self;
    fn new(&mut self, addr: Option<std::net::SocketAddr>) -> Self {
        let mut addrs = self.addrs.lock().unwrap();
        addrs.insert(self.id, addr);
        let s = self.clone();
        self.id += 1;
        s
    }
}

impl server::Handler for Server {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<anyhowResult<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<anyhowResult<(Self, Session), anyhow::Error>>;
    type FutureBool = futures::future::Ready<anyhowResult<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        futures::future::ready(Ok((self, auth)))
    }
    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool {
        futures::future::ready(Ok((self, s, b)))
    }
    fn finished(self, s: Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, s)))
    }

    fn auth_password(self, _: &str, _: &str) -> Self::FutureAuth {
        self.finished_auth(Auth::Accept)
    }

    fn channel_open_session(self, channel: ChannelId, mut session: Session) -> Self::FutureUnit {
        let mut ipaddr: String = "Unknown".to_string();
        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert((self.id, channel), session.handle());
            let addrs = self.addrs.lock().unwrap();
            let addr = addrs.get(&self.id).unwrap().clone();
            if let Some(addr) = addr {
                ipaddr = utils::ip::ipaddr_lookup(addr);
            }
        }

        session.data(
            channel,
            CryptoVec::from_slice(format!("{esc}c", esc = 27 as char).as_bytes().as_ref()),
        );
        // session.data(channel, CryptoVec::from_slice(format!("Hello, my friend from {}. :)\n\r", ipaddr).as_bytes().as_ref()));
        // session.data(channel, CryptoVec::from_slice(b">"));

        let mut term = term::Term::new(channel, &mut session);

        execute!(
            term,
            SetForegroundColor(Color::Blue),
            SetBackgroundColor(Color::Red),
            Print("Styled text here."),
            ResetColor
        )
        .unwrap();

        self.finished(session)
    }

    fn data(self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        // session.data(channel, CryptoVec::from_slice(b"Hello, world!\n"));
        session.close(channel);
        self.finished(session)
    }
}
