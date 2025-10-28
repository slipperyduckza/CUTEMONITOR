#![windows_subsystem = "windows"]

use iced::advanced::image::Handle;
use iced::widget::{canvas, column, container, image, row, text};
use iced::{
    window::{icon, resize_events},
    Element, Length, Subscription,
};
use iced_futures::subscription::from_recipe;
use windows::core::PCSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR};

use std::io::Cursor;

use crate::canvas::{BarChartProgram, OverlayBarProgram};
use crate::state::{Message, State};
use crate::styles::{black_border, black_filled_box};
use crate::subscriptions::{CpuCoresMonitor, CpuThreadsMonitor, ProcessesMonitor};
use crate::utils::is_admin;

// Embedded logos
static AMD_LOGO: &[u8] = include_bytes!("../AMD256.png");
static INTEL_LOGO: &[u8] = include_bytes!("../INTEL256.png");
static VM_LOGO: &[u8] = include_bytes!("../VM_PC256.png");

// Embedded GPU logos
static NVIDIA_LOGO: &[u8] = include_bytes!("../Nvidia_GeForce_256.png");
static AMD_GPU_LOGO: &[u8] = include_bytes!("../AMD_Radeon_256.png");
static INTEL_GPU_LOGO: &[u8] = include_bytes!("../Intel_Arc_256.png");

mod canvas;
mod data_colouring;
mod hardware_checker;
mod state;
mod styles;
mod subscriptions;
mod utils;
mod what_cpu_check;

// Constants for easy configuration
const HISTORY_SIZE: usize = 30; // How many past CPU readings to keep
const BAR_HEIGHT: f32 = 24.0; // Height of each progress bar in pixels






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
                        .color(data_colouring::temperature_color(self.cpu_temp))
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
                .color(data_colouring::voltage_color(voltage))
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
                .color(data_colouring::power_color(power))
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
                .color(data_colouring::temperature_color(temp))
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
                text("Memory Usage:").size(13),
                container(
                    text(format!("{:.1}%", self.memory_usage))
                        .size(13)
                        .color(data_colouring::memory_color(self.memory_usage))
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

        let logo = image::Image::new(Handle::from_bytes(if self.is_vm {
            VM_LOGO
        } else if self.cpu_model.to_lowercase().contains("amd") {
            AMD_LOGO
        } else {
            INTEL_LOGO
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
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
        )
        .width(Length::FillPortion(30))
        .height(Length::Fill)
        .align_y(iced::alignment::Vertical::Center)
        .padding(10);

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
            let label = text(format!("Core {}", i)).size(13);
            let chart = container(
                canvas::Canvas::new(BarChartProgram { history })
                    .width(Length::Fill)
                    .height(Length::Fixed(BAR_HEIGHT)),
            )
            .style(black_border);
            let row = row![label, chart].spacing(10).align_y(iced::Alignment::End);
            elements.push(row.into());
        }
        let cores_column_inner = column(elements).spacing(1.0);

        let graph_core_container = container(cores_column_inner)
            .style(black_filled_box)
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
                        canvas::Canvas::new(OverlayBarProgram {
                            current,
                            previous,
                            oldest,
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(BAR_HEIGHT)),
                    )
                    .style(black_border),
                );
            }
            threads_elements.push(thread_row.into());
        }
        let threads_column_inner = column(threads_elements).spacing(1.0);
        let graph_threads_container = container(threads_column_inner)
            .style(black_filled_box)
            .padding(10)
            .width(Length::FillPortion(35));

        // Create the total CPU usage section
        let total_text = text("Total").size(13);
        let current = self.total_usages[0];
        let previous = self.total_usages.get(1).copied().unwrap_or(0.0);
        let oldest = self.total_usages.get(2).copied().unwrap_or(0.0);
        let total_graph = container(
            canvas::Canvas::new(OverlayBarProgram {
                current,
                previous,
                oldest,
            })
            .width(Length::Fill)
            .height(Length::Fixed(BAR_HEIGHT)),
        )
        .style(black_border);
        let total_percentage = container(text(format!("{:.1}%", current)).size(13))
            .align_x(iced::alignment::Horizontal::Right)
            .width(Length::Fill);
        let total_row = row![total_text, total_graph, total_percentage]
            .spacing(10)
            .align_y(iced::Alignment::Center);
        let graph_total_container = container(total_row)
            .style(black_filled_box)
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

        let mut averages: Vec<(String, f32)> = self
            .processes_history
            .iter()
            .filter_map(|(name, usages)| {
                if usages.is_empty() {
                    None
                } else {
                    let avg = usages.iter().sum::<f32>() / usages.len() as f32;
                    Some((name.clone(), avg))
                }
            })
            .collect();
        averages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top3 = averages.into_iter().take(3).collect::<Vec<_>>();

        let mut process_columns = vec![];
        for (i, (name, _)) in top3.iter().enumerate() {
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

        let gpu_logo = image::Image::new(Handle::from_bytes(
            if self.gpu_data.model.to_lowercase().contains("nvidia") {
                NVIDIA_LOGO
            } else if self.gpu_data.model.to_lowercase().contains("amd") {
                AMD_GPU_LOGO
            } else if self.gpu_data.model.to_lowercase().contains("intel") {
                INTEL_GPU_LOGO
            } else {
                VM_LOGO
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

        let gpu_util_text = if let Some(util) = self.gpu_data.utilization {
            text(format!("GPU Utilization: {:.1}%", util)).size(13)
        } else {
            text("GPU Utilization: N/A").size(13)
        };
        let gpu_mem_text = if let Some(mem) = self.gpu_data.memory_usage {
            text(format!("Memory Usage: {:.1}%", mem)).size(13)
        } else {
            text("Memory Usage: N/A").size(13)
        };
        let gpu_temp_text = if let Some(temp) = self.gpu_data.temp {
            text(format!("Temperature: {:.1}°C", temp)).size(13).color(data_colouring::temperature_color(temp))
        } else {
            text("Temperature: N/A").size(13)
        };
        let gpu_encoder_text = if let Some(enc) = self.gpu_data.encoder {
            text(format!("GPU Encoder: {:.1}%", enc)).size(13)
        } else {
            text("GPU Encoder: N/A").size(13)
        };
        let gpu_decoder_text = if let Some(dec) = self.gpu_data.decoder {
            text(format!("GPU Decoder: {:.1}%", dec)).size(13)
        } else {
            text("GPU Decoder: N/A").size(13)
        };

        let gpu_monitor_container =
            container(column![gpu_util_text, gpu_mem_text, gpu_temp_text, gpu_encoder_text, gpu_decoder_text].spacing(1))
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


}

pub fn main() -> iced::Result {
    if !is_admin() {
        unsafe {
            MessageBoxA(
                HWND::default(),
                PCSTR::from_raw(b"This program requires administrator privileges.\0".as_ptr()),
                PCSTR::from_raw(b"Administrator Required\0".as_ptr()),
                MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }

    let icon = {
        let bytes = include_bytes!("../cutemonitor.ico");
        let icon_dir = ico::IconDir::read(Cursor::new(bytes)).unwrap();
        let image = icon_dir.entries()[0].decode().unwrap();
        let rgba = image.rgba_data().to_vec();
        let width = image.width();
        let height = image.height();
        icon::from_rgba(rgba, width, height).unwrap()
    };

    iced::application("LibreHardware Prototype", State::update, State::view)
        .subscription(State::subscription)
        .window(iced::window::Settings {
            icon: Some(icon),
            size: (1120.0, 800.0).into(),
            ..Default::default()
        })
        .run()
}
