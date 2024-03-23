use std::fmt::{
    Display,
    Formatter
};

use serde::{
    Deserialize,
    Serialize
};

use crate::{
    object::Object
};

#[derive(PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub struct Random {
    pub i:u32,
}

impl Random {
    pub fn new(i:u32)->Self {
	Random{ i }
    }
}

#[derive(PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub struct Periodic {
    pub i:u16,
    pub m:u16,
    pub j:u16,
    pub n:u16
}

impl Periodic {
    pub fn new(m:u16,n:u16)->Self {
	Periodic{ i:0,m,j:0,n }
    }

    pub fn next(&mut self) {
	self.j += 1;
	if self.j == self.n {
	    self.j = 0;
	    self.i += 1;
	    if self.i == self.m {
		self.i = 0;
	    }
	}
    }
}

#[derive(PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub struct Target {
    pub room:usize,
    pub door:usize
}

#[derive(PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub struct Door {
    pub id:usize,
    pub target:Option<Target>,
    pub key:Option<Object>,
    pub locked:bool
}

#[derive(PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub enum Corner {
    NE,
    SE,
    NW,
    SW
}

#[derive(PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub enum Tile {
    Empty,
    Brick,
    Rainbow,
    Object(Object),
    Vortex,
    Grass,
    Dirt,
    PyramidStone,
    Window,
    Water(Periodic),
    Fire(Periodic),
    Door(Door),
    Metal,
    Alien,
    MetalRamp(Corner),
    MetalFoot,
    Sky(Random)
}

impl Tile {
    pub fn next(&mut self) {
	match self {
	    Tile::Fire(p) | Tile::Water(p) => p.next(),
	    _ => ()
	}
    }
}

impl Display for Tile {
    fn fmt(&self,f:&mut Formatter<'_>)->std::result::Result<(),std::fmt::Error> {
	match self {
	    Tile::Empty => write!(f,"Nothing")?,
	    Tile::Brick => write!(f,"A brick")?,
	    Tile::Rainbow => write!(f,"A rainbow")?,
	    Tile::Object(o) => write!(f,"{}",o.name())?,
	    Tile::Vortex => write!(f,"A vortex")?,
	    Tile::Grass => write!(f,"Grass")?,
	    Tile::Dirt => write!(f,"Dirt")?,
	    Tile::PyramidStone => write!(f,"A Pyramid stone")?,
	    Tile::Window => write!(f,"A window")?,
	    Tile::Water(_) => write!(f,"Water")?,
	    Tile::Fire(_) => write!(f,"Fire")?,
	    Tile::Door(d) => {
		write!(f,"Door number {}.",d.id)?;
		match d.key {
		    None => (),
		    Some(k) => {
			if d.locked {
			    write!(f,"  It's locked, needs {} to open.",
				   k.name())?;
			} else {
			    write!(f,"  It was unlocked with {}.",
				   k.name())?;
			}
		    }
		}
		match d.target {
		    None => write!(f,"  This door goes nowhere!")?,
		    Some(Target { room,door }) =>
			write!(f,"  Goes to door {} of room {}",
			       door,room)?
		}
	    },
	    Tile::Metal => write!(f,"Metal")?,
	    Tile::Alien => write!(f,"An alien")?,
	    Tile::MetalRamp(_) => write!(f,"A metal ramp")?,
	    Tile::MetalFoot => write!(f,"A metal foot")?,
	    Tile::Sky(_) => write!(f,"The sky")?,
	}
	Ok(())
    }
}
