use std::rc::Rc;

pub trait Monoid: std::fmt::Debug + PartialEq + Eq + Clone {
    type Item;

    fn zero() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn op(&self, other: &Self) -> Self;
}

pub trait Key: std::fmt::Debug + PartialEq + Eq +PartialOrd + Ord +Clone {}

pub struct Tree<K, M>(Rc<Node<K, M>>)
where
    K: Key,
    M: Monoid;

impl<K: Key, M: Monoid> Clone for Tree<K, M> {
  fn clone(&self) -> Self {
    let Tree(rc_node) = self;
    Tree(rc_node.clone())
  }
}

impl<K: Key + Into<M>, M: Monoid> Tree<K, M> {
    pub fn empty() -> Tree<K, M> {
        let root = Node::nil();
        Tree(Rc::new(root))
    }

    pub fn cursor(&self) -> Cursor<K, M> {
        Cursor::new(self.0.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node<K:Key, M: Monoid> {
    Nil(M),
    Node2(Node2<K, M>),
    Node3(Node3<K, M>),
}

enum InsertResult<K: Key, M: Monoid> {
    NoSplit(Node<K, M>),
    Split(Node<K, M>, Node<K, M>, K)
}

impl<K: Key, M: Monoid> Node<K, M> {
  fn insert(&self, new_elem: K) -> Option<InsertResult<K, M>> {
    match self {
      Node::Node2(Node2{ value, left, right, .. }) => {
          // new_elem is already in the tree, don't do anything
          if &new_elem == value.key() {
            return None
          }

          // we are at a leaf!
          //   -> insert here
          //   NB: the tree is balanced, so both left and right are nil
          // otherwise recurse
          if left.is_nil() {
            
            // sort the values
            let new_pair = Pair::from_key(new_elem);
            let (value1, value2) = if new_pair < value {
              (new_pair, value)
            } else {
              (value, new_pair)
            };

            // a pointer to a nil element is a pointer to a nil element is a pointer to a nil element
            let middle = left.clone();

            let total = new_pair.monoid().op(&value.monoid());
            
            Some((
              Node::Node3(Node3{ value1, value2, left, right, middle, total}),
              Some(None)
            ))
          } else {
            if new_elem < value {
              self.try_merge(left.insert(new_elem)?, Source::Left)
            } else {
              self.try_merge(right.insert(new_elem)?, Source::Right)
            }
          }
      },
      Node::Node3(Node3{ value1, value2, left, middle, right, .. }) => {
        if new_elem == value1 || new_elem == value2 {
          return None
        }
      },
      _=> panic!(),
    }
  }

  fn try_merge(&self, insert_result: InsertResult<K, M>, src: Source) -> InsertResult<K, M> {
      match (self, insert_result) {
          (node, InsertResult::NoSplit(new_child)) => InsertResult::NoSplit(node.update_child(src, node))
          (Node::Node2(node), InsertResult::Split(new_child1, new_child2, value)  => {
              match (self, src) {
                  Node::Node2(node) => InsertResult::NoSplit(Node::Node3
              }

          }
          _ => {}
      }
  }

  fn update_child(&self, child_direction: Source, new_child: Node<K, M>) -> Node<K, M> {
      match (self, child_direction) {
          (Node::Node2(node), Source::Left) => {
            let total = node.right.monoid().op(node.value.monoid()).op(new_child.monoid());
            Node::Node2(Node2{
              left: Rc::new(new_child),
              total,
              ..node
            })
          },
          (Node::Node3(node), Source::Left) => {
            let total = node.right.monoid().op(node.middle.monoid()).op(node.value1.monoid()).op(node.value2.monoid()).op(new_child.monoid());
            Node::Node3(Node3{
              left: Rc::new(new_child),
              total,
              ..node
            })
          }
          (Node::Node2(node), Source::Right) => {
            let total = node.left.monoid().op(node.value.monoid()).op(new_child.monoid());
            Node::Node2(Node2{
              right: Rc::new(new_child),
              total,
              ..node
            })
          },
          (Node::Node3(node), Source::Right) => {
            let total = node.left.monoid().op(node.middle.monoid()).op(node.value1.monoid()).op(node.value2.monoid()).op(new_child.monoid());
            Node::Node3(Node3{
              right: Rc::new(new_child),
              total,
              ..node
            })
          },
          (Node::Node3(node), Source::Middle) => {
            let total = node.left.monoid().op(node.right.monoid()).op(node.value1.monoid()).op(node.value2.monoid()).op(new_child.monoid());
            Node::Node3(Node3{
              middle: Rc::new(new_child),
              total,
              ..node
            })
          },
          _ => panic!(),
      }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node2<K: Key, M: Monoid> {
    left: Rc<Node<K, M>>,
    right: Rc<Node<K, M>>,
    value: Pair<K, M>,
    total: M,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node3<K: Key, M: Monoid> {
    left: Rc<Node<K, M>>,
    middle: Rc<Node<K, M>>,
    right: Rc<Node<K, M>>,
    value1: Pair<K, M>,
    value2: Pair<K, M>,
    total: M,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair<K, M>(K, M)
where
    K: Key,
    M: Monoid;

impl<K: Key, M: Monoid> Node<K, M> {
    pub fn nil() -> Node<K, M> {
        Node::Nil(M::zero())
    }

    fn node2_with_total(
        value: Pair<K, M>,
        left: Rc<Node<K, M>>,
        right: Rc<Node<K, M>>,
        total: M,
    ) -> Node<K, M> {
        Node::Node2(Node2 {
            left,
            right,
            value,
            total,
        })
    }

    pub fn node2(value: Pair<K, M>, left: Rc<Node<K, M>>, right: Rc<Node<K, M>>) -> Node<K, M> {
        let total = left.monoid().op(value.monoid()).op(right.monoid());
        Node::node2_with_total(value, left, right, total)
    }

    fn node3_with_total(
        value1: Pair<K, M>,
        value2: Pair<K, M>,
        left: Rc<Node<K, M>>,
        middle: Rc<Node<K, M>>,
        right: Rc<Node<K, M>>,
        total: M,
    ) -> Node<K, M> {
        Node::Node3(Node3 {
            left,
            middle,
            right,
            value1,
            value2,
            total,
        })
    }

    pub fn node3(
        value1: Pair<K, M>,
        value2: Pair<K, M>,
        left: Rc<Node<K, M>>,
        middle: Rc<Node<K, M>>,
        right: Rc<Node<K, M>>,
    ) -> Node<K, M> {
        let total = left
            .monoid()
            .op(&value1.monoid())
            .op(&middle.monoid())
            .op(&value2.monoid())
            .op(&right.monoid());
        Node::node3_with_total(value1, value2, left, middle, right, total)
    }

    fn is_nil(&self) -> bool {
        if let Node::Nil(_) = self {
            true
        } else {
            false
        }
    }

    fn monoid(&self) -> &M {
        match self {
            Node::Nil(m) => m,
            Node::Node2(Node2 { total, .. }) => total,
            Node::Node3(Node3 { total, .. }) => total,
        }
    }
}

use core::cmp::Ordering;

impl<K: Key, M: Monoid> PartialOrd for Pair<K, M> {
   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      self.key().partial_cmp(other.key())
   }
}

impl<K: Key, M: Monoid> Ord for Pair<K, M> {
   fn cmp(&self, other: &Self) -> Ordering {
      self.key().cmp(other.key())
   }
}

impl<K: Key, M: Monoid> Pair<K, M> {
    fn from_key(k: K) -> Self {
      let m = M::lift(&k);
      Pair(k, m)
    }

    fn monoid(&self) -> &M {
        let Pair(_, m) = self;
        m
    }

    fn key(&self) -> &K {
        let Pair(k, _) = self;
        k
    }

}

///////////

enum ImmutableCursor<K: Key, M: Monoid> {
  Root(Rc<Node<K, M>>),
  Left(Rc<ImmutableCursor<K, M>>, Rc<Node<K, M>>),
  Middle(Rc<ImmutableCursor<K, M>>, Rc<Node<K, M>>),
  Right(Rc<ImmutableCursor<K, M>>, Rc<Node<K, M>>),
}

impl<K: Key, M: Monoid> ImmutableCursor<K, M> {
  pub fn new(node: Rc<Node<K, M>>) -> ImmutableCursor<K, M> {
    ImmutableCursor::Root(node)
  }

  fn push_left(parent: Rc<Self>, node: Rc<Node<K, M>>) -> ImmutableCursor<K, M> {
      #[cfg(debug_assertions)]
      {
          let parent_cursor: &ImmutableCursor<K, M> = &parent;
          let parent_node = parent_cursor.current();
          match parent_node {
              Node::Nil(_) => panic!(),
              Node::Node2(Node2 { left, .. }) => assert_eq!(left, &node),
              Node::Node3(Node3 { left, ..  }) => assert_eq!(left, &node),
          }
      }
      ImmutableCursor::Left(parent, node)
  }

  fn push_middle(parent: Rc<Self>, node: Rc<Node<K, M>>) -> ImmutableCursor<K, M> {
      #[cfg(debug_assertions)]
      {
          let parent_cursor: &ImmutableCursor<K, M> = &parent;
          let parent_node = parent_cursor.current();
          match parent_node {
              _ => panic!(),
              Node::Node3(Node3 { middle, ..  }) => assert_eq!(middle, &node),
          }
      }
      ImmutableCursor::Middle(parent, node)
  }

  fn push_right(parent: Rc<Self>, node: Rc<Node<K, M>>) -> ImmutableCursor<K, M> {
      #[cfg(debug_assertions)]
      {
          let parent_cursor: &ImmutableCursor<K, M> = &parent;
          let parent_node = parent_cursor.current();
          match parent_node {
              Node::Nil(_) => panic!(),
              Node::Node2(Node2 { right, .. }) => assert_eq!(right, &node),
              Node::Node3(Node3 { right, ..  }) => assert_eq!(right, &node),
          }
      }
      ImmutableCursor::Right(parent, node)
  }

  fn current(&self) -> &Node<K, M> {
    match self {
      ImmutableCursor::Root(node) | ImmutableCursor::Left(_, node) | ImmutableCursor::Middle(_, node) | ImmutableCursor::Right(_, node) => node
    }
  }

  fn pop(&self) -> Option<(&Self, &Node<K, M>)> {
    match self {
      ImmutableCursor::Root(node) => None,
      ImmutableCursor::Left(parent, node) | ImmutableCursor::Middle(parent, node) | ImmutableCursor::Right(parent, node) => Some((&parent, node))
    }
  }

}

enum Source {
    Root,
    Left,
    Middle,
    Right,
}

struct PathElem<K:Key, M: Monoid> {
    src: Source,
    node: Rc<Node<K, M>>,
}

pub struct Cursor<K:Key, M: Monoid> {
    path: Vec<PathElem<K, M>>,
}

impl<K: Key, M: Monoid> Cursor<K, M> {
    pub fn new(node: Rc<Node<K, M>>) -> Cursor<K, M> {
        Cursor {
            path: vec![PathElem {
                src: Source::Root,
                node: node.clone(),
            }],
        }
    }

    fn current(&self) -> &Node<K, M> {
        let PathElem { node, .. } = &self.path.first().unwrap();
        &node
    }

    fn pop(&mut self) -> PathElem<K,M> {
      self.path.pop().expect("cursor is not expected to be empty")
    }

    fn push_path_elem(&mut self, path_elem: PathElem<K, M>) {
        let PathElem { src, node } = &path_elem;
        let node = node.clone();
        //let node: &Node<K, M> = &node;
        #[cfg(debug_assertions)]
        {
            if let Some(PathElem {
                node: last_node_rc, ..
            }) = &self.path.last()
            {
                let last_node: &Node<_, _> = last_node_rc.as_ref();
                match &last_node {
                    Node::Nil(_) => panic!(),
                    Node::Node2(Node2 { left, right, .. }) => match &src {
                        Source::Left => assert_eq!(left, &node),
                        Source::Right => assert_eq!(right, &node),
                        _ => panic!(),
                    },
                    Node::Node3(Node3 {
                        left,
                        middle,
                        right,
                        ..
                    }) => match &src {
                        Source::Left => assert_eq!(left, &node),
                        Source::Middle => assert_eq!(middle, &node),
                        Source::Right => assert_eq!(right, &node),
                        _ => panic!(),
                    },
                }
            } else {
                panic!()
            }
        }

        self.path.push(path_elem)
    }

    fn push(&mut self, src: Source, node: Rc<Node<K, M>>) {
        self.push_path_elem(PathElem { src, node })
    }

    fn descend<T, F: FnMut(&Node<K, M>) -> Direction<T>>(&mut self, mut f: F) -> T {
        loop {
            let current: &Node<K, M> = &self.current();
            let next = match f(current) {
                Direction::Left => match current {
                    Node::Node2(Node2 { left, .. }) | Node::Node3(Node3 { left, .. }) => PathElem {
                        src: Source::Left,
                        node: left.clone(),
                    },
                    Node::Nil(_) => panic!(),
                },
                Direction::Right => match current {
                    Node::Node2(Node2 { right, .. }) | Node::Node3(Node3 { right, .. }) => {
                        PathElem {
                            src: Source::Right,
                            node: right.clone(),
                        }
                    }
                    Node::Nil(_) => panic!(),
                },
                Direction::Middle => match current {
                    Node::Node3(Node3 { middle, .. }) => PathElem {
                        src: Source::Middle,
                        node: middle.clone(),
                    },
                    Node::Node2(_) | Node::Nil(_) => panic!(),
                },
                Direction::Done(info) => return info,
            };

            self.push_path_elem(next)
        }
    }
}

enum Direction<T> {
    Left,
    Middle,
    Right,
    Done(T),
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
