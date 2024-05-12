use egui::{
   CentralPanel, TopBottomPanel, Ui, Context, menu, Response, Resize,  Sense, Shape, Pos2, pos2,
   Vec2, vec2, Button, FontId, RichText, FontFamily, FontData, FontDefinitions, Stroke, Color32, remap
};


use brainflow::{
   BoardIds,
   brainflow_input_params::BrainFlowInputParamsBuilder,
   board_shim::{ self, BoardShim },
   BrainFlowPresets,
};


const BOARD:   BoardIds         = BoardIds::SyntheticBoard;
const PRESET:  BrainFlowPresets = BrainFlowPresets::DefaultPreset;
const CHANS:  [usize; 16]       = [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, ];

const FTSIZE:  f32     = 15.0;
const GREEN:   Color32 = Color32::from_rgb(0, 255, 0);
const GRAY10:  Color32 = Color32::from_gray(10);
const GRAY150: Color32 = Color32::from_gray(150);


pub struct MyApp {
   board:        BoardShim,
   is_streaming: bool,
   samples:      usize,
   ui_size:      Vec2,
   plt_size:     Vec2,
   xaxis_on:     bool,
   yaxis_on:     bool,
   x_start:      f32,
   x_end:        f32,
   y_start:      f32,
   y_end:        f32,
}


impl Default for MyApp {
   fn default() -> Self {
      Self {
         board: BoardShim::new(BOARD, BrainFlowInputParamsBuilder::default().build() ).unwrap(),
         is_streaming: false,
         samples: 1000,
         ui_size: vec2(0.0, 0.0),
         plt_size: vec2(0.0, 0.0),
         xaxis_on: false,
         yaxis_on: false,
         x_start: 0.0,
         x_end: 1000.0,
         y_start: -10.0,
         y_end: 35.0,
      }
   }
}



impl MyApp {
   pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
      cc.egui_ctx.set_visuals(egui::Visuals::dark());
      configure_fonts(&cc.egui_ctx);
      board_shim::disable_board_logger().unwrap();
      let mut app: MyApp = Default::default();
      app.board.prepare_session().unwrap();
      app.board.start_stream(45000, "").unwrap();
      app.is_streaming = true;
      app
   }

   fn top_panel(&mut self, ctx: &Context) {
      TopBottomPanel::top("top_panel").show(ctx, |ui| {
         ui.horizontal(|ui| {
            if self.board.is_prepared().unwrap() {
               if button(ui, "Release Session", FTSIZE, 30.).clicked() {
                  self.board.release_session().unwrap();
                  self.is_streaming = false;
               }
               if self.is_streaming {
                  if button(ui, "Stop Streaming", FTSIZE, 30.).clicked() {
                     self.board.stop_stream().unwrap();
                     self.is_streaming = false;
                  }
               } else {
                  if button(ui, "Start Streaming", FTSIZE, 30.).clicked() {
                     self.board.start_stream(45000, "").unwrap();
                     self.is_streaming = true;
                  }
               }
            } else {
               if button(ui, "Prepare Session", FTSIZE, 30.).clicked() {
                  self.board.prepare_session().unwrap();
               }
            }

            ui.add_space(50.);

            menu::bar(ui, |ui| {
               ui.menu_button(RichText::new("Plot").font(FontId::proportional(FTSIZE)), |ui| {
                  if button(ui, "x axis", 16.0, 1.0).clicked() {
                     self.xaxis_on = !self.xaxis_on;
                  }
                  if button(ui, "y axis", 16.0, 1.0).clicked() {
                     self.yaxis_on = !self.yaxis_on;
                  }
               });
            });
         });
      });
   }

   fn plt_data(&mut self, ui: &mut Ui) {
      let all_data = self.board.get_current_board_data(self.samples, PRESET).unwrap();
      for chn in CHANS.iter() {
         let row_data: Vec<Pos2> = all_data.row(*chn).iter().enumerate().map(|(i, y)| {
            Pos2 { x: i as f32,  y: *y as f32 }
         }).collect();
         let resize = Resize::default().id_source(format!("{}", chn)).fixed_size(self.plt_size);
         resize.show(ui, |ui: &mut Ui| {
            let space: Vec2 = ui.available_size();
            //println!("{}", space == self.plt_size);
            let (response, painter) = ui.allocate_painter(space, Sense::drag() );
            let full_rect = response.rect;
            painter.rect(full_rect, 0.0, GRAY10, Stroke::new(1.0, GRAY150));
            let plt_to_scr = |p: &Pos2| -> Pos2 {
               let x = remap(p.x, self.x_start..=self.x_end, full_rect.x_range());
               let y = remap(p.y, self.y_end..=self.y_start, full_rect.y_range());
               pos2(x, y)
            };
            let points: Vec<Pos2> = row_data.iter().map( |p| plt_to_scr(p) ).collect();
            painter.add(Shape::line(points, Stroke::new(1.0, GREEN)));
         });
      }
   }
}



fn button(ui: &mut Ui, text: &str, size: f32, space: f32) -> Response {
   let btn: Response = ui.add_sized(
      Vec2 {x: 130.0, y: 30.0 },
      Button::new(RichText::new(text).font(FontId::proportional(size))).frame(false)
   );
   ui.add_space(space);
   btn
}


impl eframe::App for MyApp {

   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      ctx.request_repaint();
      self.top_panel(ctx);
      CentralPanel::default().show(ctx, |ui| {
         let s: Vec2 = ui.available_size_before_wrap();
         if !s.eq(&self.ui_size) {
            self.ui_size = s;
            self.plt_size = vec2(s.x, (s.y-5.0)/16.0 - 2.5);
         }
         if self.is_streaming {
            self.plt_data(ui);
         }
      });
   }

   fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
      if self.board.is_prepared().unwrap() {
         self.board.release_session().unwrap();
      }
   }
}


pub fn configure_fonts(ctx: &Context) {
   let mut fonts = FontDefinitions::default();
   fonts.font_data.insert(
      "MesloLGS".to_owned(),
      FontData::from_static(include_bytes!("../fonts/MesloLGS_NF_Regular.ttf"))
   );
   fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "MesloLGS".to_owned());
   fonts.families.get_mut(&FontFamily::Monospace).unwrap().push("MesloLGS".to_owned());
   ctx.set_fonts(fonts);
}
