use crate::facing::Facing;

#[derive(Copy,Clone,Debug)]
pub enum Position {
    Block(usize,usize,Facing),
    Walking{
	from:(usize,usize),
	to:(usize,usize),
	step:usize,
	total:usize
    }
}

impl Position {
    pub fn facing(self)->Facing {
	match self {
	    Position::Block(_,_,f) => f,
	    Position::Walking{ from:(i0,j0),to:(i1,j1),.. } =>
		Facing::from_deltas(i1 as isize - i0 as isize,j1 as isize - j0 as isize)
	}
    }
}
