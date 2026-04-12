#![windows_subsystem = "windows"]

use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
    thread,
};

use eframe::egui;
use egui::{Color32, Style};
use keyboard_provider::VirtualKeyboard;
use keycodes::{Key, KeyEvent, KeyEvents};
use midir::{Ignore, MidiInput, MidiInputPort};
use output_methods::OutputMethod;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

use output_methods::unified::generic;
use output_methods::unified::piano_rooms_inner;
use output_methods::unified::pv_inner;

use output_methods::unified::GenericKeymap;

mod keycodes;
mod output_methods;

use midi_event::{self, MidiEventType, Parse};

use crate::output_methods::generic::REGULAR_VP_NOTES;

mod keyboard_provider;

#[allow(dead_code)] // used for comparisons
#[derive(Serialize, Deserialize, Clone)]
enum AvailableOutputMethod {
    Generic(Option<GenericKeymap>),
    PV,
    PianoRooms,
}

#[derive(Clone)]
struct MyPortInfo {
    port: MidiInputPort,
    name: String,
}
impl Debug for MyPortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct SavedConfig {
    pub preferred_port_name: Option<String>,
    pub preferred_output_method: AvailableOutputMethod,
    pub saved_generic_keymap: GenericKeymap,
}
impl ::std::default::Default for SavedConfig {
    fn default() -> Self {
        Self {
            preferred_port_name: None,
            preferred_output_method: AvailableOutputMethod::Generic(Some(GenericKeymap {
                ..Default::default()
            })),
            saved_generic_keymap: GenericKeymap {
                ..Default::default()
            },
        }
    }
}

struct Settings {
    output_method: Arc<Mutex<dyn OutputMethod + Send>>,
    port: Option<MyPortInfo>,
    output: bool,
    pv_velocity: bool,
}
impl Settings {
    fn with_output_method(method: AvailableOutputMethod) -> Self {
        let output_method: Arc<Mutex<dyn OutputMethod + Send>> = match method {
            AvailableOutputMethod::Generic(keymap) => {
                let unwrapped = keymap.unwrap();
                Arc::new(Mutex::new(generic::Inner::new(Some(GenericKeymap {
                    keys: unwrapped.keys,
                    sustain: unwrapped.sustain,
                }))))
            }
            AvailableOutputMethod::PV => Arc::new(Mutex::new(pv_inner::new())),
            AvailableOutputMethod::PianoRooms => Arc::new(Mutex::new(piano_rooms_inner::new())),
        };

        Self {
            output_method,
            port: None,
            output: true,
            pv_velocity: true,
        }
    }
    fn set_port(mut self, port: MyPortInfo) -> Self {
        self.port = Some(port);
        self
    }
}

fn midi_update_thread(
    midi: MidiInput,
    settings: &Arc<RwLock<Settings>>,
    keyboard: VirtualKeyboard,
    settings_update_receiver: mpsc::Receiver<bool>,
) {
    let settings = Arc::clone(settings);

    let keeb = Arc::new(Mutex::new(keyboard));
    let keeb_clone = Arc::clone(&keeb);

    let create_connection = |midi: MidiInput, keeb: Arc<Mutex<VirtualKeyboard>>| {
        let port = &settings.read().unwrap().port.clone().unwrap().port.clone();
        let port_name = settings.read().unwrap().port.clone().unwrap().name.clone();

        let settings = Arc::clone(&settings);
        println!("Connecting to {}", port_name);
        midi.connect(
            &port,
            &port_name,
            move |_timestamp, message, _| {
                if !settings.read().unwrap().output {
                    return;
                }
                let parsed_event = midi_event::Event::parse(message).unwrap();
                match parsed_event {
                    midi_event::Event::Midi(event) => {
                        let keypresses: KeyEvents = match event.event {
                            MidiEventType::NoteOn(note, velocity) => {
                                if velocity == 0 {
                                    // Some pianos (Alesis Recital Grand, reportedly) send a NoteOn with 0 velocity instead of NoteOff
                                    settings
                                        .try_read()
                                        .unwrap()
                                        .output_method
                                        .lock()
                                        .unwrap()
                                        .release_note(note)
                                        .clone()
                                } else {
                                    // Non-zero, real down press
                                    settings
                                        .try_read()
                                        .unwrap()
                                        .output_method
                                        .lock()
                                        .unwrap()
                                        .press_note(note, velocity)
                                        .clone()
                                }
                            }
                            MidiEventType::NoteOff(note, _) => settings
                                .try_read()
                                .unwrap()
                                .output_method
                                .lock()
                                .unwrap()
                                .release_note(note)
                                .clone(),
                            MidiEventType::Controller(control, value) => match control {
                                64 => settings
                                    .try_read()
                                    .unwrap()
                                    .output_method
                                    .lock()
                                    .unwrap()
                                    .process_sustain(value),
                                66 => settings
                                    .try_read()
                                    .unwrap()
                                    .output_method
                                    .lock()
                                    .unwrap()
                                    .process_sostenuto(value),
                                other_control => {
                                    println!("Unknown control to set: {other_control}");
                                    vec![]
                                }
                            },
                            anything_else => {
                                println!("Unsupported MIDI event type: {:?}", anything_else);
                                vec![]
                            }
                        };
                        keeb.lock().unwrap().write_many(keypresses);
                    }
                    _ => {
                        println!("Unsupported higher-level event type")
                    }
                }
            },
            (),
        )
        .unwrap()
    };

    // Check if port exists. if it does and no connection yet, make connection and continue looping maybe?
    let mut connection = create_connection(midi, keeb_clone);

    loop {
        // Received signal that settings have changed which require reconnection
        settings_update_receiver.recv().unwrap();

        let all_key_releases: Vec<KeyEvent> = keycodes::KEYCODES
            .keys()
            .map(|x| KeyEvent::Release(Key::new(*x)))
            .collect();
        keeb.lock().unwrap().write_many(all_key_releases);
        println!("Released all keys");

        connection.close();

        let mut midi_in =
            MidiInput::new("miditoqwerty input reader").expect("Failed to create MidiInput");
        midi_in.ignore(Ignore::TimeAndActiveSense);

        connection = create_connection(midi_in, Arc::clone(&keeb));
    }
}

