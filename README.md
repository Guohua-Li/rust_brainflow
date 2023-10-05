# eegui
I am learning EGUI by building a gui for BrainFlow. I was trained to be a neurobiologist, coding is just my passion. Please correct me if you spot any mistakes. 

I forget to mention the compilation will fail unless you modify the source file like this:

In the path:
rust_package/brainflow/src/ffi/constants.rs

you will find the definition of:
#[repr(i32)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BoardIds {

///

}

After this definition, insert the following codes:

use std::fmt;

impl fmt::Display for BoardIds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

