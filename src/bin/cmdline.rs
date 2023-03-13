use std::io::stdin;

use e2eoffline::{E2EOffline, E2EOfflineBuilder};

macro_rules! readline {
    ($buffer:ident) => {
        $buffer.clear();
        stdin().read_line(&mut $buffer).unwrap();
    };
}

fn main() -> anyhow::Result<()> {
    let mut buffer = String::new();

    let mut e2e = loop {
        println!("s to send, r to recieve, k to use an existing key");

        readline!(buffer);
        let r = buffer.trim().to_lowercase();

        match r.as_str() {
            "k" => {
                println!("Shared Key?");
                readline!(buffer);

                break E2EOffline::from_key_base64(&buffer);
            }
            "r" => {
                println!("Recieving!");
                let mut reciever = E2EOfflineBuilder::new_reciever();
                println!("Your public key is {}", reciever.get_pubkey_encoded()?,);
                println!("Sender public key? (preferably exchanged with them in person)");
                readline!(buffer);

                reciever.set_other_public_key_encoded(&buffer)?;

                println!("Sender key exchange text?");

                readline!(buffer);

                reciever.recieve(&buffer)?;

                println!(
                    "Your key is {}, do not send it to anyone",
                    reciever.get_shared_key()?
                );

                break reciever.build();
            }
            "s" => {
                println!("Sending!");
                let mut sender = E2EOfflineBuilder::new_sender();
                println!("Your public key is {}.", sender.get_pubkey_encoded()?);

                println!("Reciever public key? (preferably exchanged with them in person)");
                readline!(buffer);

                sender.set_other_public_key_encoded(&buffer)?;

                let token = sender.send()?;

                println!("Your key exchange text is {token}. Send it to the other user");
                println!(
                    "Your key is {}, do not send it to anyone",
                    sender.get_shared_key()?
                );

                break sender.build();
            }

            _ => println!("Bad choice"),
        }
    }?;

    loop {
        println!("e for encrypt, d for decrypt, q for quit");
        readline!(buffer);

        match buffer.trim().to_lowercase().as_str() {
            "e" => {
                println!("Plaintext?");

                readline!(buffer);
                println!("{}", e2e.encrypt(&buffer)?);
            }

            "d" => {
                println!("Ciphertext?");

                readline!(buffer);
                println!("{}", e2e.decrypt(&buffer)?);
            }

            "q" => break,

            _ => println!("bad choice"),
        }
    }

    Ok(())
}
