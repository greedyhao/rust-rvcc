#[derive(Debug)]
#[allow(dead_code)]
pub struct BinaryTree<T> {
    key: T,
    left: Link<T>,
    right: Link<T>,
}

pub type Link<T> = Option<Box<BinaryTree<T>>>;

pub trait BinaryTreeIO<T> {
    fn new_binary_without_right(root: T, left: &mut BinaryTree<T>) -> BinaryTree<T>;
}

impl<T> BinaryTree<T> {
    pub fn new(key: T) -> Self {
        BinaryTree {
            key,
            left: None,
            right: None,
        }
    }

    #[allow(dead_code)]
    pub fn get_right(&self) -> Option<&BinaryTree<T>> {
        self.right.as_deref()
    }

    #[allow(dead_code)]
    pub fn get_left(&self) -> Option<&BinaryTree<T>> {
        self.left.as_deref()
    }

    #[allow(dead_code)]
    pub fn set_right(&mut self, right: Link<T>) {
        self.right = right;
    }

    #[allow(dead_code)]
    pub fn set_left(&mut self, left: Link<T>) {
        self.left = left;
    }
}
