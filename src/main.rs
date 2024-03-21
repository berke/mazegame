#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod common;
mod object;
mod tiles;
mod world;
mod room;
mod a2;
mod mini_rng;
mod tile_viewer;
mod ptr;

use common::*;
use a2::A2;
use tiles::{
    Corner,
    Door,
    Tile,
    Random
};
use object::Object;
use world::World;
use tile_viewer::TileViewer;

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
    world:World,
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
	    world:World::new(),
	    tv
	}
    }
}

impl eframe::App for Leved {
    fn update(&mut self,ctx:&Context,_frame:&mut eframe::Frame) {
	// let x0 = self.x0;
	// let x1 = self.x1;
	// let y0 = self.y0;
	// let y1 = self.y1;
	CentralPanel::default().show(ctx,|ui| {
	    if ui.button("Load").clicked() {
		let patho = rfd::FileDialog::new().pick_file()
		    .map(|pb| pb
			 .into_os_string()
			 .into_string()
			 .unwrap_or_else(|_| "WTF".to_string()));
		if let Some(path) = patho {
		    self.world.clear();
		    let _ = self.world.load(path);
		}
		// self.patho = patho;
	    }

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
