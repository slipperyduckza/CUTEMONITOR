use iced::widget::canvas;

// This module contains canvas drawing programs for creating custom charts and graphs
// Canvas programs in Iced allow us to draw directly to the screen using a 2D graphics API

/// A program that draws a bar chart showing historical CPU usage data
/// Each bar represents a past measurement, with height proportional to CPU usage
#[derive(Debug)]
pub struct BarChartProgram {
    /// Vector of historical CPU usage percentages (0.0 to 100.0)
    pub history: Vec<f32>,
}



// Implement the canvas drawing program for the bar chart
impl<Message> canvas::Program<Message> for BarChartProgram {
    type State = (); // No state needed for this simple drawing

    // This function is called to draw the bars on the canvas
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // Create a drawing frame with the size of the canvas area
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let bar_width = 0.4;
        let spacing = 0.5;
        let total_width_needed = self.history.len() as f32 * spacing;
        let scale_x = bounds.width / total_width_needed;

        for (i, &usage) in self.history.iter().enumerate() {
            let x = i as f32 * spacing * scale_x;
            let bar_height = (usage / 100.0) * bounds.height;
            let y = bounds.height - bar_height;

            // Draw bar with color similar to PROTOTYPE
            frame.fill_rectangle(
                iced::Point::new(x, y),
                iced::Size::new(bar_width * scale_x, bar_height),
                iced::Color::from_rgb(123.0 / 255.0, 104.0 / 255.0, 238.0 / 255.0), // Medium slate blue
            );

            // Draw stroke
            frame.stroke(
                &canvas::Path::rectangle(
                    iced::Point::new(x, y),
                    iced::Size::new(bar_width * scale_x, bar_height),
                ),
                canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(
                        25.0 / 255.0,
                        25.0 / 255.0,
                        112.0 / 255.0,
                    ))
                    .with_width(0.5),
            );
        }

        // Return the drawn frame as geometry for rendering
        vec![frame.into_geometry()]
    }
}

/// A program that draws overlaid bars showing current, previous, and oldest CPU usage
/// The bars are stacked vertically with different colors and transparency
#[derive(Debug)]
pub struct OverlayBarProgram {
    /// Current CPU usage percentage
    pub current: f32,
    /// Previous CPU usage percentage
    pub previous: f32,
    /// Oldest CPU usage percentage in history
    pub oldest: f32,
}

/// Implementation of the Canvas Program trait for drawing overlaid bars
impl<Message> canvas::Program<Message> for OverlayBarProgram {
    type State = (); // No internal state needed

    /// Draw function that creates the overlaid bar visualization
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // Create drawing frame
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw bars from back to front (oldest to newest)
        // Oldest bar - dark color, drawn first (appears at bottom)
        if self.oldest > 0.0 {
            let width = bounds.width * self.oldest / 100.0; // Width based on usage %
            let height = bounds.height - 15.0; // Slightly shorter than full height
            let y = bounds.height - height; // Position from bottom
            frame.fill_rectangle(
                iced::Point::new(0.0, y),
                iced::Size::new(width, height),
                iced::Color::from_rgba(0.1, 0.1, 0.3, 0.8), // Dark blue with transparency
            );
        }

        // Previous bar - medium color, drawn second
        if self.previous > 0.0 {
            let width = bounds.width * self.previous / 100.0;
            let height = bounds.height - 8.0; // Medium height
            let y = bounds.height - height;
            frame.fill_rectangle(
                iced::Point::new(0.0, y),
                iced::Size::new(width, height),
                iced::Color::from_rgba(0.3, 0.3, 0.6, 0.65), // Grey-blue with transparency
            );
        }

        // Current bar - bright color, drawn last (appears on top)
        let current_width = bounds.width * self.current / 100.0;
        let height = bounds.height - 1.0; // Almost full height
        let y = bounds.height - height;
        frame.fill_rectangle(
            iced::Point::new(0.0, y),
            iced::Size::new(current_width, height),
            iced::Color::from_rgba(0.1, 0.1, 1.0, 1.0), // Bright blue, fully opaque
        );

        // Return the completed drawing
        vec![frame.into_geometry()]
    }
}