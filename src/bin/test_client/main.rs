use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
};

use unionize::{
    easy::uniform::*,
    item::le_byte_array::LEByteArray,
    protocol::{first_message, respond_to_message, Message},
};

fn to_item(s: String) -> Item {
    let mut buf = [0u8; 30];
    let s_bs = s.as_bytes();
    for i in 0..30.min(s_bs.len()) {
        buf[29 - i] = s_bs[i];
    }

    LEByteArray(buf)
}

fn main() -> std::io::Result<()> {
    let items: Vec<Item> = std::env::args().skip(1).map(to_item).collect();

    let mut tree = Node::nil();
    for item in items {
        tree = tree.insert(item);
    }

    let stream = TcpStream::connect("127.0.0.1:2342")?;
    let learned = handle_connection(stream, &tree)?;
    println!("Items learned from the server: {learned:?}");
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

fn handle_connection(mut stream: TcpStream, tree: &Node) -> std::io::Result<Vec<Item>> {
    println!(
        "accepted connection. local_addr:{:?} peer_addr:{:?}",
        stream.local_addr().unwrap(),
        stream.peer_addr().unwrap()
    );

    let mut learned = vec![];

    let first = first_message(tree).unwrap();
    let msg = serde_cbor::to_vec(&first).unwrap();
    write_frame(&mut stream, &msg)?;

    loop {
        let msg_bytes = read_frame(&mut stream)?;
        let msg: Message<Monoid> = serde_cbor::from_slice(&msg_bytes).unwrap();
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
