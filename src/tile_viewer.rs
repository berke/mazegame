use std::collections::{
    BTreeMap,
    VecDeque
};

use crate::{
    common::*,
    a2::A2,
    tiles::{
	Corner,
	Door,
	Target,
	Tile,
	Random
    },
    object::Object,
    world::{
	World,
	TileAddress
    },
    mini_rng::MiniRNG,
    ptr::*,
    refresher::Refresher,
    room::Room
};

#[derive(Copy,Clone,PartialEq)]
pub enum Tool {
    Nothing,
    Place(Tile),
    PlaceSky
}

#[derive(Copy,Clone,Debug)]
struct Edit {
    iy:usize,
    ix:usize,
    old:Tile,
    new:Tile,
}

struct Undo {
    undo:VecDeque<Edit>,
    redo:VecDeque<Edit>,
    limit:usize
}

impl Undo {
    pub fn new()->Self {
	Self {
	    undo:VecDeque::new(),
	    redo:VecDeque::new(),
	    limit:50
	}
    }

    pub fn edit(&mut self,edit:Edit) {
	self.undo.push_back(edit);
	if self.undo.len() > self.limit {
	    self.undo.pop_front();
	}
    }

    pub fn undo(&mut self)->Option<Edit> {
	if let Some(edit) = self.undo.pop_back() {
	    self.redo.push_back(edit);
	    if self.redo.len() > self.limit {
		self.redo.pop_front();
	    }
	    Some(edit)
	} else {
	    None
	}
    }

    pub fn redo(&mut self)->Option<Edit> {
	if let Some(edit) = self.redo.pop_back() {
	    self.undo.push_back(edit);
	    if self.undo.len() > self.limit {
		self.undo.pop_front();
	    }
	    Some(edit)
	} else {
	    None
	}
    }
}

