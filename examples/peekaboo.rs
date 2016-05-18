extern crate loirc;
extern crate encoding;

use std::env;
use std::io;
use std::time::Duration;

use loirc::{Code, ConnectionManager, Connection, Message, RawEvents, Writer};

/// Say "peekaboo" in a channel on freenode and then quit.
/// target/debug/examples/peekaboo "#mychannel"
fn main() {
    let args: Vec<String> = env::args().collect();
    let channel = args.get(1).expect("Channel must be given as an argument.");

    // Connect to freenode and use no not reconnect.
    let mut cm = ConnectionManager::new();

    struct H(String);

    impl RawEvents for H {
        fn connect(&mut self, mut conn: Writer) {
            println!("Connected");
            conn.write("USER simon 8 * :simon\n");
            conn.write("NICK peekaboo\n");
        }
        fn disconnect(&mut self) {
            println!("Disconnected");
        }
        fn message(&mut self, mut w: Writer, msg: Message) {
            println!("{:?}", msg);
            match msg.code {
                Code::RplWelcome => {
                    w.write(format!("JOIN {}\n", self.0));
                    w.write("PING DEADBEEF\n");
                }
                Code::Join => {
                    w.write(format!("PRIVMSG {} :peekaboo\n", self.0));
                    //w.write(format!("QUIT :peekaboo\n"));
                }
                _ => {}
            }
        }
    }

    cm.connect("irc.mozilla.org:6667", H(channel.clone()), Some(Duration::from_secs(5)), None).unwrap();

    println!("Post connect");

    cm.run();
}
