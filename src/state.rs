use crate::hardware_checker;
// This module manages the application's state and UI updates
// It handles messages from subscriptions and updates the display accordingly
// The State struct holds all the data needed to render the UI

use crate::what_cpu_check;

/// Messages that can be sent to update the application state
/// Messages that can be sent to update the application state
/// These are processed in the update() function to modify the UI
#[derive(Debug, Clone)]
pub enum Message {
    /// Update hardware data (temperatures, voltages, etc.)
    UpdateData(hardware_checker::HardwareData),
    /// Update per-core CPU usage percentages
    UpdateCores(Vec<f32>),
    /// Update per-thread CPU usage percentages
    UpdateThreads(Vec<f32>),
    /// Update the list of top user processes
    UpdateProcesses(Vec<what_cpu_check::ProcessInfo>),
    /// Update GPU information
    UpdateGpu(hardware_checker::GpuData),
    /// Handle window resize events
    WindowResized((f32, f32)),
    /// Handle other window events
    WindowEvent(iced::window::Event),
}

/// The main application state containing all data needed for the UI
/// This struct holds current hardware readings, CPU usage history, and process information
pub struct State {
    pub motherboard_model: String,
    pub cpu_temp: f32,
    pub ccd_temperatures: Vec<Option<f32>>,
    pub cpu_voltage: Option<f32>,
    pub cpu_power: Option<f32>,
    pub chipset_temp: Option<f32>,
    pub memory_usage: f32,
    pub total_memory_mb: i32,
    pub used_memory_mb: f32,
    pub memory_speed_mts: i32,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub is_vm: bool,
    pub core_usages: Vec<Vec<f32>>,
    pub thread_usages: Vec<Vec<f32>>,
    pub total_usages: Vec<f32>,
    pub top_processes: Vec<what_cpu_check::ProcessInfo>,
    pub gpu_data: hardware_checker::GpuData,
    pub window_size: (f32, f32),
    pub window_position: Option<(i32, i32)>,
}

/// Implementation of the Default trait to create initial application state
impl Default for State {
    fn default() -> Self {
        // Get CPU information at startup
        let cpu_info = what_cpu_check::get_cpu_info();
        let is_vm = what_cpu_check::is_virtual_machine();

        Self {
            // Initialize hardware data as empty/zero (will be filled by subscriptions)
            motherboard_model: String::new(),
            cpu_temp: 0.0,
            ccd_temperatures: Vec::new(),
            cpu_voltage: None,
            cpu_power: None,
            chipset_temp: None,
            memory_usage: 0.0,
            total_memory_mb: 0,
            used_memory_mb: 0.0,
            memory_speed_mts: 0,

            // CPU info from system detection
            cpu_model: cpu_info.model,
            cpu_cores: cpu_info.cores,
            cpu_threads: cpu_info.threads,
            is_vm,

            // Initialize usage history buffers
            // Each core gets its own history buffer, pre-filled with 10% usage
            core_usages: vec![vec![10.0; crate::HISTORY_SIZE]; cpu_info.cores],
            // Each thread gets its own history buffer, initialized to 0%
            thread_usages: vec![vec![0.0; crate::HISTORY_SIZE]; cpu_info.threads],
            // Total CPU usage history, initialized to 0%
            total_usages: vec![0.0; crate::HISTORY_SIZE],

            // Process monitoring starts empty
            top_processes: Vec::new(),

            // GPU data starts empty
            gpu_data: hardware_checker::GpuData {
                model: String::new(),
                vram_mb: 0,
                temp: None,
                utilization: None,
                memory_usage: None,
                encoder: None,
                decoder: None,
            },

            // Default window size
            window_size: (800.0, 600.0),
            window_position: None,
        }
    }
}

