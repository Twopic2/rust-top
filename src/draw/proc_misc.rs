
// anything with proc like signals, Proce tree ops

use signal_hook::iterator::{SignalsInfo, exfiltrator::SignalOnly};
use sysinfo::Pid;

use crate::processes::processdata::CollectProcessData;

pub type Signals = SignalsInfo<SignalOnly>;

/// TODO: Right now we don't need to add Treemode but treemode might help visualize Process parent-child.

#[derive(PartialEq)]
enum ProcessCommands{
    Select,
    Kill,
//  Info,
//     Signals,
//     Terminate,
//     TreeMode, 
}

impl ProcessCommands {
    pub fn bottom_header(process_command: ProcessCommands) -> &'static str {
        match process_command {
            ProcessCommands::Select => "select",
            ProcessCommands::Kill => "kill",
        }
    }
}

struct ProcessButton {
    command: ProcessCommands,
}

impl ProcessButton {
    pub fn new() -> Self {
        ProcessButton {
            command: ProcessCommands::Select,
        }
    }

    pub fn signal_process(&mut self, process_button: ProcessButton) -> ProcessButton {
        if process_button.command == ProcessCommands::Select {

        } else if process_button.command == ProcessCommands::Kill {

        }    
        
    }

    pub fn render() {
        
    }

    fn selected_proc(&mut self, process: Vec<CollectProcessData>) -> Pid {


    }

    fn kill_proc(pid: Pid, ) {}

}
