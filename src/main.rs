//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(dead_code)]

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
	self,
	Button,
	menu,
	ImageSource,
	mutex::Mutex,
	Key,
	Vec2
    },
    egui_glow,
    glow
};

fn main()->Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
	viewport:egui::ViewportBuilder::default()
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
    tex:Option<egui::TextureHandle>,
    frame_rate:f32,
    play:bool,
    tv:TileViewer
}

impl Leved {
    fn new(_cc:&eframe::CreationContext<'_>)->Self {
	let tv = TileViewer::new();
	let mut this =
	    Self {
		tex:None,
		frame_rate:10.0,
		play:false,
		tv
	    };
	this
    }
}

pub struct TileViewer {
    img:Option<egui::load::TexturePoll>,
    tile_size:egui::Vec2,
    nx:usize,
    ny:usize
}

impl TileViewer {
    pub fn new()->Self {
	let img = None;
	let tile_size = egui::vec2(16.0,16.0);
	let nx = 16;
	let ny = 16;
	Self { img,tile_size,nx,ny }
    }

    pub fn do_ui(&mut self,ui:&mut egui::Ui)->egui::Response {
	let desired_size = egui::vec2(self.nx as f32,self.ny as f32)*
	    self.tile_size;
	let (rect,mut response) = ui.allocate_exact_size(desired_size,
							 egui::Sense::click());
	if ui.is_rect_visible(rect) {
	    let mut painter = ui.painter();

	    painter.rect(
		rect,
		0.0,
		egui::Color32::DARK_GREEN,
		egui::Stroke::NONE
	    );
	    
	    let tpoll = self.img.get_or_insert_with(|| {
		egui::include_image!("../gfx/tiles.png")
		    .load(
			ui.ctx(),
			egui::TextureOptions::NEAREST,
			egui::load::SizeHint::Size(32,32))
		    .expect("Can't load image")
	    });

	    if let Some(tid) = tpoll.texture_id() {
		painter.image(
		    tid,
		    rect,
		    egui::Rect::from_min_max(egui::pos2(0.0,0.0),egui::pos2(1.0,1.0)),
		    egui::Color32::WHITE
		);
	    }

	}
	response
    }
}

impl egui::Widget for &mut TileViewer {
    fn ui(self,ui:&mut egui::Ui)->egui::Response {
	self.do_ui(ui)
    }
}

impl eframe::App for Leved {
    fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame) {
	// let x0 = self.x0;
	// let x1 = self.x1;
	// let y0 = self.y0;
	// let y1 = self.y1;
	let mut step_it = false;
	egui::CentralPanel::default().show(ctx,|ui| {
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

	    // 	let cimg = egui::ColorImage::from_rgba_unmultiplied(
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
