use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ReqtAppState {
    // The current selected environment
    // iF none is selected, then the app's default environment is used
    current_env: String,

    // The current request being viewed
    current_req: Option<String>,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ReqtApp {
    // The current state of the app
    state: ReqtAppState,

    // TODO: REMOVE
    raw_url: String,

    // Whether or not the Environments window is showing
    show_env_window: bool,

    #[serde(skip)] // This how you opt-out of serialization of a field
    response_receiver: Receiver<ehttp::Result<ehttp::Response>>,

    #[serde(skip)]
    response_sender: Sender<ehttp::Result<ehttp::Response>>,

    #[serde(skip)]
    response: Option<String>,

    #[serde(skip)]
    ctx: Option<egui::Context>,
}

impl Default for ReqtAppState {
    fn default() -> Self {
        Self {
            current_env: "default".to_owned(),
            current_req: None,
        }
    }
}

impl Default for ReqtApp {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            state: ReqtAppState::default(),
            raw_url: "https://nekos.best/api/v2/hug?amount=10".to_owned(),
            show_env_window: false,
            response_sender: sender,
            response_receiver: receiver,
            response: None,
            ctx: None,
        }
    }
}

impl ReqtApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        Self {
            ctx: Some(cc.egui_ctx.clone()),
            ..Default::default()
        }
    }

    pub fn send_request(&self) {
        println!("send request!");
        let sender = self.response_sender.clone();
        // let url = format!("{}://{}", self.protocol, self.host);
        let url = self.raw_url.clone();
        println!("url: {}", url);
        let ctx = self.ctx.clone();
        let request = ehttp::Request::get(url);
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            // println!("Status code: {:?}", result.unwrap().status);
            sender.send(result).ok();
            if let Some(ctx) = ctx {
                println!("requesting repaint");
                ctx.request_repaint();
            }
        });
    }

    pub fn create_new_request(&self) {
        println!("TODO: create_new_request");
        // TODO: insert into requests
        // TODO: render new request in central panel
    }
}

impl eframe::App for ReqtApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        if let Ok(result) = self.response_receiver.try_recv() {
            match result {
                Ok(response) => {
                    // self.response = Some(format!("Status: {}", response.status));
                    // self.response = Some(format!("{:?}", response.text()));
                    // self.response = Some(format!("{}", response.json()));
                    match serde_json::from_slice::<serde_json::Value>(&response.bytes) {
                        Ok(json) => {
                            // Pretty print the JSON
                            self.response = Some(
                                serde_json::to_string_pretty(&json)
                                    .unwrap_or_else(|_| "Failed to format JSON".to_string()),
                            );
                        }
                        Err(e) => {
                            self.response = Some(format!("Failed to parse JSON: {}", e));
                        }
                    }
                }
                Err(err) => {
                    println!("got an error");
                    self.response = Some(format!("Error: {}", err));
                }
            }
        }

        // egui::SidePanel::left("side-panel-left-1").show(ctx, |ui| {
        //     // TODO: want this side panel to be smaller
        //     ui.label("Environments");
        // });

        egui::SidePanel::left("side-panel-left-2").show(ctx, |ui| {
            ui.label("Requests");

            if ui.button("New Request").clicked() {
                self.create_new_request();
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);

                if ui.button("Environments").clicked() {
                    self.show_env_window = true;
                }

                // ui.menu_button("Environments", |ui| {
                //     if ui.button("New Env").clicked() {
                //         println!("add new environment!");
                //         ui.close_menu();
                //     }
                // });
                ui.separator();
            })
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            match &self.state.current_req {
                Some(req) => {}
                None => {
                    ui.label("get reqt");
                    if ui.button("Create new Request").clicked() {
                        self.create_new_request();
                    };
                }
            }

            ui.horizontal(|ui| {
                ui.menu_button("GET", |ui| {
                    if ui.button("GET").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("POST").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("PUT").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("PATCH").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("DELETE").clicked() {
                        ui.close_menu();
                    }
                });

                // ui.menu_button(self.protocol.clone(), |ui| {
                //     if ui.button("https").clicked() {
                //         self.protocol = "https".to_owned();
                //         ui.close_menu();
                //     }
                //     if ui.button("http").clicked() {
                //         self.protocol = "http".to_owned();
                //         ui.close_menu();
                //     }
                // });
                // ui.label("://");
                //
                // ui.menu_button(self.host.clone(), |ui| {
                //     ui.label("select host:");
                //     if ui.button("nekos.best/api/v2/hug?amount=3").clicked() {
                //         self.host = "nekos.best/api/v2/hug?amount=3".to_owned();
                //         ui.close_menu();
                //     }
                //     ui.menu_button("My sub-menu", |ui| {
                //         if ui.button("Close the menu").clicked() {
                //             ui.close_menu();
                //         }
                //     });
                // });
                let url_box = egui::TextEdit::singleline(&mut self.raw_url)
                    .desired_width(f32::INFINITY)
                    .min_size(egui::Vec2 { x: 100.0, y: 30.0 })
                    .margin(egui::Margin::symmetric(10.0, 15.0))
                    .font(egui::TextStyle::Monospace);

                ui.add(url_box);

                // ui.add(egui::TextEdit::singleline(&mut self.raw_url).desired_width(f32::INFINITY));
            });

            if ui.button("Send").clicked() {
                self.send_request();
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    egui::CollapsingHeader::new("Response headers")
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::Grid::new("response_headers")
                                .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                                .show(ui, |ui| {
                                    ui.label("fake header key");
                                    ui.label("fake header value");
                                    ui.end_row();
                                })
                        });

                    ui.separator();

                    if let Some(resp) = self.response.clone() {
                        ui.add(egui::Label::new(resp).selectable(true));
                    }
                });

            ui.separator();

            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     // powered_by_egui_and_eframe(ui);
            //     egui::warn_if_debug_build(ui);
            // });
        });

        if self.show_env_window {
            egui::Window::new("Environments")
                .open(&mut self.show_env_window.clone())
                .show(ctx, |ui| {
                    if ui.button("New Environment").clicked() {
                        println!("TODO: show new environment window");
                    }

                    if ui.button("Close").clicked() {
                        self.show_env_window = false;
                    }
                });
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