fn main() -> eframe::Result<()> {
    let virtual_keyboard = keyboard_provider::create_virtual_keyboard();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 180.0]),
        ..Default::default()
    };

    let config: Arc<RwLock<SavedConfig>> = Arc::new(RwLock::new(
        confy::load("miditoqwerty-rs", None).expect("Failed to load configuration"),
    ));

    let mut midi_in =
        MidiInput::new("miditoqwerty input reader").expect("Failed to create MidiInput");
    midi_in.ignore(Ignore::TimeAndActiveSense);
    let midi_in = Some(midi_in);

    // Immutable and does not handle actual input reading (.connect is never called), etc.
    let meta_midi_in =
        MidiInput::new("miditoqwerty meta reader").expect("Unable to create meta MidiInput");

    let ports = Arc::new(RwLock::new(
        meta_midi_in
            .ports()
            .into_iter()
            .map(|port| MyPortInfo {
                port: port.clone(),
                name: meta_midi_in
                    .port_name(&port)
                    .unwrap_or("This port is no longer valid.".to_owned()),
            })
            .collect::<Vec<MyPortInfo>>(),
    )); // initial port population

    dbg!(ports.clone());

    let first_port = ports.read().unwrap().first().cloned();
    let first_port = Arc::new(RwLock::new(match first_port {
        Some(port) => port,
        None => {
            let mut options_for_popup = options.clone();
            options_for_popup.viewport =
                egui::ViewportBuilder::default().with_inner_size([320.0, 120.0]);

            let _ = eframe::run_simple_native(
                "Midi to Qwerty [error]",
                options.clone(),
                move |ctx, _frame| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("No available input ports found");
                    });
                },
            );
            panic!("At least one input port is needed");
        }
    }));

    let port_list_ports_clone = Arc::clone(&ports);
    let port_list_first_port_clone = Arc::clone(&first_port);
    thread::spawn(move || {
        // Only moves the clone
        // Immutable and does not handle actual input reading (.connect is never called), etc.
        let meta_midi_in = MidiInput::new("miditoqwerty port reader");
        let meta_midi_in = meta_midi_in.expect("Unable to create port MidiInput");

        loop {
            {
                let mut ports = port_list_ports_clone.write().unwrap();
                *ports = meta_midi_in
                    .ports()
                    .into_iter()
                    .map(|port| MyPortInfo {
                        port: port.clone(),
                        name: meta_midi_in
                            .port_name(&port)
                            .unwrap_or("This port is no longer valid.".to_owned()),
                    })
                    .collect::<Vec<MyPortInfo>>();
            }
            let mut first_port = port_list_first_port_clone.write().unwrap();
            *first_port = port_list_ports_clone
                .read()
                .unwrap()
                .first()
                .cloned()
                .expect("At least one input port is needed");

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
    // port_list_update_thread.join().unwrap();

    let port_wait_ports_clone = Arc::clone(&ports);
    loop {
        // Wait for thread above to update the value
        std::thread::sleep(std::time::Duration::from_millis(50));
        if port_wait_ports_clone.read().unwrap().len() > 0 {
            break;
        };
    }

    println!("Available ports: {:?}", ports.read().unwrap());

    let settings = Arc::new(RwLock::new(
        Settings::with_output_method(config.read().unwrap().preferred_output_method.clone())
            .set_port({
                let port = first_port.read().unwrap().port.clone();
                let name = meta_midi_in.port_name(&port).unwrap();
                MyPortInfo { port, name }
            }),
    ));

    // Select port from saved config
    let port_name_option = config.read().unwrap().preferred_port_name.clone();
    if let Some(desired_port_name) = port_name_option {
        let ports = ports.read().unwrap();
        if let Some(port) = ports.iter().find(|&port| port.name == desired_port_name) {
            let mut my_settings = settings.write().unwrap();
            my_settings.port = Some(port.clone());
        } else {
            println!("Configured port was not found, first will be used")
        };
    }

    // If anything is transmitted to this receiver, midi_update_thread restarts the MIDI connection with the new &settings
    let (settings_update_tx, settings_update_rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) =
        mpsc::channel();

    thread::spawn({
        let settings = Arc::clone(&settings);
        move || {
            midi_update_thread(
                midi_in.unwrap(),
                &settings,
                virtual_keyboard,
                settings_update_rx,
            )
        }
    });

    let settings = Arc::clone(&settings);

    let update_settings = move || {
        settings_update_tx
            .send(true)
            .expect("Failed to update listener");
    };

    let mut generic_keymap_string = config.read().unwrap().saved_generic_keymap.keys.clone();
    let mut generic_keymap_string_valid = true;
    let mut sustain_keymap_string = config.read().unwrap().saved_generic_keymap.sustain.clone();
    let mut sustain_keymap_string_valid = true;

    let mut did_style = false;
    eframe::run_simple_native("Midi to Qwerty", options, move |ctx, _frame| {
        if !did_style {
            did_style = true;
            ctx.set_style({
                let mut custom_style = Style::default();
                custom_style.interaction.tooltip_delay = 0.0;
                custom_style.interaction.show_tooltips_only_when_still = false;
                custom_style
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let selected_midi_port_name = settings.read().unwrap().port.clone().unwrap().name;
            egui::ComboBox::from_label("MIDI Input")
                .selected_text(selected_midi_port_name)
                .show_ui(ui, |ui| {
                    for (port, port_name) in meta_midi_in
                        .ports()
                        .iter()
                        .map(|port| {
                            (
                                port,
                                meta_midi_in
                                    .port_name(port)
                                    .unwrap_or("This port is no longer valid.".to_owned()),
                            )
                        })
                        .collect::<Vec<(&MidiInputPort, String)>>()
                    {
                        if ui.selectable_label(false, port_name.clone()).clicked() {
                            settings.write().unwrap().port = Some(MyPortInfo {
                                port: port.clone(),
                                name: port_name.clone(),
                            });
                            update_settings();
                            config.write().unwrap().preferred_port_name = Some(port_name.clone());

                            confy::store("miditoqwerty-rs", None, &(*config.read().unwrap()))
                                .expect("Failed to store configuration");
                        }
                    }
                });

            let selected_output_method_name = {
                let settings_read = settings.try_read().unwrap();
                let output_method = settings_read.output_method.lock().unwrap();
                output_method.get_name()
            };

            egui::ComboBox::from_label("Output Method")
                .selected_text(&selected_output_method_name)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(false, "Generic")
                        .on_hover_text("Basic QWERTY system, no 88-key or velocity support")
                        .clicked()
                    {
                        update_settings();

                        let new_keymap = GenericKeymap {
                            keys: generic_keymap_string.to_string(),
                            sustain: sustain_keymap_string.to_string(),
                        };

                        settings.write().unwrap().output_method =
                            Arc::new(Mutex::new(generic::Inner::new(Some(new_keymap.clone()))));

                        config.write().unwrap().preferred_output_method =
                            AvailableOutputMethod::Generic(Some(new_keymap.clone()));
                        confy::store("miditoqwerty-rs", None, &(*config.read().unwrap()))
                            .expect("Failed to store configuration");
                    }
                    if ui
                        .selectable_label(false, "Piano Visualizations")
                        .on_hover_text("Uses control for 88-key and alt for velocity")
                        .clicked()
                    {
                        update_settings();
                        let mut my_settings = settings.write().unwrap();
                        let velocity_info = match my_settings.pv_velocity {
                            true => "velocity-on",
                            false => "velocity-off",
                        };
                        my_settings.output_method = Arc::new(Mutex::new(pv_inner::new()));
                        my_settings
                            .output_method
                            .lock()
                            .unwrap()
                            .reset(velocity_info);

                        config.write().unwrap().preferred_output_method = AvailableOutputMethod::PV;
                        confy::store("miditoqwerty-rs", None, &(*config.read().unwrap()))
                            .expect("Failed to store configuration");
                    }
                    if ui
                        .selectable_label(false, "Piano Rooms")
                        .on_hover_text(
                            "Uses the custom numpad input system\nimplemented by Piano Rooms",
                        )
                        .clicked()
                    {
                        update_settings();

                        settings.write().unwrap().output_method =
                            Arc::new(Mutex::new(piano_rooms_inner::new()));

                        config.write().unwrap().preferred_output_method =
                            AvailableOutputMethod::PianoRooms;
                        confy::store("miditoqwerty-rs", None, &(*config.read().unwrap()))
                            .expect("Failed to store configuration");
                    }
                });

            if &selected_output_method_name == "Piano Visualizations" {
                if ui
                    .checkbox(
                        &mut settings.write().unwrap().pv_velocity,
                        "Use velocity (alt)",
                    )
                    .clicked()
                {
                    update_settings();
                    let my_settings = settings.write().unwrap();
                    let info = match my_settings.pv_velocity {
                        true => "velocity-on",
                        false => "velocity-off",
                    };
                    my_settings.output_method.lock().unwrap().reset(info);
                }
            }

            if ui
                .checkbox(&mut settings.write().unwrap().output, "Enable output")
                .clicked()
            {
                update_settings();
                let my_settings = settings.write().unwrap();
                my_settings.output_method.lock().unwrap().reset("");
            }

            if &selected_output_method_name == "Generic" {
                ui.collapsing("Edit keymap", |ui| {
                    ui.horizontal(|ui| {
                        let text_edit_response = ui.add(
                            egui::TextEdit::singleline(&mut generic_keymap_string).text_color(
                                if generic_keymap_string_valid {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                },
                            ),
                        );
                        if text_edit_response.changed() {
                            generic_keymap_string_valid = generic_keymap_string.len() == 61;

                            if generic_keymap_string_valid {
                                let new_keymap = GenericKeymap {
                                    keys: generic_keymap_string.to_string(),
                                    sustain: sustain_keymap_string.to_string(),
                                };

                                update_settings();
                                let mut my_settings = settings.write().unwrap();

                                my_settings.output_method = Arc::new(Mutex::new(
                                    generic::Inner::new(Some(new_keymap.clone())),
                                ));

                                config.write().unwrap().preferred_output_method =
                                    AvailableOutputMethod::Generic(Some(new_keymap.clone()));
                                config.write().unwrap().saved_generic_keymap = new_keymap.clone();
                                confy::store("miditoqwerty-rs", None, &(*config.read().unwrap()))
                                    .expect("Failed to store configuration");
                            }
                        }

                        if ui.button("Reset").clicked() {
                            generic_keymap_string =
                                String::from_utf8_lossy(&REGULAR_VP_NOTES).to_string();
                            generic_keymap_string_valid = true;

                            sustain_keymap_string = "space".to_string();
                            sustain_keymap_string_valid = true;

                            let mut my_settings = settings.write().unwrap();
                            let new_keymap = GenericKeymap {
                                keys: String::from_utf8_lossy(&REGULAR_VP_NOTES).to_string(),
                                sustain: "space".to_string(),
                            };

                            my_settings.output_method =
                                Arc::new(Mutex::new(generic::Inner::new(Some(new_keymap.clone()))));

                            config.write().unwrap().preferred_output_method =
                                AvailableOutputMethod::Generic(Some(new_keymap.clone()));
                            config.write().unwrap().saved_generic_keymap = new_keymap.clone();
                            confy::store("miditoqwerty-rs", None, &(*config.read().unwrap()))
                                .expect("Failed to store configuration");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sustain:");
                        let sustain_text_edit_response = ui.add(
                            egui::TextEdit::singleline(&mut sustain_keymap_string).text_color(
                                if sustain_keymap_string_valid {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                },
                            ),
                        );

                        if sustain_text_edit_response.changed() {
                            println!("sustain changed to: {}", sustain_keymap_string);
                            sustain_keymap_string_valid = Key::exists(&sustain_keymap_string);

                            if sustain_keymap_string_valid {
                                update_settings();

                                let mut my_settings = settings.write().unwrap();
                                let new_keymap = GenericKeymap {
                                    keys: generic_keymap_string.to_string(),
                                    sustain: sustain_keymap_string.to_string(),
                                };

                                my_settings.output_method = Arc::new(Mutex::new(
                                    generic::Inner::new(Some(new_keymap.clone())),
                                ));

                                config.write().unwrap().preferred_output_method =
                                    AvailableOutputMethod::Generic(Some(new_keymap.clone()));
                                config.write().unwrap().saved_generic_keymap = new_keymap.clone();
                                confy::store("miditoqwerty-rs", None, &(*config.read().unwrap()))
                                    .expect("Failed to store configuration");
                            }
                        }
                    });
                });
            }
        });
    })
}
