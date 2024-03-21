#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod object;
mod tiles;
mod a2;

use anyhow::{
    Result
};

use std::{
    time::SystemTime,
    sync::Arc,
    f64::consts::PI
};

use eframe::{
    egui::{
	Button,
	CentralPanel,
	Color32,
	Context,
	ImageSource,
	Key,
	load::{
	    self,
	    SizeHint,
	    TexturePoll
	},
	include_image,
	menu,
	mutex::Mutex,
	pos2,
	Pos2,
	Rect,
	Response,
	self,
	Sense,
	Stroke,
	TextureHandle,
	TextureOptions,
	Ui,
	vec2,
	Vec2,
	ViewportBuilder,
	Widget
    },
    egui_glow,
    glow
};

use a2::A2;
use tiles::{
    Corner,
    Door,
    Tile,
    Random
};
use object::Object;

fn main()->Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
	viewport:ViewportBuilder::default()
	    .with_inner_size([500.0,800.0]),
	multisampling:4,
	renderer:eframe::Renderer::Glow,
	..Default::default()
    };
    eframe::run_native(
	"Mazegame Level Editor",
	options,
	Box::new(|cc| {
	    egui_extras::install_image_loaders(&cc.egui_ctx);
	    Box::new(Leved::new(cc))
	}),
    )
}

struct Leved {
    tex:Option<TextureHandle>,
    frame_rate:f32,
    play:bool,
    tv:TileViewer
}

impl Leved {
    fn new(_cc:&eframe::CreationContext<'_>)->Self {
	let tv = TileViewer::new();
	Self {
	    tex:None,
	    frame_rate:10.0,
	    play:false,
	    tv
	}
    }
}

pub struct TileViewer {
    img:Option<load::TexturePoll>,
    tile_size:Vec2,
    map:A2<Tile>,
    rainbow_index:usize
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

    pub fn new()->Self {
	let img = None;
	let tile_size = vec2(32.0,32.0);
	let map = A2::new((16,16),Tile::Empty);
	Self { img,tile_size,map,rainbow_index:0 }
    }

    pub fn find_tile(&self,tl:Tile)->TileAspect {
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
	let (ny,nx) = self.map.dims();
	let desired_size = vec2(nx as f32,ny as f32)*self.tile_size;
	let (rect,mut response) = ui.allocate_exact_size(desired_size,
							 Sense::click());
	if ui.is_rect_visible(rect) {
	    let painter = ui.painter();

	    painter.rect(
		rect,
		0.0,
		Color32::BLACK,
		Stroke::NONE
	    );
	    
	    // let tpoll =
	    self.img.get_or_insert_with(|| {
		include_image!("../gfx/tiles.png")
		    .load(
			ui.ctx(),
			TextureOptions::NEAREST,
			load::SizeHint::Size(32,32))
		    .expect("Can't load image")
	    });

	    let p0 = rect.left_top();

	    for iy in 0..ny {
		for ix in 0..nx {
		    let p1 = p0 + vec2(ix as f32,iy as f32)*self.tile_size;
		    let p2 = p1 + self.tile_size;
		    let rect = Rect::from_points(&[p1,p2]);
		    match self.find_tile(self.map[[iy,ix]]) {
			TileAspect::Solid(color) => {
			    painter.rect(
				Rect::from_points(&[p1,p2]),
				0.0,
				color,
				Stroke::NONE
			    );
			},
			TileAspect::FromImage(q0,q1) => {
			    if let Some(TexturePoll::Ready { texture }) = self.img {
				let u0 = q0/texture.size;
				let u1 = q1/texture.size;
				let uv = Rect::from_points(&[u0.to_pos2(),u1.to_pos2()]);
				painter.image(
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
	response
    }
}

impl Widget for &mut TileViewer {
    fn ui(self,ui:&mut Ui)->Response {
	self.do_ui(ui)
    }
}

impl eframe::App for Leved {
    fn update(&mut self,ctx:&Context,_frame:&mut eframe::Frame) {
	// let x0 = self.x0;
	// let x1 = self.x1;
	// let y0 = self.y0;
	// let y1 = self.y1;
	CentralPanel::default().show(ctx,|ui| {
	    let _ = ui.checkbox(&mut self.play,
				"Play");

	    if ui.button("Refresh").clicked() {
		self.tex = None;
	    }

	    // ctx.load_texture(
	    // 	"foo",
	    // 	self.tv.img,
	    // 	Default::default());
	    ui.add(&mut self.tv);



	    // let tex = self.tex.get_or_insert_with(|| {
	    // 	let phi = self.phi.array();
	    // 	let (ny,nx) = phi.dims();
	    // 	let mut data = vec![0_u8;(ny*nx*4) as usize];
	    // 	for (i,v) in phi.as_slice().iter().enumerate() {
	    // 	    let v = v / self.phi1;
	    // 	    let (r,g,b) =
	    // 		if v < 0.0 {
	    // 		    (0.0,0.0,-v)
	    // 		} else {
	    // 		    (v,0.0,0.0)
	    // 		};
	    // 	    let f = |x:f32| (255.0*x).max(0.0).min(255.0).round() as u8;
	    // 	    let k = 4*i;
	    // 	    data[k + 0] = f(r);
	    // 	    data[k + 1] = f(g);
	    // 	    data[k + 2] = f(b);
	    // 	    data[k + 3] = 255;
	    // 	};

	    // 	let cimg = ColorImage::from_rgba_unmultiplied(
	    // 	    [nx as usize,
	    // 	     ny as usize],&data);
	    // 	ctx.load_texture(
	    // 	    "foo",
	    // 	    cimg,
	    // 	    Default::default())
	    // });

	    // if true {
	    // 	let fld = &self.phi;
	    // 	let phi = fld.array();
	    // 	let (ny,nx) = phi.dims();
	    // 	let gs = fld.grids();
	    // 	for iy in 0..ny {
	    // 	    let y = gs[0].get(iy,0.0);
	    // 	    for ix in 0..nx {
	    // 		let x = gs[1].get(ix,0.0);
	    // 		let p = vc([y,x]);
	    // 		let v1 = fld.interp(p);
	    // 		let v2 = phi[[iy,ix]];
	    // 		println!("{} {} {} {} {} {}",ix,iy,x,y,v1,v2);
	    // 	    }
	    // 	}
	    // }


	    if ui.button("Randomize").clicked() {
	    }

	});
    }
}
