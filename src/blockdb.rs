use std::cmp::PartialOrd;
use std::fmt::{Debug, Display, Error, Formatter, Write};
use std::ops::{Add, Mul, Sub};

pub struct BlockDb<T, I> {
    root: Option<Box<Node<T, I>>>,
}

#[derive(Debug, Copy, Clone)]
enum Dimension {
    First,
    Second,
    Third,
}

impl Dimension {
    fn next(self) -> Self {
        match self {
            Self::First => Dimension::Second,
            Self::Second => Dimension::Third,
            Self::Third => Dimension::First,
        }
    }
}

impl From<Dimension> for usize {
    fn from(d: Dimension) -> Self {
        match d {
            Dimension::First => 0,
            Dimension::Second => 1,
            Dimension::Third => 2,
        }
    }
}

#[derive(Debug)]
struct Node<T, I> {
    key: [T; 3],
    item: I,
    dim: Dimension,
    right: Option<Box<Node<T, I>>>,
    left: Option<Box<Node<T, I>>>,
}

trait KeyElem:
    Copy + PartialOrd + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
}
impl KeyElem for i16 {}
impl KeyElem for i32 {}
impl KeyElem for i64 {}
impl KeyElem for f32 {}
impl KeyElem for f64 {}

impl<T, I> Display for Node<T, I>
where
    I: Debug,
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "key: ({},{},{}) ", self.key[0], self.key[1], self.key[2])?;
        writeln!(f, "dim: {:?} ", self.dim)?;
        if let Some(l) = &self.left {
            write!(f, " left: \n {}", l)?;
        };
        if let Some(r) = &self.right {
            write!(f, " right: \n {}", r)?;
        }
        Ok(())
    }
}

impl<T, I> Node<T, I>
where
    T: KeyElem,
{
    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
    fn squared_dist(&self, target: &[T; 3]) -> T {
        let k = &self.key;
        let d0 = target[0] - k[0];
        let d1 = target[1] - k[1];
        let d2 = target[2] - k[2];
        return (d0 * d0) + (d1 * d1) + (d2 * d2);
    }
}

impl<T, I> BlockDb<T, I>
where
    T: KeyElem,
{
    pub fn new(items: Vec<I>, keyfn: fn(&I) -> [T; 3]) -> Self {
        let mut nodes: Vec<Box<Node<T, I>>> = Vec::with_capacity(items.len());

        for item in items {
            let n = Node {
                key: keyfn(&item),
                item: item,
                dim: Dimension::First,
                right: None,
                left: None,
            };
            nodes.push(Box::from(n));
        }
        return BlockDb {
            root: Self::build_tree(nodes, Dimension::First),
        };
    }

    fn build_tree(mut nodes: Vec<Box<Node<T, I>>>, dim: Dimension) -> Option<Box<Node<T, I>>> {
        if nodes.len() < 2 {
            return nodes.pop().map(|mut n| {
                n.dim = dim;
                n
            });
        }
        let mut left = nodes;
        let index: usize = dim.into();
        let median = left.len() / 2;
        left.sort_by(|a, b| {
            if b.key[index] < a.key[index] {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
        let right = left.split_off(median);
        let mut curr = left.pop()?;
        curr.left = Self::build_tree(left, dim.next());
        curr.right = Self::build_tree(right, dim.next());
        curr.dim = dim;
        return Some(curr);
    }

    pub fn find_closest_pos(&self, pos: [T; 3]) -> Option<&I> {
        self.root.as_ref().map(|root| &Self::find_closest(root, pos).item)
    }

    fn find_closest(node: &Node<T, I>, pos: [T; 3]) -> &Node<T, I> {
        if node.is_leaf() {
            return node;
        };
        let index = node.dim as usize;
        let is_less = pos[index] < node.key[index];
        let best = if is_less {
            node.left
                .as_ref()
                .map_or(node, |l| Self::find_closest(l, pos))
        } else {
            node.right
                .as_ref()
                .map_or(node, |r| Self::find_closest(r, pos))
        };

        let best = Self::pick_closer_node(&pos, best, node);

        // If best distance intersects the boundary search then the other branch
        let best = if Self::get_dist(node.dim, &node.key, &pos) < best.squared_dist(&pos) {
            let best2 = if !is_less {
                node.left
                    .as_ref()
                    .map_or(node, |l| Self::find_closest(l, pos))
            } else {
                node.right
                    .as_ref()
                    .map_or(node, |r| Self::find_closest(r, pos))
            };
            Self::pick_closer_node(&pos, best, best2)
        } else {
            best
        };

        Self::pick_closer_node(&pos, best, node)
    }

    fn pick_closer_node<'a>(
        pos: &[T; 3],
        n1: &'a Node<T, I>,
        n2: &'a Node<T, I>,
    ) -> &'a Node<T, I> {
        if n1.squared_dist(pos) < n2.squared_dist(pos) {
            n1
        } else {
            n2
        }
    }

    fn get_dist(dim: Dimension, k1: &[T; 3], k2: &[T; 3]) -> T {
        let n1 = k1[dim as usize];
        let n2 = k2[dim as usize];
        if n1 > n2 {
            n1 - n2
        } else {
            n2 - n1
        }
    }
}

impl<T, I> BlockDb<T, I>
where
    T: Display,
{
    pub fn to_dot_str(&self) -> String {
        let mut out = String::new();
        out.push_str("graph rtree {\n");
        if let Some(root) = &self.root {
            Self::to_dot(&root, &mut out, 0);
        }
        out.push_str("}");
        return out;
    }

    fn to_dot(node: &Node<T, I>, w: &mut dyn Write, id: u64) -> u64 {
        writeln!(
            w,
            "{} [label=\"{}@({},{},{})\"]",
            id, node.dim as usize, node.key[0], node.key[1], node.key[2]
        )
        .unwrap();
        let mut next_id = id + 1;
        if let Some(l) = &node.left {
            writeln!(w, "{} -- {} [label=\"left\"]", id, next_id).unwrap();
            next_id = Self::to_dot(l, w, next_id);
        }
        if let Some(r) = &node.right {
            writeln!(w, "{} -- {} [label=\"right\"]", id, next_id).unwrap();
            next_id = Self::to_dot(r, w, next_id);
        }
        return next_id;
    }
}

#[test]
fn test_r_tree() {
    let coords: Vec<(i64, i64, i64)> = vec![
        (1, 1, 0),
        (1, 3, 0),
        (1, 6, 0),
        (3, 1, 0),
        (6, 1, 0),
        (3, 1, 1),
        (3, 1, 4),
    ];
    let bdb = BlockDb::new(coords, |x| [x.0, x.1, x.2]);
    assert_eq!(
        (3, 1, 0),
        bdb.find_closest_pos([4, 1, 0])
            .cloned()
            .unwrap_or((0, 0, 0))
    );
    assert_eq!(
        (1, 1, 0),
        bdb.find_closest_pos([1, 1, 0])
            .cloned()
            .unwrap_or((0, 0, 0))
    );
    assert_eq!(
        (6, 1, 0),
        bdb.find_closest_pos([10, 1, 0])
            .cloned()
            .unwrap_or((0, 0, 0))
    );
    assert_eq!(
        (3, 1, 0),
        bdb.find_closest_pos([3, 1, -1])
            .cloned()
            .unwrap_or((0, 0, 0))
    );
    assert_eq!(
        (3, 1, 4),
        bdb.find_closest_pos([3, 1, 3])
            .cloned()
            .unwrap_or((0, 0, 0))
    );
}