pub struct TileViewer {
    img:Option<load::TexturePoll>,
    ny:isize,
    nx:isize,
    tile_size:Vec2,
    room:Option<Ptr<Room>>,
    rainbow_index:usize,
    selection1:Option<TileAddress>,
    selection2:Option<TileAddress>,
    tool:Tool,
    info:String,
    goto:Option<usize>,
    hover:Option<(usize,usize)>,
    last_edit:Option<(usize,usize)>,
    refresher:Refresher,
    rng:MiniRNG,
    undos:BTreeMap<usize,Undo>
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
	self.hover = None;
	self.last_edit = None;
	self.info.clear();
    }

    pub fn room(&self)->Option<Ptr<Room>> {
	self.room.as_ref().map(Ptr::refer)
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
	let ny = 48;
	let nx = 48;
	Self { img,tile_size,
	       rainbow_index:0,
	       room:None,
	       ny,
	       nx,
	       selection1:None,
	       selection2:None,
	       tool:Tool::Nothing,
	       info:String::new(),
	       goto:None,
	       hover:None,
	       last_edit:None,
	       refresher:Refresher::new(0.05),
	       rng:MiniRNG::new(1),
	       undos:BTreeMap::new()
	}
    }

    pub fn selection1(&self)->Option<TileAddress> {
	self.selection1
    }

    pub fn selection2(&self)->Option<TileAddress> {
	self.selection2
    }

    fn tile_rect(&self,p0:Pos2,iy:usize,ix:usize,enlarge:f32)->Rect {
	let p1 = p0 + vec2(ix as f32,iy as f32)*self.tile_size;
	let p2 = p1 + self.tile_size;

	Rect::from_points(&[
	    p1 - enlarge*vec2(1.0,1.0),
	    p2 + enlarge*vec2(1.0,1.0)])
    }
    
    fn find_tile(&self,tl:Tile)->TileAspect {
	let (tw,th) = (16,16);
	let vec = |u,v| vec2((v*tw) as f32,(u*th) as f32);
	let tile = |u,v| {
	    let p0 = vec(u,v);
	    let p1 = vec(u+1,v+1);
	    TileAspect::FromImage(p0,p1)
	};
	let fill = TileAspect::Solid;
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
	    Tile::Door(Door{ target:None, .. }) => tile(2,3),
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

    fn which_tile(&self,p0:Pos2,p:Pos2)->Option<(usize,usize)> {
	let r = (p - p0) / self.tile_size;
	let iy = r[1].floor() as isize;
	let ix = r[0].floor() as isize;
	if 0 <= iy && iy < self.ny && 0 <= ix && ix < self.nx {
	    Some((iy as usize,ix as usize))
	} else {
	    None
	}
    }
    
    pub fn ui(&mut self,ui:&mut Ui) {
	ui.label(&self.info);
	ui.separator();
	ScrollArea::both()
	    .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
	    .max_height(600.0)
	    .show(ui,|ui| ui.add(self)); // self.ui(ui));
    }

    fn info(&mut self,u:&str) {
	self.info.clear();
	self.info.push_str(u);
    }

    pub fn take_goto(&mut self)->Option<usize> {
	self.goto.take()
    }

    fn edit(&mut self,room:&mut RefMut<'_,Room>,iy:usize,ix:usize,tile:Tile) {
	let undo = self.undos.entry(room.id).or_insert_with(Undo::new);
	let old = room.map()[[iy,ix]];
	let new = room.modify(iy,ix,tile);
	let edit = Edit { iy,ix,old,new };
	undo.edit(edit);
    }

    pub fn undo(&mut self) {
	if let Some(room_ptr) = self.room.as_ref().map(|p| p.refer()) {
	    let mut room = room_ptr.yank_mut();
	    let undo = self.undos.entry(room.id).or_insert_with(Undo::new);
	    if let Some(edit) = undo.undo() {
		room.modify(edit.iy,edit.ix,edit.old);
	    }
	} else {
	    self.info("No room");
	}
    }

    pub fn redo(&mut self) {
	if let Some(room_ptr) = self.room.as_ref().map(|p| p.refer()) {
	    let mut room = room_ptr.yank_mut();
	    let undo = self.undos.entry(room.id).or_insert_with(Undo::new);
	    if let Some(edit) = undo.redo() {
		room.modify(edit.iy,edit.ix,edit.new);
	    }
	} else {
	    self.info("No room");
	}
    }
    
    pub fn do_ui(&mut self,ui:&mut Ui)->Response {
	let (ny,nx) = (self.ny,self.nx);
	let desired_size = vec2(nx as f32,ny as f32)*self.tile_size;
	let (rect,response) =
	    ui.allocate_exact_size(desired_size,
				   Sense::click_and_drag());

	let mut hover = None;
	if ui.is_rect_visible(rect) {
	    ui.painter().rect(
		rect,
		0.0,
		Color32::DARK_GREEN,
		Stroke::NONE
	    );

	    if let Some(room_ptr) = self.room.as_ref().map(|p| p.refer()) {
		let mut room = room_ptr.yank_mut();

		if self.refresher.tick(ui) {
		    room.next();
		    self.rainbow_index = (self.rainbow_index + 1).rem_euclid(Self::RAINBOW.len());
		}

		let room_id = room.id;
		// let map = room.map();
		let (ny,nx) = room.map().dims();

		self.img.get_or_insert_with(|| {
		    include_image!("../gfx/tiles.png")
			.load(
			    ui.ctx(),
			    TextureOptions::NEAREST_MIRRORED_REPEAT,
			    load::SizeHint::Size(320,320))
			.expect("Can't load image")
		});

		let p0 = rect.left_top();

		if let Some(p) = response.hover_pos() {
		    hover = self.which_tile(p0,p);
		}

		if let Some((iy,ix)) = hover {
		    if hover != self.hover {
			let mut info = String::new();
			write!(info,"({:02},{:02}) {}",iy,ix,room.map()[[iy,ix]]).unwrap();
			self.info = info;
			self.hover = hover;
		    }
		}

		if response.is_pointer_button_down_on() {
		    if let Some(p) = response.interact_pointer_pos() {
			if let Some((iy,ix)) = self.which_tile(p0,p) {
			    ui.input(|input| {
				if input.pointer
				    .button_down(PointerButton::Primary) {
					if Some((iy,ix)) != self.last_edit {
					    match self.tool {
						Tool::Nothing => {
						    if let Tile::Door(
							Door { target:Some(Target { room, .. }),
							       .. }) = room.map()[[iy,ix]] {
							self.goto = Some(room);
						    }
						},
						Tool::Place(tile) => {
						    self.edit(&mut room,iy,ix,tile);
						    // room.modify(iy,ix,tile);
						    self.last_edit = Some((iy,ix));
						    self.hover = None;
						},
						Tool::PlaceSky => {
						    let tile = Tile::Sky(
							Random { i:self.rng.sample_u32(20) });
						    // room.modify(iy,ix,tile);
						    self.edit(&mut room,iy,ix,tile);
						    self.last_edit = Some((iy,ix));
						    self.hover = None;
						}
					    }
					}
				    } else {
					self.last_edit = None;
					if input.pointer.button_pressed(PointerButton::Secondary) {
					    let sel =
						if input.modifiers.shift {
						    &mut self.selection2
						} else {
						    &mut self.selection1
						};
					    let new_sel = Some(TileAddress { room_id,iy,ix });
					    if *sel == new_sel {
						*sel = None;
					    } else {
						*sel = new_sel;
					    }
					}
				    }
			    });
			}
		    }
		} else {
		    self.last_edit = None;
		}

		{
		    let map = room.map();
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
			}
		    }
		}

		match self.selection1 {
		    Some(TileAddress { iy,ix,room_id:id }) if id == room_id => {
			ui.painter().rect_stroke(
			    self.tile_rect(p0,iy,ix,1.0),
			    0.0,
			    Stroke::new(2.0,Color32::GREEN));
		    },
		    _ => ()
		}

		match self.selection2 {
		    Some(TileAddress { iy,ix,room_id:id }) if id == room_id => {
			ui.painter().rect_stroke(
			    self.tile_rect(p0,iy,ix,2.0),
			    0.0,
			    Stroke::new(2.0,Color32::RED));
		    },
		    _ => ()
		}

		match hover {
		    Some((iy,ix)) => {
			let col = match self.tool {
			    Tool::Nothing => Color32::WHITE,
			    Tool::Place(_) | Tool::PlaceSky => {
				let x = ui.input(|input| input.time).rem_euclid(1.0) < 0.5;
				ui.ctx().request_repaint_after(Duration::from_millis(100));
				if x {
				    Color32::from_rgb(255,128,20)
				} else {
				    Color32::from_rgb(255,64,10)
				}
			    }
			};
			ui.painter().rect_stroke(
			    self.tile_rect(p0,iy,ix,3.0),
			    0.0,
			    Stroke::new(2.0,col));
		    },
		    _ => ()
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
