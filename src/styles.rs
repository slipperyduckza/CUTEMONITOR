// This module contains styling functions for UI elements
// These functions create reusable styles for containers and other widgets

use iced::widget::container;

/// Creates a style for containers with a transparent background and black border
/// Used for the progress bars to give them a subtle outline
/// The _theme parameter is required by Iced but not used in this simple styling
pub fn black_border(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.0, 0.0, 0.0, 0.0, // RGBA: fully transparent
        ))),
        border: iced::Border {
            width: 1.0, // 1 pixel wide border
            color: iced::Color::BLACK, // Black border color
            radius: iced::border::Radius::default(), // Sharp corners
        },
        ..Default::default() // Use default values for other style properties
    }
}

/// Creates a style for containers with a black background and rounded corners
/// Used for the graph sections to create visually distinct areas
/// The _theme parameter is required by Iced but not used in this simple styling
pub fn black_filled_box(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(iced::Color::BLACK)), // Solid black background
        border: iced::Border {
            width: 0.0, // No border
            color: iced::Color::BLACK, // Not used since width is 0
            radius: iced::border::Radius::from(10.0), // 10 pixel corner radius
        },
        ..Default::default() // Use default values for other style properties
    }
}