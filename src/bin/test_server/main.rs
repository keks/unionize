use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

use unionize::{
    easy::uniform::*,
    object::Object,
    protocol::{respond_to_message, Message},
};

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestObject(Item);

impl Object<Item> for TestObject {
    fn to_item(&self) -> Item {
        self.0.clone()
    }

    fn validate_self_consistency(&self) -> bool {
        true
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2342")?;
    println!("listening in {}", listener.local_addr()?);

    let mut tree = Node::nil();
    let mut objects = BTreeMap::new();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &mut tree, &mut objects)?;
    }

    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    tree: &mut Node,
    objects: &mut BTreeMap<Item, TestObject>,
) -> std::io::Result<()> {
    loop {
        let payload = read_frame(&mut stream)?;
        let msg: Message<Monoid, TestObject> = serde_cbor::from_slice(&payload).unwrap();
        if msg.is_end() {
            break;
        }

        let (resp, new_objs) = respond_to_message(tree, objects, &msg, 3, split::<2>).unwrap();
        for obj in new_objs {
            *tree = tree.insert(obj.to_item());
            objects.insert(obj.to_item(), obj);
        }

        let msg_bytes = serde_cbor::to_vec(&resp).unwrap();
        write_frame(&mut stream, &msg_bytes)?;
        if resp.is_end() {
            break;
        }
    }

    stream.shutdown(Shutdown::Both)?;
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

    Ok(buf)
}
