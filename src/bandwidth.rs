use crate::interface_stats;
use iced::widget::canvas::{self, Path, Frame, LineCap, Geometry};
use iced::{Color, Point, Theme, Rectangle, Element, Task, Size};
use iced::widget::{container, column, row, text, Canvas};
use std::time::Duration;

const BANDWIDTH_GRAPH_HEIGHT: f64 = 1000.0; // 1 Gbps scale for graph (values are now in Mbps)
const GRAPH_POINTS: usize = 80;

// Layout constants
const CANVAS_HEIGHT: f32 = 48.0;
const CONTAINER_HEIGHT: f32 = 48.0;
const PADDING_VERTICAL: f32 = 0.0;
const PADDING_HORIZONTAL: f32 = 10.0;
const BORDER_RADIUS: f32 = 10.0;
const LINE_WIDTH: f32 = 2.0;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    StatsUpdated(Option<interface_stats::NetworkStats>),
}

#[derive(Default)]
pub struct BandwidthGraph {
    upload_points: Vec<f64>,
    download_points: Vec<f64>,
    current_stats: Option<interface_stats::NetworkStats>,
}

impl BandwidthGraph {
    pub fn new() -> Self {
        Self {
            upload_points: vec![0.0; GRAPH_POINTS],
            download_points: vec![0.0; GRAPH_POINTS],
            current_stats: None,
        }
    }

