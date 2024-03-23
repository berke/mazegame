use crate::common::*;

pub struct Refresher {
    /// Desired refresh period
    period:f32,

    /// Last refresh
    t_last:SystemTime
}

impl Refresher {
    pub fn new(period:f32)->Self {
	Self { period,t_last:SystemTime::now() }
    }

    pub fn tick(&mut self,ui:&mut Ui)->bool {
	let t_now = SystemTime::now();
	let dt = t_now.duration_since(self.t_last).unwrap().as_secs_f32();
	if dt >= self.period {
	    self.t_last = t_now;
	    ui.ctx().request_repaint_after(
		Duration::from_secs_f32(self.period));
	    true
	} else {
	    ui.ctx().request_repaint_after(
		Duration::from_secs_f32(self.period - dt)
	    );
	    false
	}
    }
}