impl State {
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::UpdateData(data) => {
                self.motherboard_model = data.motherboard_model;
                self.cpu_temp = data.cpu_temp;
                self.ccd_temperatures = data.ccd_temperatures;
                self.cpu_voltage = data.cpu_voltage;
                self.cpu_power = data.cpu_power;
                self.chipset_temp = data.chipset_temp;
                self.memory_usage = data.memory_usage;
                self.total_memory_mb = data.total_memory_mb;
                self.memory_speed_mts = data.memory_speed_mts;
                self.used_memory_mb = (data.memory_usage / 100.0) * data.total_memory_mb as f32;
                iced::Task::none()
            }
            Message::UpdateCores(core) => {
                for (i, &usage) in core.iter().enumerate() {
                    self.core_usages[i].insert(0, usage);
                    self.core_usages[i].truncate(crate::HISTORY_SIZE);
                }
                // Calculate and update total CPU usage
                let total: f32 = core.iter().sum();
                let avg_total = total / core.len() as f32;
                self.total_usages.insert(0, avg_total);
                self.total_usages.truncate(crate::HISTORY_SIZE);
                iced::Task::none()
            }
            // Update CPU thread usage data
            Message::UpdateThreads(thread) => {
                // Update usage history for each thread
                for (i, &usage) in thread.iter().enumerate() {
                    self.thread_usages[i].insert(0, usage); // Add new reading
                    self.thread_usages[i].truncate(crate::HISTORY_SIZE); // Maintain history size
                }
                iced::Task::none()
            }

            // Update process monitoring data
            Message::UpdateProcesses(processes) => {
                self.top_processes = processes;
                iced::Task::none()
            }

            // Update GPU monitoring data
            Message::UpdateGpu(data) => {
                self.gpu_data = data; // Store the new GPU data
                iced::Task::none()
            }

