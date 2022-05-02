pub struct MoveInfo<Node> {
    pub net: Node,
    pub v: Node,
    pub from_part: u8,
    pub to_part: u8
}

pub struct MoveInfoV<Node> {
    pub v: Node,
    pub from_part: u8,
    pub to_part: u8
    // node_t v;
}
