
trait StrategyV0 {
    fn split<M: ProtocolMonoid>(
        range: &Range<M::Item>,
        own_fp: &M,
        root: &Rc<Node<M>>,
    ) -> Vec<Range<M::Item>>;

    fn decide_send_item_set<M: ProtocolMonoid>(
        range: &Range<M::Item>,
        own_fp: &M,
        root: &Rc<Node<M>>,
    ) -> bool;
}

struct ProtocolV0<M: ProtocolMonoid, S: StrategyV0>
where
    M::Item: Display,
{
    root: Rc<Node<M>>,
    new_items: Vec<M::Item>,
    _strategy: PhantomData<S>,
}

impl<M: ProtocolMonoid, S: StrategyV0> ProtocolV0<M, S>
where
    M::Item: Display + Debug,
{
    fn respond(&mut self, msg: &Message<M>) -> Message<M> {
        let Message(parts) = msg;

        let mut resp_parts = vec![];

        'L: for (range, part) in parts {
            match part {
                MessagePart::Fingerprint(fp) => {
                    let own_fp = self.root.query_range_monoid(range);
                    if &own_fp == fp {
                        break 'L;
                    }

                    let next_ranges = S::split(range, &own_fp, &self.root);

                    resp_parts.extend(next_ranges.into_iter().map(|range| {
                        let m = self.root.query_range_monoid(&range);
                        if S::decide_send_item_set(&range, &m, &self.root) {
                            let items = Node::items(self.root.clone(), range.clone()).collect();
                            (range, MessagePart::item_set(items, false))
                        } else {
                            (range, MessagePart::Fingerprint(m))
                        }
                    }))
                }
                MessagePart::ItemSet(items, already_received) => {
                    self.new_items.extend(items.iter().cloned());
                    let own_items = Node::items(Rc::clone(&self.root), range.clone()).collect();

                    if !already_received {
                        resp_parts.push((range.clone(), MessagePart::item_set(own_items, true)))
                    }
                }
            }
        }

        Message(resp_parts)
    }
}

trait StrategyV1:
    Iterator<
    Item = (
        Range<<Self::Monoid as LiftingMonoid>::Item>,
        MessagePart<Self::Monoid>,
    ),
>
{
    type Monoid: ProtocolMonoid;

    fn new(
        range: Range<<Self::Monoid as LiftingMonoid>::Item>,
        fp: Self::Monoid,
        root: Rc<Node<Self::Monoid>>,
    ) -> Self;
}

struct ProtocolV1<M: ProtocolMonoid, S: StrategyV1>
where
    M::Item: Display,
{
    root: Rc<Node<M>>,
    new_items: Vec<M::Item>,
    _strategy: PhantomData<S>,
}

impl<M: ProtocolMonoid, S: StrategyV1<Monoid = M>> ProtocolV1<M, S>
where
    M::Item: Display + Debug,
{
    fn respond(&mut self, msg: &Message<M>) -> Message<M> {
        let Message(parts) = msg;

        let mut resp_parts = vec![];

        'L: for (range, part) in parts {
            match part {
                MessagePart::Fingerprint(fp) => {
                    let own_fp = self.root.query_range_monoid(range);
                    if &own_fp == fp {
                        break 'L;
                    }

                    let strategy = S::new(range.clone(), own_fp, self.root.clone());
                    resp_parts.extend(strategy);
                }
                MessagePart::ItemSet(items, already_received) => {
                    self.new_items.extend(items.iter().cloned());
                    let own_items = Node::items(Rc::clone(&self.root), range.clone()).collect();

                    if !already_received {
                        resp_parts.push((range.clone(), MessagePart::item_set(own_items, true)))
                    }
                }
            }
        }

        Message(resp_parts)
    }
}

struct FirstCursorBasedStrategy<M: ProtocolMonoid> {
    range: Range<M::Item>,
    fp: M,
    cursor: Cursor<M>,
    is_done: bool,
    prev_range: Option<Range<M::Item>>,
}

