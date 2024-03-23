#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right
}

impl Facing {
    pub fn from_deltas(di:isize,dj:isize)->Self {
	if di != 0 {
	    if di > 0 {
		Facing::Down
	    } else {
		Facing::Up
	    }
	} else if dj > 0 {
	    Facing::Right
	} else {
	    Facing::Left
	}
    }

    pub fn to_deltas(self)->(isize,isize) {
	match self {
	    Facing::Up => (-1,0),
	    Facing::Down => (1,0),
	    Facing::Left => (0,-1),
	    Facing::Right => (0,1)
	}
    }
}
