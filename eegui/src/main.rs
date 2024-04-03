use std::{
    thread,
    time::Duration
};

use brainflow::{
    board_shim::{
        self,
        BoardShim, // BoardShim is the main interface for all boards
    },
    brainflow_input_params::BrainFlowInputParamsBuilder,
    BoardIds,
    BrainFlowPresets,
};


fn main() {
    board_shim::enable_dev_board_logger().unwrap();
    let _ = board_shim::set_log_file("brainflow.log"); // log info will go to this file
    let params = BrainFlowInputParamsBuilder::default().build();
    let board = BoardShim::new(BoardIds::SyntheticBoard, params).unwrap();
    board.prepare_session().unwrap(); // print params
    // buffer_size
    // streamer_params
    board.start_stream(45000, "").unwrap(); // streams data and stores it in a ring buffer
    thread::sleep(Duration::from_secs(5)); // Puts the current thread to sleep for at least 5 sec.

    board.stop_stream().unwrap();
    // get all data and remove it from the internal buffer
    let data = board.get_board_data(Some(10), BrainFlowPresets::DefaultPreset).unwrap();

    board.release_session().unwrap(); // release all resources
    println!("{}", data.len());
    println!("{:?}", data);
}