/*
Approach for the first strategy: split at around the median, and return item sets if there are less than 5 elemens inside.

*/
impl<'a, M: ProtocolMonoid> Iterator for FirstCursorBasedStrategy<M> {
    type Item = (Range<M::Item>, MessagePart<M>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let mut cur_fp = M::neutral();
        let mut cur_range: Range<M::Item>;

        match &self.prev_range {
            // branch for if this is the first call
            None => match self.range {
                Range::Full | Range::UpTo(_) => {
                    // we are the left-most leaf, so this is the very first element.
                    // this range is exclusive on the right, so we say it is not contained.
                    cur_range = Range::UpTo(self.cursor.current().get_item(0).unwrap());

                    while cur_fp.count() <= self.fp.count() / 2 {
                        // does this if even make sense? shouldn't it always be the case?
                        // or was it intended the other way around?
                        if self.cursor.current_range().is_subrange_of(&self.range) {
                            let node_monoid = self.cursor.current().monoid();
                            cur_fp = cur_fp.combine(node_monoid);
                            match self.cursor.current_range() {
                                Range::Full | Range::StartingFrom(_) => {
                                    // this can only be true for the right flank of the
                                    // tree.
                                    // but we started from the left! This would mean we convered the whole tree!
                                    // this is a weird case. Maybe we should just log here and return an item set for now.
                                    // actually this might be the case for trees with only 1 or 2 elements, when there only is a root node

                                    println!("weird case");
                                    self.is_done = true;
                                    return Some((
                                        Range::Full,
                                        MessagePart::ItemSet(
                                            Node::items(self.cursor.current_rc(), Range::Full)
                                                .collect(),
                                            false,
                                        ),
                                    ));
                                }
                                Range::UpTo(end) | Range::Between(_, end) => {
                                    cur_range = Range::UpTo(end);
                                }
                            };

                            match self.cursor.pop() {
                                Some((child_id, node)) => match child_id {
                                    crate::tree::ChildId::Normal(idx) => todo!(),
                                    crate::tree::ChildId::Last => todo!(),
                                },
                                None => {
                                    // ???
                                }
                            }
                        }
                        let current = self.cursor.current();

                        // add all the
                    }
                }
                Range::StartingFrom(_) => todo!(),
                Range::Between(_, _) => todo!(),
            },
            Some(prev_range) => todo!(),
        }

        todo!()
    }
}

impl<M: ProtocolMonoid> StrategyV1 for FirstCursorBasedStrategy<M> {
    type Monoid = M;

    fn new(range: Range<M::Item>, fp: M, root: Rc<Node<M>>) -> Self {
        let mut cursor = Cursor::new(root);
        match &range {
            Range::Full | Range::UpTo(_) => cursor.find_first(),
            Range::StartingFrom(start) | Range::Between(start, _) => {
                cursor.find(start);
            }
        }
        Self {
            range,
            fp,
            cursor,
            is_done: false,
            prev_range: None,
        }
    }
}

fn respond_to_message<M: ProtocolMonoid>(
    root: Rc<Node<M>>,
    msg: &Message<M>,
) -> (Message<M>, Vec<M::Item>) {
    let current: &Node<M> = &root;
    let mut new_items = vec![];

    let theirs: Vec<(&Range<M::Item>, &M)> = msg
        .0
        .iter()
        .filter_map(|(range, part)| match part {
            MessagePart::Fingerprint(fp) => Some((range, fp)),
            _ => None,
        })
        .collect();

    let response_msg = match current {
        Node::Node2(node_data) => Message(
            respond_to_message_node_data(node_data, Range::Full, &theirs, &mut new_items)
                .into_iter()
                .map(|(_range, message_parts)| message_parts)
                .flatten()
                .collect(),
        ),
        Node::Node3(node_data) => Message(
            respond_to_message_node_data(node_data, Range::Full, &theirs, &mut new_items)
                .into_iter()
                .map(|(_range, message_parts)| message_parts)
                .flatten()
                .collect(),
        ),
        Node::Nil(_) => {
            let out_parts: Vec<(Range<M::Item>, MessagePart<M>)> = msg
                .0
                .iter()
                .map(|(range, msg_part)| match msg_part {
                    MessagePart::Fingerprint(fp) => {
                        Some((range.clone(), MessagePart::<M>::ItemSet(vec![], false)))
                    }
                    MessagePart::ItemSet(items, false) => {
                        new_items.extend_from_slice(&items);
                        Some((range.clone(), MessagePart::ItemSet(vec![], true)))
                    }
                    MessagePart::ItemSet(items, true) => {
                        new_items.extend_from_slice(&items);
                        None
                    }
                })
                .filter_map(|x| x)
                .collect();

            Message(out_parts)
        }
    };

    (response_msg, new_items)
}

fn respond_to_message_node_data<M: ProtocolMonoid, const N: usize>(
    node_data: &NodeData<M, N>,
    node_range: Range<M::Item>,
    theirs: &[(&Range<M::Item>, &M)],
    new_items: &mut Vec<M::Item>,
) -> Vec<(Range<M::Item>, Vec<(Range<M::Item>, MessagePart<M>)>)> {
    let item = &node_data.get_item(0).unwrap();
    let left_node_range = node_range.with_end(item.clone());
    let right_node_range = node_range.with_start(item.clone());

    vec![]
}

fn respond_to_message_inner<M: ProtocolMonoid>(
    node: &Rc<Node<M>>,
    node_range: Range<M::Item>,
    theirs: &[(&Range<M::Item>, &M)],
    new_items: &mut Vec<M::Item>,
) -> Vec<(Range<M::Item>, Vec<(Range<M::Item>, MessagePart<M>)>)> {
    vec![]
}

