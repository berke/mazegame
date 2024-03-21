use anyhow::{
    bail,
    Result
};

use serde::{
    Deserialize,
    Serialize
};

#[derive(PartialEq,Copy,Clone,Debug,Serialize,Deserialize)]
pub enum Object {
    Coin,
    Key,
    ToyCar,
    SquaresAndTriangles,
    IceCream,
    Tomato,
    Eggplant,
    Banana,
    Carrot
}

impl Object {
    pub fn is_food(self)->bool {
	match self {
	    Object::IceCream |
	    Object::Carrot |
	    Object::Tomato |
	    Object::Eggplant |
	    Object::Banana => true,
	    _ => false
	}
    }
    pub fn is_consumable(self)->bool {
	match self {
	    Object::IceCream |
	    Object::Carrot |
	    Object::Tomato |
	    Object::Eggplant |
	    Object::Banana |
	    Object::Coin => true,
	    _ => false
	}
    }
    pub fn is_fattening(self)->bool {
	match self {
	    Object::IceCream => true,
	    _ => false
	}
    }
    pub fn name(self)->&'static str {
	match self {
	    Object::Coin => "A COIN",
	    Object::IceCream => "SOME ICE CREAM",
	    Object::Key => "A KEY",
	    Object::ToyCar => "A TOY CAR",
	    Object::SquaresAndTriangles => "SOME SQUARES AND TRIANGLES",
	    Object::Tomato => "A TOMATO",
	    Object::Eggplant => "AN EGGPLANT",
	    Object::Banana => "A BANANA",
	    Object::Carrot => "A CARROT",
	}
    }
    pub fn from_char(c:char)->Result<Self> {
	let obj = 
	    match c {
		'K' => Self::Key,
		'T' => Self::ToyCar,
		'I' => Self::IceCream,
		'C' => Self::Coin,
		'S' => Self::SquaresAndTriangles,
		'c' => Self::Carrot,
		't' => Self::Tomato,
		'e' => Self::Eggplant,
		'b' => Self::Banana,
		_ => bail!("Invalid object {c:?}")
	    };
	Ok(obj)
    }
}
