use std::sync::mpsc::{Sender, Receiver, self};

// TODO have one thread per each functional unit, and one unifying thread which syncronizes
// outputs from all the FU's.

// Order the outputs by some variation of program counter and looping.

