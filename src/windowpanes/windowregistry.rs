use crate::windowpanes::window::Window;
use crate::windowpanes::pianoroll::PianoRollPane;

pub enum WindowRegistryEntry {
    Category {
        name: &'static str,
        children: Vec<WindowRegistryEntry>,
    },

    Window {
        name: &'static str,
        create: fn() -> Box<dyn Window>,
    },
}

pub fn get_window_registry() -> Vec<WindowRegistryEntry> {
    vec![
        WindowRegistryEntry::Category {
            name: "MIDI",
            children: vec![
                WindowRegistryEntry::Window {
                    name: "Piano Roll",
                    create: || Box::new(PianoRollPane::new()),
                },
            ],
        },

        WindowRegistryEntry::Category {
            name: "Extra",
            children: vec![],
        },
    ]
}

