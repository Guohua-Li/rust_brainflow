use egui::{
    CentralPanel,
    SidePanel,
    Ui,
    Context,
    Response,
    Pos2,
    Vec2,
    vec2,
    Button,
    Slider,
    ComboBox,
    FontId,
    RichText,
    FontFamily, FontData, FontDefinitions
};


use crate::eplot::PlotCtx;

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
};



const ALL_BOARDS: [BoardIds; 44] = [
    BoardIds::PlaybackFileBoard,
    BoardIds::StreamingBoard,
    BoardIds::SyntheticBoard,
    BoardIds::CytonBoard,
    BoardIds::GanglionBoard,
    BoardIds::CytonDaisyBoard,
    BoardIds::GaleaBoard,
    BoardIds::GanglionWifiBoard,
    BoardIds::CytonWifiBoard,
    BoardIds::CytonDaisyWifiBoard,
    BoardIds::BrainbitBoard,
    BoardIds::UnicornBoard,
    BoardIds::CallibriEegBoard,
    BoardIds::CallibriEmgBoard,
    BoardIds::CallibriEcgBoard,
    //BoardIds::FasciaBoard,//?
    BoardIds::Notion1Board,
    BoardIds::Notion2Board,
    //BoardIds::IronbciBoard,//?
    BoardIds::GforceProBoard,
    BoardIds::Freeeeg32Board,
    BoardIds::BrainbitBledBoard,
    BoardIds::GforceDualBoard,
    BoardIds::GaleaSerialBoard,
    BoardIds::MuseSBledBoard,
    BoardIds::Muse2BledBoard,
    BoardIds::CrownBoard,
    BoardIds::AntNeuroEe410Board,
    BoardIds::AntNeuroEe411Board,
    BoardIds::AntNeuroEe430Board,
    BoardIds::AntNeuroEe211Board,
    BoardIds::AntNeuroEe212Board,
    BoardIds::AntNeuroEe213Board,
    BoardIds::AntNeuroEe214Board,
    BoardIds::AntNeuroEe215Board,
    BoardIds::AntNeuroEe221Board,
    BoardIds::AntNeuroEe222Board,
    BoardIds::AntNeuroEe223Board,
    BoardIds::AntNeuroEe224Board,
    BoardIds::AntNeuroEe225Board,
    BoardIds::EnophoneBoard,
    BoardIds::Muse2Board,
    BoardIds::MuseSBoard,
    BoardIds::BrainaliveBoard,
    BoardIds::Muse2016Board,
    BoardIds::Muse2016BledBoard,
    //BoardIds::PieegBoard,//?
];


pub struct MyApp {
    plot_ctx: PlotCtx,
    grps_channels: Vec<Vec<usize>>,
    plt_size_mem: Vec2,
    plt_size: Vec2,
    show_xtick: bool,
    show_ytick: bool,
    drag_y_on: bool,
    zoom_y_on: bool,
    board_id: BoardIds,
    board: BoardShim,
    show_samples: usize,
}


impl Default for MyApp {
    fn default() -> Self {
        Self {
            //sampling_rate: 250,
            plot_ctx: PlotCtx::default(),
            grps_channels: vec![vec![1,2,3,4], vec![5,6,7,8]],
            plt_size_mem: vec2(300.0, 100.0),
            plt_size: vec2(0.0, 0.0),

            board_id: BoardIds::SyntheticBoard,
            board: BoardShim::new(BoardIds::SyntheticBoard, BrainFlowInputParamsBuilder::default().build()).unwrap(),

            show_xtick: false,
            show_ytick: false,
            drag_y_on: false,
            zoom_y_on: false,
            show_samples: 1000,
        }
    }
}


impl MyApp {

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        configure_fonts(&cc.egui_ctx);
        board_shim::enable_dev_board_logger().unwrap();
        let mut app: MyApp = Default::default();
        app.board.prepare_session().unwrap();
        app.board.start_stream(45000, "").unwrap();
        let channel_indeces = board_shim::get_eeg_channels(app.board_id, BrainFlowPresets::DefaultPreset).unwrap();
        app.grps_channels = channel_indeces.chunks(2).map(|s| s.into()).collect();
        app
    }
}



fn button(ui: &mut Ui, text: &str) -> Response {
    let btn = ui.add_sized(
        Vec2 {x: 130.0, y: 30.0 },
        Button::new(RichText::new(text).font(FontId::proportional(11.5)))
    );
    ui.add_space(3.0);
    btn
}


impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        SidePanel::left("left_panel").show(ctx, |ui| {
            let Self { board_id, .. } = self; //&mut,  channel_indeces, 
            ui.add_space(10.0);

            ui.collapsing("Board control", |ui| {
                if button(ui, "Prepare session").clicked() {
                    println!("prepare session");
                }
                if button(ui, "Start streaming").clicked() {
                    println!("start streaming");
                }
                if button(ui, "Stop streaming").clicked() {
                    println!("stop session");
                }
                if button(ui, "Release session").clicked() {
                    println!("release session");
                }
            });
            ui.add_space(20.0);

            ui.collapsing("Plot setting:", |ui| {
                if button(ui, "Show x-val").clicked() { self.show_xtick = !self.show_xtick; }
                if button(ui, "Show y-val").clicked() { self.show_ytick = !self.show_ytick; }
                if button(ui, "drag y-axis").clicked() { self.drag_y_on = !self.drag_y_on; }
                if button(ui, "zoom y-axis").clicked() { self.zoom_y_on = !self.zoom_y_on; }
            });
            ui.add_space(20.0);


            ui.group(|ui| {
                ui.label("Select One:");
                let id_string = board_id.to_string();
                ComboBox::new("cbid", "").selected_text(id_string).width(150.0).show_ui(ui, |ui| {
                    let board_id_clone = board_id.clone();
                    for board_item in ALL_BOARDS.iter() {
                        ui.selectable_value(board_id, *board_item, board_item.to_string());//???
                    }
                    if board_id_clone != *board_id {
                        let params = BrainFlowInputParamsBuilder::default().build();
                        if *board_id != BoardIds::SyntheticBoard {
                            println!("board selected: {:?}.", board_id);
                            println!("We don't have this board!\n")
                        } else {
                            self.board = BoardShim::new(*board_id, params).unwrap();
                            println!("We have this board!\n")
                        }
                    }
                });
                ui.set_min_height(20.0);
            });
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("Samples");
                ui.style_mut().spacing.slider_width = 100.0;
                ui.add(
                    Slider::new(&mut self.show_samples, 100..=2000)//.text("samples")
                );
            });
            ui.add_space(20.0);
            if button(ui, "Quit").clicked() {
                _frame.close()
            };

        });

        CentralPanel::default().show(ctx, |ui| {
            let size_available: Vec2 = ui.available_size_before_wrap();
            if !size_available.eq(&self.plt_size_mem) {
                let plt_x = (size_available.x-10.0) / 2.0;
                let plt_y = (size_available.y-20.0) / (self.grps_channels.len() as f32);
                self.plt_size_mem = size_available;
                self.plt_size = vec2(plt_x, plt_y);
            }

            let Self { plot_ctx, plt_size, show_xtick, show_ytick, drag_y_on, zoom_y_on, .. } = self;
            let data = self.board.get_current_board_data(
                self.show_samples,
                BrainFlowPresets::DefaultPreset
            ).unwrap();

            egui::Grid::new("fig_grid").show(ui, |ui| {
                for list in self.grps_channels.iter() {
                    for chn_i in list {
                        let raw_data: Vec<Pos2> = data.row(*chn_i).iter().enumerate().map(|(i, y)| {
                            Pos2 { x: i as f32,  y: *y as f32,}
                        }).collect();
                        let s = format!("plot {chn_i}");
                        let plt = plot_ctx.plot(s) // how big is the space between plts?
                            .size(*plt_size)
                            //.title(format!("chan {chn_i}"))
                            .set_x_end(self.show_samples as f32)
                            .show_ytick_val(*show_ytick)
                            .show_xtick_val(*show_xtick)
                            .set_drag_yaxis(*drag_y_on)
                            .set_zoom_y(*zoom_y_on);
                        plt.show(ui, raw_data);

                    }
                    ui.end_row();
                }
            });

        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.board.stop_stream().unwrap();
        if self.board.is_prepared().unwrap() {
            self.board.release_session().unwrap();
            println!("session released");
        }
    }

}

pub fn configure_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("MesloLGS".to_owned(), FontData::from_static(include_bytes!("MesloLGS_NF_Regular.ttf")));
    fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "MesloLGS".to_owned());
    fonts.families.get_mut(&FontFamily::Monospace).unwrap().push("MesloLGS".to_owned());
    ctx.set_fonts(fonts);
}

