// Array<A, D> = ArrayBase<OwnedRepr<A>, D>

/*
use ndarray::{
   ArrayBase,
   OwnedRepr,
   Dim,
};*/

use std::sync::Arc;
use ndarray::Array2; // Array<A, Dim<[usize; 2]>>

use egui::{
   CentralPanel, TopBottomPanel, Ui, Context, Response, Color32, Vec2, vec2, menu,
   Button, Align2, Window, FontId, RichText, TextStyle, FontFamily, FontData,
   FontDefinitions, //ViewportCommand,
};

use egui_plot::{ Line, Plot, PlotPoints, };

use brainflow::{
   BoardIds,
   brainflow_input_params::{
      BrainFlowInputParamsBuilder,
   },
   board_shim::{
      self,
      BoardShim,
   },
   BrainFlowPresets,
   //error::Error,
};

const GREEN:  Color32   = Color32::from_rgb(17, 240, 97);
const BUTTON: TextStyle = TextStyle::Button;
const FTSIZE: f32       = 15.0;

 const TOPLT: Align2 = Align2::LEFT_TOP;
 const TOPRT: Align2 = Align2::RIGHT_TOP;

const BOARD:  BoardIds  = BoardIds::SyntheticBoard;
const PRESET: BrainFlowPresets = BrainFlowPresets::DefaultPreset;
const CHANS:  [[usize; 8]; 2] = [ [1, 2, 3, 4, 5, 6, 7, 8], [9, 10,11,12,13,14,15,16] ];

pub struct MyApp {
   board:        BoardShim,
   data:         Array2<f64>,
   canvas_size:  Vec2,
   plt_size:     Vec2,
   samples:      usize,
   is_streaming: bool,
}

impl Default for MyApp {
   fn default() -> Self {
      Self {
         data: Array2::<f64>::zeros((16, 1000)),
         board: BoardShim::new(
            BOARD,
            BrainFlowInputParamsBuilder::default().build()
         ).unwrap(),
         canvas_size: vec2(0.0, 0.0),
         plt_size: vec2(0.0, 0.0),
         samples: 1000,
         is_streaming: false,
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
      /*if let Err(error) = app.board.prepare_session() {
         panic!("ERROR: {:?}", error);
      }*/
      app.board.start_stream(45000, "").unwrap();
      /*if let Err(error) = app.board.start_stream(45000, "") {
         panic!("ERROR: {:?}", error);
      };*/
      app.is_streaming = true;
      app
   }

   fn top_panel(&mut self, ctx: &Context) {
      TopBottomPanel::top("top_panel").show(ctx, |ui| {
         menu::bar(ui, |ui| {
            ui.menu_button(RichText::new("Session").text_style(BUTTON), |ui| {
               if button(ui, "Prepare", FTSIZE).clicked() {
                  if !self.board.is_prepared().unwrap() {
                     self.board.prepare_session().unwrap();
                  }
               }
               if button(ui, "Release", FTSIZE).clicked() {
                  if self.board.is_prepared().unwrap() {
                     self.board.release_session().unwrap();
                  }
                  self.is_streaming = false;
               }
            });
            ui.menu_button(RichText::new("Streaming").text_style(BUTTON), |ui| {
               if button(ui, "Start", FTSIZE).clicked() {
                  if !self.board.is_prepared().unwrap() { return; }
                  if self.is_streaming { return; }
                  self.board.start_stream(45000, "").unwrap();
                  self.is_streaming = true;
               }
               if button(ui, "Stop", FTSIZE).clicked() {
                  if self.is_streaming {
                     self.board.stop_stream().unwrap();
                     self.is_streaming = false;
                  }
               }
            });
         });
      });
   }

   fn plt_data(&mut self, ctx: &Context) {
      for (w, list) in CHANS.iter().enumerate() {
         Window::new(format!("window {}", w+1)).resizable(false).vscroll(false)
         .anchor(if w == 0 {TOPLT} else {TOPRT}, vec2(0.0,0.0)).show(ctx, |ui| {
            for chn in list {
               let points = self.data.row(*chn).iter().enumerate().map(|(i, y)| {
                  [i as f64, *y as f64]
               }).collect::<PlotPoints>();

               let plot = Plot::new(format!("plot {chn}"))
               .width(self.plt_size.x).height(self.plt_size.y)
               .set_margin_fraction(vec2(0.0,0.0)).show_axes([false,false]);

               plot.show(ui, |plot_ui| {
                  plot_ui.line(Line::new(points).color(GREEN));
               });
            }
         });
      }
   }
}


impl eframe::App for MyApp {

   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      ctx.request_repaint();
      self.top_panel(ctx); // menu

      CentralPanel::default().show(ctx, |ui| {
         let s: Vec2 = ui.available_size_before_wrap();
         if !s.eq(&self.canvas_size) {
            self.canvas_size = s;
            self.plt_size = vec2((s.x-20.0) / 2.0, (s.y-60.0) / 8.0);
         }

         if self.is_streaming {
            //self.data = self.board.get_current_board_data(self.samples, PRESET).unwrap();
            self.data = match self.board.get_current_board_data(self.samples, PRESET) {
               Ok(data) => data,
               Err(error) => panic!("ERROR: {:?}", error),
            };
         }
         self.plt_data(ctx);
       });
   }

   fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
      if self.board.is_prepared().unwrap() {
         self.board.release_session().unwrap();
      }
   }
}

fn button(ui: &mut Ui, text: &str, size: f32) -> Response {
   let btn = ui.add_sized(
       Vec2 {x: 130.0, y: 30.0 },
       Button::new(RichText::new(text).font(FontId::proportional(size)))
   );
   ui.add_space(3.0);
   btn
}

pub fn configure_fonts(ctx: &Context) {
   let mut fonts = FontDefinitions::default();
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
