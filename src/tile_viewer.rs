use crate::{
    common::*,
    a2::A2,
    tiles::{
	Corner,
	Door,
	Tile,
	Random
    },
    object::Object,
    world::World,
    ptr::*,
    room::Room
};

#[derive(Copy,Clone,PartialEq)]
pub enum Tool {
    Place(Tile)
}

pub struct TileViewer {
    img:Option<load::TexturePoll>,
    ny:isize,
    nx:isize,
    tile_size:Vec2,
    room:Option<Ptr<Room>>,
    rainbow_index:usize,
    selection:Option<(isize,isize)>,
    tool:Tool
}

#[derive(Copy,Clone)]
enum TileAspect {
    FromImage(Vec2,Vec2),
    Solid(Color32)
}

impl TileViewer {
    const RAINBOW : &'static [Color32] = &[
	Color32::from_rgb(255,  0,  0),
	Color32::from_rgb(255,255,  0),
	Color32::from_rgb(  0,255,  0),
	Color32::from_rgb(  0,255,255),
	Color32::from_rgb(  0,  0,255),
	Color32::from_rgb(255,  0,255),
    ];

    pub fn set_room(&mut self,room:Option<Ptr<Room>>) {
	self.room = room;
    }

    pub fn room(&self)->Option<Ptr<Room>> {
	if let Some(p) = &self.room {
	    Some(Ptr::clone(p))
	} else {
	    None
	}
    }

    pub fn set_tool(&mut self,tool:Tool) {
	self.tool = tool;
    }

    pub fn get_tool_mut(&mut self)->&mut Tool {
	&mut self.tool
    }

    pub fn new()->Self {
	let img = None;
	let tile_size = vec2(32.0,32.0);
	let map = A2::new((16,16),Tile::Empty);
	let ny = 24;
	let nx = 48;
	Self { img,tile_size,
	       rainbow_index:0,
	       room:None,
	       ny,
	       nx,
	       selection:None,
	       tool:Tool::Place(Tile::Empty) }
    }

    fn find_tile(&self,tl:Tile)->TileAspect {
	let (tw,th) = (16,16);
	let vec = |u,v| vec2((v*tw) as f32,(u*th) as f32);
	let tile = |u,v| {
	    let p0 = vec(u,v);
	    let p1 = vec(u+1,v+1);
	    TileAspect::FromImage(p0,p1)
	};
	let fill = |c| TileAspect::Solid(c);
	match tl {
	    Tile::Fire(p) => tile(4+p.i,4),
	    Tile::Brick => tile(0,0),
	    Tile::Metal => tile(0,12),
	    Tile::Alien => tile(0,14),
	    Tile::Sky(Random{ i }) => tile(i as u16,10),
	    Tile::MetalRamp(Corner::NW) => tile(2,13),
	    Tile::MetalRamp(Corner::NE) => tile(2,12),
	    Tile::MetalRamp(Corner::SW) => tile(1,12),
	    Tile::MetalRamp(Corner::SE) => tile(1,13),
	    Tile::MetalFoot => tile(3,12),
	    Tile::Object(o) => match o {
		Object::Coin => tile(0,1),
		Object::IceCream => tile(0,3),
		Object::Key => tile(0,4),
		Object::ToyCar => tile(0,5),
		Object::SquaresAndTriangles => tile(0,6),
		Object::Tomato => tile(1,5),
		Object::Eggplant => tile(1,6),
		Object::Banana => tile(1,7),
		Object::Carrot => tile(1,8)
	    },
	    Tile::Door(Door{ key:None, .. }) => tile(0,2),
	    Tile::Door(Door{ key:Some(_), locked:true, ..}) => tile(1,3),
	    Tile::Door(Door{ key:Some(_), locked:false, ..}) => tile(1,4),
	    Tile::Vortex => tile(0,7),
	    Tile::Grass => tile(0,8),
	    Tile::Dirt => tile(2,8),
	    Tile::PyramidStone => tile(3,8),
	    Tile::Window => tile(0,11),
	    Tile::Water(p) => tile(p.i,9),
	    Tile::Empty => fill(Color32::BLACK),
	    Tile::Rainbow => fill(Self::RAINBOW[self.rainbow_index]),
	}
    }

    pub fn do_ui(&mut self,ui:&mut Ui)->Response {
	let (ny,nx) = (self.ny,self.nx);
	let desired_size = vec2(nx as f32,ny as f32)*self.tile_size;
	let (rect,mut response) =
	    ui.allocate_exact_size(desired_size,
				   Sense::click_and_drag());

	if ui.is_rect_visible(rect) {
	    ui.painter().rect(
		rect,
		0.0,
		Color32::DARK_GREEN,
		Stroke::NONE
	    );

	    if let Some(room_ptr) = &self.room {
		let mut room = room_ptr.yank_mut();
		// ui.text_edit_singleline(&mut room.name);
		let mut map = room.map_mut();
		let (ny,nx) = map.dims();

		self.img.get_or_insert_with(|| {
		    include_image!("../gfx/tiles.png")
			.load(
			    ui.ctx(),
			    TextureOptions::NEAREST_MIRRORED_REPEAT,
			    load::SizeHint::Size(320,320))
			.expect("Can't load image")
		});

		let p0 = rect.left_top();

		if response.is_pointer_button_down_on() {
		    ui.input(|input| {
			if let Some(p) = response.interact_pointer_pos() {
			    let r = (p - p0) / self.tile_size;
			    let iy = r[1].floor() as isize;
			    let ix = r[0].floor() as isize;
			    if 0 <= iy && iy < ny && 0 <= ix && ix < nx {
				if input.pointer
				    .button_down(PointerButton::Primary) {
					match self.tool {
					    Tool::Place(tile) =>
						map[[iy,ix]] = tile
					}
				    } else if input.pointer
				    .button_pressed(PointerButton::Secondary) {
					let new_sel = Some((iy,ix));
					if self.selection == new_sel {
					    self.selection = None;
					} else {
					    self.selection = new_sel;
					}
				    }
			    }
			}
		    })
		};

		for iy in 0..ny {
		    for ix in 0..nx {
			let p1 = p0 + vec2(ix as f32,iy as f32)*self.tile_size;
			let p2 = p1 + self.tile_size;
			let rect = Rect::from_points(&[p1,p2]);
			match self.find_tile(map[[iy,ix]]) {
			    TileAspect::Solid(color) => {
				ui.painter().rect(
				    Rect::from_points(&[p1,p2]),
				    0.0,
				    color,
				    Stroke::NONE
				);
			    },
			    TileAspect::FromImage(q0,q1) => {
				if let Some(TexturePoll::Ready { texture })
				    = self.img {
					let ts = texture.size;
					let u0 = q0/ts;
					let u1 = q1/ts;
					let uv = Rect::from_points(
					    &[u0.to_pos2(),u1.to_pos2()]);
					ui.painter().image(
					    texture.id,
					    rect,
					    uv,
					    Color32::WHITE
					);
				    }
			    }
			}
			if Some((iy,ix)) == self.selection {
			    ui.painter().rect_stroke(
				Rect::from_points(&[p1,p2]),
				0.0,
				Stroke::new(2.0,Color32::GREEN)
			    );
			}
		    }
		}
	    }
	}
	response
    }
}

impl Widget for &mut TileViewer {
    fn ui(self,ui:&mut Ui)->Response {
	self.do_ui(ui)
    }
}
