
// anything with proc like signals, Proce tree ops

use signal_hook::iterator::{SignalsInfo, exfiltrator::SignalOnly};
use sysinfo::Pid;

pub type Signals = SignalsInfo<SignalOnly>;

/// TODO: Right now we don't need to add Treemode but treemode might help visualize Process parent-child.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Unselected,
    Selected,
}

struct ProcessButton {
    state: State, 
    label: ProcessCommands,
}

impl ProcessButton {
    pub fn new() -> Self {
        ProcessButton {
            state: State::Unselected,
            label: ProcessCommands::Select,
        }
    }

    pub fn signal_process(&mut self, process_button: ProcessButton) {

    }

    pub fn render() {
        
    }

    fn selected_proc() {}

    fn kill_proc(pid: Pid, ) {}

}
