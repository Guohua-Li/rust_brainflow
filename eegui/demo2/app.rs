use std::collections::HashSet;
use std::sync::Arc;

use egui::{
   CentralPanel, TopBottomPanel, Ui, Context, Response,
   Color32, Vec2, vec2, menu, Button, Key,
   ScrollArea, FontId, RichText, FontFamily,
   FontData, FontDefinitions,
};

use egui_plot::{ Line, Plot, PlotPoints, };


use brainflow::{
   BoardIds,
   brainflow_input_params::{
      BrainFlowInputParamsBuilder,
   },
   board_shim::{ self, BoardShim, },
   BrainFlowPresets,
};

const CHANS:  [usize; 16] = [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, ];
const FTSIZE: f32         = 15.0;

const GREEN:   Color32          = Color32::from_rgb(17, 240, 97);
const BOARD:   BoardIds         = BoardIds::SyntheticBoard;
const PRESET:  BrainFlowPresets = BrainFlowPresets::DefaultPreset;


pub struct MyApp {
   board:        BoardShim,
   is_streaming: bool,
   ui_size:      Vec2,
   plt_size:     Vec2,
   xaxis_on:     bool,
   yaxis_on:     bool,
   samples:      usize,
   //zoom_y:       bool,
   //drag_y_on: bool,
   //grps_channels: Vec<usize>,
}


impl Default for MyApp {
   fn default() -> Self {
      Self {
         board: BoardShim::new(
            BOARD,
            BrainFlowInputParamsBuilder::default().build()
         ).unwrap(),
         is_streaming: false,
         ui_size:  vec2(300.0, 100.0),
         plt_size: vec2(0.0, 0.0),
         xaxis_on: false,
         yaxis_on: false,
         samples: 1000,
         //zoom_y:  false,
         //drag_y_on: false,
         //grps_channels: vec![1,2,3,4,5,6,7,8, 9,10,11,12,13,14,15,16],
      }
   }
}


impl MyApp {

   pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
      cc.egui_ctx.set_visuals(egui::Visuals::dark());
      configure_fonts(&cc.egui_ctx);
      let mut app: MyApp = Default::default();
      board_shim::disable_board_logger().unwrap();
      app.board.prepare_session().unwrap();
      app.board.start_stream(45000, "").unwrap();
      //app.grps_channels = board_shim::get_eeg_channels(BOARD, PRESET).unwrap();
      app.is_streaming = true;
      app
   }

   fn top_panel(&mut self, ctx: &Context) {
      TopBottomPanel::top("top_panel").show(ctx, |ui| {
         ui.horizontal(|ui| {
            /*if button(ui, "Prepare", FTSIZE, 10.0).clicked() {
               if !self.board.is_prepared().unwrap() {
                  self.board.prepare_session().unwrap();
               }
            }
            if button(ui, "Release", FTSIZE, 10.0).clicked() {
               if self.board.is_prepared().unwrap() {
                  self.board.release_session().unwrap();
               }
               self.is_streaming = false;
            }
            if button(ui, "Start Streaming", FTSIZE, 10.0).clicked() {
               if !self.board.is_prepared().unwrap() { return; }
               if self.is_streaming { return; }
               self.board.start_stream(45000, "").unwrap();
               self.is_streaming = true;
            }
            if button(ui, "Stop Streaming", FTSIZE, 10.0).clicked() {
               if self.is_streaming {
                  self.board.stop_stream().unwrap();
                  self.is_streaming = false;
               }
            }*/

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
                  let switch = if self.xaxis_on { "hide x-axis" } else { "show x-axis" };
                  if button(ui, switch, 16.0, 1.0).clicked() {
                     self.xaxis_on = !self.xaxis_on;
                  }
                  let switch = if self.yaxis_on { "hide y-axis" } else { "show y-axis" };
                  if button(ui, switch, 16.0, 1.0).clicked() {
                     self.yaxis_on = !self.yaxis_on;
                  }
               });
            });
         });
      });
   }

   fn plt_data(&mut self, ui: &mut Ui) {
      let data = self.board.get_current_board_data(self.samples, PRESET).unwrap();

      ScrollArea::vertical().show(ui, |ui| {
         for chn in CHANS {
            let points = data.row(chn).iter().enumerate().map(|(i, y)| {
               [i as f64, *y as f64]
            }).collect::<PlotPoints>();
            let plot: Plot = Plot::new(format!("plot {chn}"))
            .width(self.plt_size.x).height(self.plt_size.y)
            .set_margin_fraction(vec2(0.0,0.0))
            .allow_drag(false)
            .show_x(false).show_y(false).show_grid(false)
            .show_axes([self.xaxis_on && chn == 16, self.yaxis_on]);
            //.auto_bounds();
            //.include_x(0.0)
            //.clamp_grid(true)
            //.view_aspect(2.0).include_y(100.0)
            //.y_axis_label("y");

            plot.show(ui, |plot_ui| {
               plot_ui.line(Line::new(points).color(GREEN));
            });
         }
      });
   }
}

impl eframe::App for MyApp {

   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      ctx.request_repaint();
      self.top_panel(ctx);
      let keys_down: HashSet<Key> = ctx.input(|i|
         i.keys_down.to_owned()
      );
      if keys_down.contains(&Key::ArrowUp) {
         self.samples += 10;
         if self.samples > 2000 { self.samples = 2000; }
      } else if keys_down.contains(&Key::ArrowDown) {
         self.samples -= 10;
         if self.samples < 100 { self.samples = 100; }
      }

      CentralPanel::default().show(ctx, |ui| {
         let s: Vec2 = ui.available_size_before_wrap();
         if !s.eq(&self.ui_size) {
            self.ui_size = s;
            self.plt_size = vec2(s.x-2.0, s.y/16.0-3.0);
         }
         if !self.is_streaming {
            return;
         }
         self.plt_data(ui);
      });
   }

   fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
      if self.board.is_prepared().unwrap() {
         self.board.release_session().unwrap();
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


pub fn configure_fonts(ctx: &Context) {
   let mut fonts: FontDefinitions = FontDefinitions::default();
   fonts.font_data.insert(
      "MesloLGS".to_owned(),
      //FontData::from_static(include_bytes!("../fonts/MesloLGS_NF_Regular.ttf"))
        Arc::new(FontData::from_static(include_bytes!(
            "../fonts/MesloLGS_NF_Regular.ttf"
        ))),
   );
   fonts.families.get_mut(&FontFamily::Proportional)
   .unwrap()
   .insert(0, "MesloLGS".to_owned());
   fonts.families.get_mut(&FontFamily::Monospace)
   .unwrap()
   .push("MesloLGS".to_owned());
   ctx.set_fonts(fonts);
}
