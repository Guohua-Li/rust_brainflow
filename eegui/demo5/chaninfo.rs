use brainflow::{board_shim, BoardIds, BrainFlowPresets};


const PRESET: BrainFlowPresets = BrainFlowPresets::DefaultPreset;
const BOARD:  BoardIds         = BoardIds::SyntheticBoard;

pub struct ChanInfo {
   pub description:     String,
   pub eeg_checked:     bool,
   pub marker_checked:  bool,
   pub battery_checked: bool,
   pub eeg_indices:     Vec<usize>,
   pub marker_index:    usize,
   pub battery_index:   usize,
}


impl Default for ChanInfo {
   fn default() -> Self {
      Self {
         description:     "".to_owned(),
         eeg_checked:     true,
         marker_checked:  false,
         battery_checked: false,
         eeg_indices:     vec![],
         marker_index:    0,
         battery_index:   0,
      }
   }
}

impl ChanInfo {
   pub fn collect(&mut self) {
      self.description   = board_shim::get_board_descr(BOARD, PRESET).unwrap();
      self.eeg_indices   = board_shim::get_eeg_channels(BOARD, PRESET).unwrap();
      self.marker_index  = board_shim::get_marker_channel(BOARD, PRESET).unwrap();
      self.battery_index = board_shim::get_battery_channel(BOARD, PRESET).unwrap();
   }
}