    pub fn update_stats(&mut self, stats: interface_stats::NetworkStats) {
        // Convert bytes per second to megabits per second for graph
        const BYTES_TO_MEGABITS: f64 = 8.0 / 1_000_000.0;
        let upload_mbps = stats.upload_bps * BYTES_TO_MEGABITS;
        let download_mbps = stats.download_bps * BYTES_TO_MEGABITS;
        
        self.current_stats = Some(stats);
        
        // Efficient circular buffer - avoid remove(0) which is O(n)
        if self.upload_points.len() >= GRAPH_POINTS {
            self.upload_points.rotate_left(1);
            self.upload_points[GRAPH_POINTS - 1] = upload_mbps;
        } else {
            self.upload_points.push(upload_mbps);
        }
        
        if self.download_points.len() >= GRAPH_POINTS {
            self.download_points.rotate_left(1);
            self.download_points[GRAPH_POINTS - 1] = download_mbps;
        } else {
            self.download_points.push(download_mbps);
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // Start async network stats collection using Iced's Task system
                Task::perform(interface_stats::get_network_stats_async(), Message::StatsUpdated)
            }
            Message::StatsUpdated(stats) => {
                // Handle async result when it completes
                if let Some(stats) = stats {
                    self.update_stats(stats);
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let bandwidth_data = if let Some(stats) = &self.current_stats {
            let upload_str = interface_stats::format_rate(stats.upload_bps);
            let download_str = interface_stats::format_rate(stats.download_bps);
            
            column![
                text(format!("↑ {}", upload_str))
                    .size(12)
                    .style(|_theme| iced::widget::text::Style { 
                        color: Color::from_rgb(0.0, 0.5, 1.0).into() // Blue for upload
                    }),
                text(format!("↓ {}", download_str))
                    .size(12)
                    .style(|_theme| iced::widget::text::Style { 
                        color: Color::from_rgb(0.0, 1.0, 0.5).into() // Green for download
                    }),
            ]
            .spacing(4)
        } else {
            column![
                text("↑ --").size(12),
                text("↓ --").size(12),
            ]
            .spacing(4)
        };

        let bandwidth_graph_container = container(
            Canvas::new(self)
                .width(iced::Length::Fill)
                .height(iced::Length::Fixed(CANVAS_HEIGHT))
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(CANVAS_HEIGHT))
        .padding([0.0, PADDING_HORIZONTAL]) // top/bottom, left/right
        .align_x(iced::Alignment::Center);

        let bandwidth_data_container = container(bandwidth_data)
            .width(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .align_x(iced::Alignment::End)
            .padding([PADDING_VERTICAL, PADDING_HORIZONTAL])
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.0, 0.0, 0.0).into()),
                border: iced::border::Border {
                    radius: BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let inner_row = row![
            container(bandwidth_graph_container)
                .width(iced::Length::FillPortion(84)) // 75% for graph
                .height(iced::Length::Fixed(CONTAINER_HEIGHT))
                .align_x(iced::Alignment::Center)
                .style(|_theme| container::Style {
                    background: Some(Color::from_rgb(0.0, 0.0, 0.0).into()),
                    border: iced::border::Border {
                        radius: iced::border::Radius {
                            top_left: BORDER_RADIUS,
                            top_right: 0.0,
                            bottom_right: 0.0,
                            bottom_left: BORDER_RADIUS,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            container(bandwidth_data_container)
                .width(iced::Length::FillPortion(12)) // 25% for data
                .height(iced::Length::Fixed(CONTAINER_HEIGHT))
                .style(|_theme| container::Style {
                    background: Some(Color::from_rgb(0.0, 0.0, 0.0).into()),
                    border: iced::border::Border {
                        radius: iced::border::Radius {
                            top_left: 0.0,
                            top_right: BORDER_RADIUS,
                            bottom_right: BORDER_RADIUS,
                            bottom_left: 0.0,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(0)
        .width(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

        let bandwidth_container = container(inner_row)
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(54.0))
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgb(0.0, 0.0, 0.0).into()),
            border: iced::border::Border {
                radius: BORDER_RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        container(bandwidth_container)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .style(|_theme| container::Style {
                border: iced::border::Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(200)).map(|_| Message::Tick)
    }
}

impl canvas::Program<Message> for BandwidthGraph {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &Theme, bounds: Rectangle, _cursor: iced::mouse::Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        let width = bounds.width;
        let height = bounds.height;
        
        // Ensure we have valid dimensions - use minimum size if too small
        if width <= 1.0 || height <= 1.0 {
            return vec![];
        }
        
        // Draw a background to make canvas visible
        let background = Path::rectangle(Point::new(0.0, 0.0), Size::new(width, height));
        frame.fill(&background, Color::from_rgb(0.0, 0.0, 0.0));
        
        // Draw upload line (blue)
        self.draw_line(&self.upload_points, &mut frame, Color::from_rgb(0.0, 0.5, 1.0), width, height);
        
        // Draw download line (green)
        self.draw_line(&self.download_points, &mut frame, Color::from_rgb(0.0, 1.0, 0.5), width, height);
        
        vec![frame.into_geometry()]
    }
}

impl BandwidthGraph {
    fn draw_line(&self, points: &[f64], frame: &mut Frame, color: Color, width: f32, height: f32) {
        if points.len() < 2 || width <= 0.0 || height <= 0.0 {
            return;
        }

        // Use the full width of the canvas
        let x_step = width / (points.len() - 1) as f32;
        
        // Convert points to screen coordinates - both lines start at bottom (y = height)
        let screen_points: Vec<Point> = points.iter().enumerate().map(|(i, &value)| {
            let x = i as f32 * x_step;
            let normalized_value = (value / BANDWIDTH_GRAPH_HEIGHT).min(1.0).max(0.0);
            // Both lines start at bottom (height) and go up from there
            let y = height - (normalized_value as f32 * height * 0.9); // Use 90% of height to leave some margin at top
            Point::new(x, y)
        }).collect();

        let path = Path::new(|builder| {
            if let Some(&first_point) = screen_points.first() {
                // Start at the first point
                builder.move_to(first_point);

                // Draw smooth lines using quadratic_curve_to as specified in migration plan
                for i in 0..screen_points.len() - 1 {
                    let p1 = screen_points[i];
                    let p2 = screen_points[i + 1];
                    
                    let mid_point = Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
                    
                    if i == 0 {
                        // For the first segment, use a quadratic curve
                        builder.quadratic_curve_to(p1, mid_point);
                    } else if i == screen_points.len() - 2 {
                        // For the last segment, use a quadratic curve
                        builder.quadratic_curve_to(p2, p2);
                    } else {
                        // For middle segments, use quadratic curve to midpoint
                        builder.quadratic_curve_to(p1, mid_point);
                    }
                }
            }
        });

        let stroke = canvas::Stroke {
            width: LINE_WIDTH,
            style: canvas::Style::Solid(color),
            line_cap: LineCap::Round,
            ..Default::default()
        };

        frame.stroke(&path, stroke);
    }
}
