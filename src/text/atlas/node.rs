use crate::util::vec2::Vec2;

pub struct Node {
    origin: Vec2<u32>,
    size: Vec2<u32>,
    is_filled: bool,
    children: Option<(Box<Node>, Box<Node>)>,
}

impl Node {
    pub const fn new(origin: Vec2<u32>, size: Vec2<u32>) -> Self {
        Self {
            origin,
            size,
            is_filled: false,
            children: None,
        }
    }

    pub fn pack(&mut self, size: Vec2<u32>, texture_size: u32) -> Option<Vec2<u32>> {
        if self.is_filled {
            return None;
        } else if let Some(children) = self.children.as_mut() {
            children
                .0
                .pack(size, texture_size)
                .or_else(|| children.1.pack(size, texture_size))
        } else {
            let real_size = {
                let mut real_size = self.size;
                if self.origin.x + self.size.x == u32::MAX {
                    real_size.x = texture_size - self.origin.x;
                }
                if self.origin.y + self.size.y == u32::MAX {
                    real_size.y = texture_size - self.origin.y;
                }
                real_size
            };

            if self.size == size {
                self.is_filled = true;
                Some(self.origin)
            } else if real_size.x < size.x || real_size.y < size.y {
                None
            } else {
                let remainder = real_size - size;
                let vertical_split = if remainder == Vec2::new(0, 0) {
                    // If we are going to the edge of the texture, split
                    // according to the glyph dimensions instead
                    self.size.x < self.size.y
                } else {
                    remainder.x < remainder.y
                };

                self.children = Some(if vertical_split {
                    (
                        Box::new(Node::new(self.origin, Vec2::new(self.size.x, size.y))),
                        Box::new(Node::new(
                            Vec2::new(self.origin.x, self.origin.y + size.y),
                            Vec2::new(self.size.x, self.size.y - size.y),
                        )),
                    )
                } else {
                    (
                        Box::new(Node::new(self.origin, Vec2::new(size.x, self.size.y))),
                        Box::new(Node::new(
                            Vec2::new(self.origin.x + size.x, self.origin.y),
                            Vec2::new(self.size.x - size.x, self.size.y),
                        )),
                    )
                });
                self.children.as_mut().unwrap().0.pack(size, texture_size)
            }
        }
    }
}
