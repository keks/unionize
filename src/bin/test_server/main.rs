use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

use unionize::{
    easy::uniform::*,
    protocol::{respond_to_message, Message},
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2342")?;

    let mut tree = Node::nil();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let new_items = handle_connection(stream, &mut tree)?;

        for item in new_items {
            // TODO actually this vector will contain some false positives, i.e. values we already
            // know. we need a function to filter these out, and it should operate over both the
            // entire tree and the the list at once.
            tree = tree.insert(item);
            println!("learned {item:?}")
        }
    }

    Ok(())
}

fn write_frame(stream: &mut TcpStream, payload: &[u8]) -> std::io::Result<()> {
    let l: u16 = payload.len().try_into().unwrap();
    let l_bs = l.to_be_bytes();

    stream.write(&l_bs)?;
    stream.write(payload)?;
    Ok(())
}

fn read_frame(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut l_bs = [0u8; 2];
    stream.read(&mut l_bs)?;
    let l = u16::from_be_bytes(l_bs) as usize;

    let mut buf = vec![0u8; l];
    stream.read(&mut buf)?;

    println!("received data");
    std::io::stdout().flush()?;

    Ok(buf)
}

fn handle_connection(mut stream: TcpStream, tree: &mut Node) -> std::io::Result<Vec<Item>> {
    println!(
        "accepted connection. local_addr:{:?} peer_addr:{:?}",
        stream.local_addr().unwrap(),
        stream.peer_addr().unwrap()
    );

    let mut learned = vec![];

    loop {
        let payload = read_frame(&mut stream)?;
        let msg: Message<Monoid> = serde_cbor::from_slice(&payload).unwrap();
        if msg.is_end() {
            break;
        }

        let (resp, new_items) = respond_to_message(tree, &msg, 3, split::<2>).unwrap();
        learned.extend(&new_items);

        let msg_bytes = serde_cbor::to_vec(&resp).unwrap();
        write_frame(&mut stream, &msg_bytes)?;
        if resp.is_end() {
            break;
        }
    }

    stream.shutdown(Shutdown::Both)?;
    Ok(learned)
}