            // Handle window resize events
            Message::WindowResized(size) => {
                self.window_size = size; // Update stored window size
                iced::Task::none()
            }
            Message::WindowEvent(event) => {
                if let iced::window::Event::Moved(point) = event {
                    let x = point.x as i32;
                    let y = point.y as i32;
                    self.window_position = Some((x, y));
                    // Save to registry
                    crate::utils::save_window_position(x, y);
                }
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        use iced::widget::{canvas, column, container, image, row, text};
        use iced::Length;

        let mut elements = vec![
            row![
                text("Motherboard:").size(13),
                container(text(self.motherboard_model.to_string()).size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
            row![
                text("CPU Temperature:").size(13),
                container(
                    text(format!("{:.1}°C", self.cpu_temp))
                        .size(13)
                        .color(crate::data_colouring::temperature_color(self.cpu_temp))
                )
                .align_x(iced::alignment::Horizontal::Right)
                .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        ];

        for (i, &temp) in self.ccd_temperatures.iter().enumerate() {
            if let Some(t) = temp {
                elements.push(
                    row![
                        text(format!("CCD{} Temperature:", i + 1)).size(13),
                        container(text(format!("{:.1}°C", t)).size(13))
                            .align_x(iced::alignment::Horizontal::Right)
                            .width(Length::Fill)
                    ]
                    .width(Length::Fill)
                    .into(),
                );
            }
        }

        let cpu_voltage_text = if let Some(voltage) = self.cpu_voltage {
            text(format!("{:.3} V", voltage))
                .size(13)
                .color(crate::data_colouring::voltage_color(voltage))
        } else {
            text("N/A").size(13)
        };
        elements.push(
            row![
                text("CPU Voltage:").size(13),
                container(cpu_voltage_text)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        );

        let cpu_power_text = if let Some(power) = self.cpu_power {
            text(format!("{:.1} W", power))
                .size(13)
                .color(crate::data_colouring::power_color(power))
        } else {
            text("N/A").size(13)
        };
        elements.push(
            row![
                text("CPU Power:").size(13),
                container(cpu_power_text)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        );

        let chipset_temp_text = if let Some(temp) = self.chipset_temp {
            text(format!("{:.1}°C", temp))
                .size(13)
                .color(crate::data_colouring::temperature_color(temp))
        } else {
            text("N/A").size(13)
        };
        elements.push(
            row![
                text("Chipset Temperature:").size(13),
                container(chipset_temp_text)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        );

        elements.push(
            row![
                text("Memory Utilized:").size(13),
                container(
                    text(format!("{:.1}%", self.memory_usage))
                        .size(13)
                        .color(crate::data_colouring::memory_color(self.memory_usage))
                )
                .align_x(iced::alignment::Horizontal::Right)
                .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        );

        elements.push(
            row![
                text("Memory Usage:").size(13),
                container(
                    text(format!("{:.0} MB", self.used_memory_mb))
                        .size(13)
                )
                .align_x(iced::alignment::Horizontal::Right)
                .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        );

        elements.push(
            row![
                text("Total Memory:").size(13),
                container(text(format!("{} MB", self.total_memory_mb)).size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        );

        elements.push(
            row![
                text("Memory Speed:").size(13),
                container(text(format!("{} MT/s", self.memory_speed_mts)).size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ]
            .width(Length::Fill)
            .into(),
        );

        let hardware_info = container(
            column![
                text("HARDWARE INFORMATION").size(17),
                column(elements).spacing(1)
            ]
            .spacing(5),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.3, 0.3, 0.3,
            ))),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)
        .width(Length::FillPortion(50))
        .height(Length::Fill);

        let logo = image::Image::new(crate::Handle::from_bytes(if self.is_vm {
            crate::VM_LOGO
        } else if self.cpu_model.to_lowercase().contains("amd") {
            crate::AMD_LOGO
        } else {
            crate::INTEL_LOGO
        }))
        .width(128)
        .height(128);

        let logo_container = container(
            container(logo)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.3, 0.3, 0.3,
            ))),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)
        .width(Length::FillPortion(20))
        .height(Length::Fill);

        let model_container = container(
            container(
                column![
                    text(format!("CPU Model: {}", self.cpu_model)).size(13),
                    text(format!("CPU Cores: {}", self.cpu_cores)).size(13),
                    text(format!("CPU Threads: {}", self.cpu_threads)).size(13),
                ]
                .spacing(1),
            )
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                50.0 / 255.0,
                50.0 / 255.0,
                50.0 / 255.0,
            ))),
            border: iced::Border {
                radius: 0.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)
        .width(Length::FillPortion(30))
        .height(Length::Fixed(100.0));

        let hardware_container = hardware_info;

        let top_container = container(row![logo_container, model_container, hardware_container])
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    50.0 / 255.0,
                    50.0 / 255.0,
                    50.0 / 255.0,
                ))),
                border: iced::Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding(6)
            .height(Length::Fixed(200.0));

        // Create the CPU cores section
        let mut elements = vec![text("CPU CORES").size(13).into()];
        for i in 0..self.cpu_cores {
            // Get usage history for this core
            let history = self.core_usages[i].clone();
            // Create row with label and chart
            let label = container(text(format!("Core {}", i)).size(13)).width(Length::Fixed(60.0)).align_x(iced::alignment::Horizontal::Left);
            let chart = container(
                canvas::Canvas::new(crate::canvas::BarChartProgram { history })
                    .width(Length::Fill)
                    .height(Length::Fixed(crate::BAR_HEIGHT)),
            )
            .style(crate::styles::black_border);
            let row = row![label, chart].spacing(10).align_y(iced::Alignment::End);
            elements.push(row.into());
        }
        let cores_column_inner = column(elements).spacing(1.0);

        let graph_core_container = container(cores_column_inner)
            .style(crate::styles::black_filled_box)
            .padding(10)
            .width(Length::FillPortion(65));

        // Create the CPU threads section
        let threads_per_core = self.cpu_threads / self.cpu_cores;
        let mut threads_elements = vec![text("CPU THREADS").size(13).into()];
        for i in 0..self.cpu_cores {
            let mut thread_row = row![];
            for j in 0..threads_per_core {
                let idx = i * threads_per_core + j;
                let current = self.thread_usages[idx][0];
                let previous = self.thread_usages[idx].get(1).copied().unwrap_or(0.0);
                let oldest = self.thread_usages[idx].get(2).copied().unwrap_or(0.0);
                thread_row = thread_row.push(
                    container(
                        canvas::Canvas::new(crate::canvas::OverlayBarProgram {
                            current,
                            previous,
                            oldest,
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(crate::BAR_HEIGHT)),
                    )
                    .style(crate::styles::black_border),
                );
            }
            threads_elements.push(thread_row.into());
        }
        let threads_column_inner = column(threads_elements).spacing(1.0);
        let graph_threads_container = container(threads_column_inner)
            .style(crate::styles::black_filled_box)
            .padding(10)
            .width(Length::FillPortion(35));

        // Create the total CPU usage section
        let total_text = text("Total").size(13).width(Length::FillPortion(4));
        let current = self.total_usages[0];
        let previous = self.total_usages.get(1).copied().unwrap_or(0.0);
        let oldest = self.total_usages.get(2).copied().unwrap_or(0.0);
        let total_graph = container(
            canvas::Canvas::new(crate::canvas::OverlayBarProgram {
                current,
                previous,
                oldest,
            })
            .width(Length::Fill)
            .height(Length::Fixed(crate::BAR_HEIGHT)),
        )
        .style(crate::styles::black_border)
        .width(Length::FillPortion(90));
        let total_percentage = container(text(format!("{:.1}%", current)).size(13))
            .align_x(iced::alignment::Horizontal::Right)
            .width(Length::FillPortion(6));
        let total_row = row![total_text, total_graph, total_percentage]
            .spacing(10)
            .align_y(iced::Alignment::Center);
        let graph_total_container = container(total_row)
            .style(crate::styles::black_filled_box)
            .padding(10)
            .width(Length::Fill);

        let mid_container = container(
            column![
                row![graph_core_container, graph_threads_container].spacing(0),
                graph_total_container
            ]
            .spacing(10),
        )
        .padding(6)
        .width(Length::Fill)
        .height(Length::Shrink)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                50.0 / 255.0,
                50.0 / 255.0,
                50.0 / 255.0,
            ))),
            border: iced::Border {
                radius: 0.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let top_processes: Vec<String> = self.top_processes.iter().take(4).map(|p| p.name.clone()).collect();

        let mut process_columns = vec![];
        for (i, name) in top_processes.iter().enumerate() {
            let label = format!("{}. {}", i + 1, name);
            let col = container(
                text(label)
                    .size(16)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(Length::FillPortion(1))
            .align_x(iced::alignment::Horizontal::Center);
            process_columns.push(col.into());
        }
        while process_columns.len() < 3 {
            process_columns.push(
                container(text("").size(16))
                    .width(Length::FillPortion(1))
                    .into(),
            );
        }
        let userprocess_container = container(
            column![
                text("TOP USER PROCESSES:").size(13),
                row(process_columns).spacing(10)
            ]
            .spacing(5),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.3, 0.3, 0.3,
            ))),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)
        .width(Length::Fill)
        .height(Length::Shrink);

        let bot_container = container(userprocess_container)
            .padding(6)
            .width(Length::Fill)
            .height(Length::Shrink)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    50.0 / 255.0,
                    50.0 / 255.0,
                    50.0 / 255.0,
                ))),
                border: iced::Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let gpu_logo = image::Image::new(crate::Handle::from_bytes(
            if self.gpu_data.model.to_lowercase().contains("nvidia") {
                crate::NVIDIA_LOGO
            } else if self.gpu_data.model.to_lowercase().contains("amd") {
                crate::AMD_GPU_LOGO
            } else if self.gpu_data.model.to_lowercase().contains("intel") {
                crate::INTEL_GPU_LOGO
            } else {
                crate::VM_LOGO
            },
        ))
        .width(128)
        .height(128);

        let gpu_logo_container = container(
            container(gpu_logo)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.3, 0.3, 0.3,
            ))),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)
        .width(Length::FillPortion(20))
        .height(Length::Fill);

        let gpu_model_container = container(
            container(
                column![
                    text(format!("GPU Model: {}", self.gpu_data.model)).size(13),
                    text(format!("VRAM: {} MB", self.gpu_data.vram_mb)).size(13),
                ]
                .spacing(1),
            )
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                50.0 / 255.0,
                50.0 / 255.0,
                50.0 / 255.0,
            ))),
            border: iced::Border {
                radius: 0.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)
        .width(Length::FillPortion(30))
        .height(Length::Fill)
        .align_y(iced::alignment::Vertical::Center)
        .padding(10);

        let gpu_util_row = if let Some(util) = self.gpu_data.utilization {
            row![
                text("GPU Utilization:").size(13),
                container(text(format!("{:.1}%", util)).size(13).color(crate::data_colouring::utilization_color(util)))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        } else {
            row![
                text("GPU Utilization:").size(13),
                container(text("N/A").size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        };
        let gpu_mem_row = if let Some(mem) = self.gpu_data.memory_usage {
            row![
                text("Memory Utilized:").size(13),
                container(text(format!("{:.1}%", mem)).size(13).color(crate::data_colouring::memory_color(mem)))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        } else {
            row![
                text("Memory Utilized:").size(13),
                container(text("N/A").size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        };
        let gpu_mem_usage_row = if let Some(mem) = self.gpu_data.memory_usage {
            let used_mb = (mem / 100.0) * self.gpu_data.vram_mb as f32;
            row![
                text("Memory Usage:").size(13),
                container(text(format!("{:.0} MB", used_mb)).size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        } else {
            row![
                text("Memory Usage:").size(13),
                container(text("N/A").size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        };
        let gpu_temp_row = if let Some(temp) = self.gpu_data.temp {
            row![
                text("Temperature:").size(13),
                container(text(format!("{:.1}°C", temp)).size(13).color(crate::data_colouring::temperature_color(temp)))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        } else {
            row![
                text("Temperature:").size(13),
                container(text("N/A").size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        };
        let gpu_encoder_row = if let Some(enc) = self.gpu_data.encoder {
            row![
                text("GPU Encoder:").size(13),
                container(text(format!("{:.1}%", enc)).size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        } else {
            row![
                text("GPU Encoder:").size(13),
                container(text("N/A").size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        };
        let gpu_decoder_row = if let Some(dec) = self.gpu_data.decoder {
            row![
                text("GPU Decoder:").size(13),
                container(text(format!("{:.1}%", dec)).size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        } else {
            row![
                text("GPU Decoder:").size(13),
                container(text("N/A").size(13))
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ].width(Length::Fill)
        };

        let gpu_monitor_container =
            container(
                column![
                    text("GPU INFORMATION").size(17),
                    column![gpu_util_row, gpu_mem_row, gpu_mem_usage_row, gpu_temp_row, gpu_encoder_row, gpu_decoder_row].spacing(1)
                ]
                .spacing(5)
            )
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.3, 0.3, 0.3,
                    ))),
                    border: iced::Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .padding(10)
                .width(Length::FillPortion(50))
                .height(Length::Shrink);

        let gfx_monitor_container = container(row![
            gpu_logo_container,
            gpu_model_container,
            gpu_monitor_container
        ])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                50.0 / 255.0,
                50.0 / 255.0,
                50.0 / 255.0,
            ))),
            border: iced::Border {
                radius: 0.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)
        .height(Length::Fixed(200.0));

        container(
            column![
                top_container,
                mid_container,
                bot_container,
                gfx_monitor_container
            ]
            .spacing(0),
        )
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            crate::hardware_checker::hardware_data_stream().map(Message::UpdateData),
            iced_futures::subscription::from_recipe(crate::subscriptions::CpuCoresMonitor),
            iced_futures::subscription::from_recipe(crate::subscriptions::CpuThreadsMonitor),
            iced_futures::subscription::from_recipe(crate::subscriptions::ProcessesMonitor),
            crate::hardware_checker::gpu_data_stream().map(Message::UpdateGpu),
            iced::window::resize_events()
                .map(|(_id, size)| Message::WindowResized((size.width, size.height))),
            iced::window::events().map(|(_id, event)| Message::WindowEvent(event)),
        ])
    }
}