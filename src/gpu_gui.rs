// Import required Iced GUI framework components
use iced::widget::{container, image, row, text, column};  // GUI widgets
use iced::{Element, Length, Color};                        // Core GUI types
use crate::gpu_data::GpuData;                               // GPU data structure
use crate::data_colouring::{temperature_color, utilization_color, memory_color}; // Color utilities
use crate::gpu_assets::get_gpu_logo;                        // GPU logo loading
use crate::state::Message as AppStateMessage;               // Main app message type

/// Main GUI structure for the GPU Monitor application
/// 
/// This struct represents the main application state and holds all GPU data
/// that needs to be displayed in the user interface.
pub struct GpuMonitor {
    /// Legacy single GPU data (kept for backward compatibility)
    pub gpu_data: Option<GpuData>,
    /// List of all detected GPUs with their current metrics
    pub gpu_data_list: Vec<GpuData>,
}

// Default implementation for GpuMonitor
// This creates an empty monitor with no GPU data initially
impl Default for GpuMonitor {
    fn default() -> Self {
        Self {
            gpu_data: None,              // No single GPU data initially
            gpu_data_list: Vec::new(),   // Empty list of GPUs
        }
    }
}

impl GpuMonitor {
    /// Create a new GPU monitor instance
    /// 
    /// This method is marked as #[allow(dead_code)] because we typically
    /// use the Default implementation instead, but it's kept for completeness.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            gpu_data: None,              // No single GPU data initially
            gpu_data_list: Vec::new(),   // Empty list of GPUs
        }
    }

    /// Update the GPU data with new metrics from the monitoring system
    /// 
    /// This method is called by the main application when new GPU metrics
    /// are available from the hardware checker. It updates both the list
    /// and maintains backward compatibility with the single GPU field.
    /// 
    /// Arguments:
    /// - data_list: New list of GPU data with updated metrics
    pub fn update_multiple_gpu_data(&mut self, data_list: Vec<GpuData>) {
        self.gpu_data_list = data_list;
        
        // Also update the single GPU data for backward compatibility
        // This ensures older code that expects a single GPU still works
        if let Some(first_gpu) = self.gpu_data_list.first() {
            self.gpu_data = Some(first_gpu.clone());
        }
    }

    /// Creates GUI containers for all detected GPUs
    /// 
    /// This is the main method that builds the entire GPU monitoring interface.
    /// It creates a container for each GPU and arranges them vertically.
    /// 
    /// Returns:
    /// - An Iced Element containing all GPU containers
    /// - Shows loading state if no GPUs are detected yet
    pub fn create_all_gpu_containers(&self) -> Element<'_, AppStateMessage> {
        // Check if we have any GPU data yet
        if self.gpu_data_list.is_empty() {
            // Show loading state while GPU detection is in progress
            // This is better than showing an empty interface
            return self.create_loading_container();
        }

        // Create a container for each GPU in the system
        // enumerate() provides both the index and the GPU data
        // The index is used to number the GPUs (GPU 1, GPU 2, etc.)
        let gpu_containers: Vec<_> = self.gpu_data_list
            .iter()
            .enumerate()
            .map(|(index, gpu_data)| self.create_gpu_monitor_container_for_gpu(gpu_data, index))
            .collect();

        // Arrange all GPU containers in a vertical column with spacing
        column(gpu_containers).spacing(0).into()
    }



    /// Creates a loading container while GPU detection is in progress
    /// 
    /// This method creates a user-friendly loading screen that appears
    /// while the application is detecting GPUs in the background.
    /// It provides visual feedback that the application is working.
    /// 
    /// Returns:
    /// - An Iced Element showing a centered loading message
    fn create_loading_container(&self) -> Element<'_, AppStateMessage> {
        container(
            container(
                column![
                    // Main loading text with larger font
                    text("Detecting GPUs...").size(20),
                    // Secondary text with smaller font
                    text("Please wait while we scan your system").size(14)
                ]
                .spacing(10)  // Space between the two text elements
                .align_x(iced::alignment::Horizontal::Center)  // Center horizontally
            )
            .align_x(iced::alignment::Horizontal::Center)  // Center in container
            .align_y(iced::alignment::Vertical::Center)    // Center in container
            .width(Length::Fill)   // Take full width available
            .height(Length::Fill)   // Take full height available
        )
        // Style the container with dark theme and rounded corners
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                30.0 / 255.0,  // Dark gray background
                30.0 / 255.0,
                30.0 / 255.0,
            ))),
            border: iced::Border {
                radius: 10.0.into(),  // Rounded corners
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(20)  // Internal padding
        .width(Length::Fill)  // Take full width
        .height(Length::Fixed(200.0))  // Fixed height of 200 pixels
        .into()
    }

    /// Creates a complete GPU monitor container for a specific GPU
    /// 
    /// This method creates the main container for a single GPU that includes:
    /// - GPU logo on the left
    /// - GPU model information in the middle
    /// - Performance metrics on the right
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data to display
    /// - gpu_index: The index of this GPU (for numbering)
    /// 
    /// Returns:
    /// - An Iced Element containing the complete GPU monitor container
    fn create_gpu_monitor_container_for_gpu(&self, gpu_data: &GpuData, gpu_index: usize) -> Element<'_, AppStateMessage> {
        // Create the three main sections of the GPU container
        let gpu_logo_container = self.create_logo_container_for_gpu(gpu_data);
        let gpu_model_container = self.create_model_container_for_gpu(gpu_data, gpu_index);
        let gpu_monitor_container = self.create_monitoring_container_for_gpu(gpu_data);

        // Combine all three sections in a horizontal row
        let gfx_monitor_container = container(row![
            gpu_logo_container,      // Left: GPU logo
            gpu_model_container,      // Middle: Model info
            gpu_monitor_container     // Right: Performance metrics
        ])
        // Style the main container with dark theme
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                50.0 / 255.0,  // Medium gray background
                50.0 / 255.0,
                50.0 / 255.0,
            ))),
            border: iced::Border {
                radius: 0.0.into(),  // No rounded corners for main container
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(0)  // No padding around content
        .height(Length::Fixed(160.0));  // Reduced height for tighter layout

        gfx_monitor_container.into()
    }

    

    /// Creates the GPU logo display container for specific GPU data
    /// 
    /// This method creates the leftmost section of each GPU container,
    /// displaying the appropriate GPU logo (NVIDIA, AMD, Intel, or Virtual).
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data to get logo for
    /// 
    /// Returns:
    /// - An Iced Element containing the GPU logo
    fn create_logo_container_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        // Load the appropriate GPU logo based on GPU model name
        // get_gpu_logo() returns the correct logo bytes (NVIDIA, AMD, Intel, etc.)
        let gpu_logo = image::Image::new(iced::advanced::image::Handle::from_bytes(
            get_gpu_logo(&gpu_data.model).to_vec(),
        ))
        .width(128)   // Fixed width for logo
        .height(128);  // Fixed height for logo

        // Create a nested container structure for proper alignment and styling
        container(
            container(gpu_logo)
                .align_x(iced::alignment::Horizontal::Center)  // Center logo horizontally
                .align_y(iced::alignment::Vertical::Center)    // Center logo vertically
                .width(Length::Fill)   // Take full width of inner container
                .height(Length::Fixed(122.0)),  // Fixed height for logo area
        )
        // Style the logo container with dark background and rounded corners
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.3, 0.3, 0.3,  // Dark gray background for logo area
            ))),
            border: iced::Border {
                radius: 10.0.into(),  // Rounded corners
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)  // Internal padding
        .width(Length::FillPortion(20))  // Take 20% of total width
        .height(Length::Fixed(152.0))    // Fixed height for consistency
        .into()
    }

    

    /// Creates the GPU model information container for specific GPU data
    /// 
    /// This method creates the middle section of each GPU container,
    /// displaying basic GPU information like model name and VRAM.
    /// For virtual GPUs, it shows appropriate virtual GPU information.
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data to display information for
    /// - gpu_index: The index of this GPU (for numbering)
    /// 
    /// Returns:
    /// - An Iced Element containing GPU model information
    fn create_model_container_for_gpu(&self, gpu_data: &GpuData, gpu_index: usize) -> Element<'_, AppStateMessage> {
        // Create different content based on whether this is a virtual or physical GPU
        let content = if self.is_virtual_gpu(gpu_data) {
            // Virtual GPU content - shows limited information
            column![
                text(format!("GPU {}: {}", gpu_index + 1, gpu_data.model)).size(13),
                text("Virtual GPU").size(13),
                text("No performance metrics available").size(11),
            ]
            .spacing(1)  // Small spacing between lines
        } else {
            // Physical GPU content - shows model and VRAM
            column![
                text(format!("GPU {}: {}", gpu_index + 1, gpu_data.model)).size(13),
                text(format!("VRAM: {} MB", gpu_data.vram_mb)).size(13),
            ]
            .spacing(1)  // Small spacing between lines
        };

        // Create container for the model information
        container(content)
            .align_x(iced::alignment::Horizontal::Left)  // Align text to left
            .align_y(iced::alignment::Vertical::Top)     // Align to top
            .width(Length::Fill)   // Take full width
            .height(Length::Fill)  // Take full height
        // Style the container to match overall theme
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                50.0 / 255.0,  // Medium gray background
                50.0 / 255.0,
                50.0 / 255.0,
            ))),
            border: iced::Border {
                radius: 0.0.into(),  // No rounded corners
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(6)  // Internal padding
        .width(Length::FillPortion(30))  // Take 30% of total width
        .height(Length::Fixed(152.0))    // Fixed height for consistency
        .align_y(iced::alignment::Vertical::Center)  // Center vertically
        .padding(10)  // Additional padding for text
        .into()
    }

    

    /// Creates the monitoring data container for specific GPU data
    /// 
    /// This method creates the rightmost section of each GPU container,
    /// which displays real-time performance metrics. The content differs
    /// significantly between physical and virtual GPUs.
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data to create monitoring container for
    /// 
    /// Returns:
    /// - An Iced Element containing appropriate monitoring information
    fn create_monitoring_container_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        // Route to appropriate container based on GPU type
        if self.is_virtual_gpu(gpu_data) {
            // Virtual GPUs have limited monitoring capabilities
            self.create_virtual_gpu_monitoring_container(gpu_data)
        } else {
            // Physical GPUs show full performance metrics
            self.create_physical_gpu_monitoring_container(gpu_data)
        }
    }

    /// Creates monitoring container for physical GPUs with performance metrics
    /// 
    /// This method creates a comprehensive monitoring display for physical GPUs,
    /// showing all available performance metrics with color-coded values.
    /// Each metric is displayed as a labeled row with appropriate formatting.
    /// 
    /// Arguments:
    /// - gpu_data: The physical GPU data to display metrics for
    /// 
    /// Returns:
    /// - An Iced Element containing all performance metrics
    fn create_physical_gpu_monitoring_container(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        // Create individual metric rows
        let gpu_util_row = self.create_utilization_row_for_gpu(gpu_data);        // GPU utilization %
        let gpu_mem_row = self.create_memory_utilization_row_for_gpu(gpu_data);   // Memory utilization %
        let gpu_mem_usage_row = self.create_memory_used_row_for_gpu(gpu_data);    // Memory used in MB
        let gpu_temp_row = self.create_temperature_row_for_gpu(gpu_data);          // Temperature in °C
        let gpu_encoder_row = self.create_encoder_row_for_gpu(gpu_data);          // Video encoder %
        let gpu_decoder_row = self.create_decoder_row_for_gpu(gpu_data);          // Video decoder %

        // Create the main container with title and all metric rows
        container(
            column![
                text("GPU INFORMATION").size(17),  // Section title
                column![gpu_util_row, gpu_mem_row, gpu_mem_usage_row, gpu_temp_row, gpu_encoder_row, gpu_decoder_row]
                    .spacing(1)  // Small spacing between rows
            ]
            .spacing(5)  // Spacing between title and metrics
        )
        // Style the monitoring container
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.3, 0.3, 0.3,  // Dark gray background
            ))),
            border: iced::Border {
                radius: 10.0.into(),  // Rounded corners
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(10)  // Internal padding
        .width(Length::FillPortion(50))  // Take 50% of total width
        .height(Length::Fixed(160.0))    // Fixed height for consistency
        .into()
    }

    /// Creates monitoring container for virtual GPUs with static information
    /// 
    /// Virtual GPUs don't provide performance metrics, so this container
    /// shows basic information about the virtual GPU type, name, and driver.
    /// This helps users understand what kind of virtual GPU they're using.
    /// 
    /// Arguments:
    /// - gpu_data: The virtual GPU data to display information for
    /// 
    /// Returns:
    /// - An Iced Element containing virtual GPU information
    fn create_virtual_gpu_monitoring_container(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        // Create information rows for virtual GPU
        let gpu_type_row = self.create_virtual_gpu_type_row(gpu_data);    // Type of virtual GPU
        let gpu_name_row = self.create_virtual_gpu_name_row(gpu_data);      // Virtual GPU name
        let gpu_driver_row = self.create_virtual_gpu_driver_row(gpu_data);  // Driver version

        // Create container with title and information rows
        container(
            column![
                text("VIRTUAL GPU INFORMATION").size(17),  // Section title
                column![gpu_type_row, gpu_name_row, gpu_driver_row]
                    .spacing(1)  // Small spacing between rows
            ]
            .spacing(5)  // Spacing between title and information
        )
        // Style the container to match physical GPU container
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.3, 0.3, 0.3,  // Dark gray background
            ))),
            border: iced::Border {
                radius: 10.0.into(),  // Rounded corners
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(10)  // Internal padding
        .width(Length::FillPortion(50))  // Take 50% of total width
        .height(Length::Fixed(160.0))    // Fixed height for consistency
        .into()
    }

    /// Helper function to detect if GPU is virtual
    /// 
    /// This function checks the GPU model name for keywords that indicate
    /// it's a virtual GPU rather than a physical one. Virtual GPUs
    /// don't provide performance metrics and need different UI treatment.
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data to check
    /// 
    /// Returns:
    /// - true if the GPU is virtual, false if physical
    fn is_virtual_gpu(&self, gpu_data: &GpuData) -> bool {
        let model_lower = gpu_data.model.to_lowercase();
        
        // Check for common virtualization platform keywords
        model_lower.contains("hyper-v") ||        // Microsoft Hyper-V
        model_lower.contains("vmware") ||          // VMware products
        model_lower.contains("virtualbox") ||       // Oracle VirtualBox
        model_lower.contains("qemu") ||           // QEMU/KVM virtualization
        model_lower.contains("virtual") ||         // Generic virtual GPU
        model_lower.contains("remote display")     // Remote desktop adapters
    }

    /// Creates virtual GPU type row
    /// 
    /// This method identifies the specific type of virtual GPU based on
    /// the model name and creates a descriptive label. This helps users
    /// understand what virtualization platform they're using.
    /// 
    /// Arguments:
    /// - gpu_data: The virtual GPU data
    /// 
    /// Returns:
    /// - An Iced Element showing the virtual GPU type
    fn create_virtual_gpu_type_row(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        // Determine the specific virtual GPU type based on model name
        let gpu_type = if gpu_data.model.to_lowercase().contains("hyper-v") {
            "Hyper-V Virtual GPU"
        } else if gpu_data.model.to_lowercase().contains("vmware") {
            "VMware Virtual GPU"
        } else if gpu_data.model.to_lowercase().contains("virtualbox") {
            "VirtualBox Virtual GPU"
        } else if gpu_data.model.to_lowercase().contains("qemu") {
            "QEMU/KVM Virtual GPU"
        } else if gpu_data.model.to_lowercase().contains("remote display") {
            "Remote Display Adapter"
        } else {
            "Virtual GPU"  // Generic fallback
        };
        
        // Create a labeled value row for the GPU type
        self.create_value_row("Type: ", gpu_type.to_string(), None)
    }

    /// Creates virtual GPU name row
    /// 
    /// Displays the full model name of the virtual GPU.
    /// This helps identify the specific virtual GPU implementation.
    /// 
    /// Arguments:
    /// - gpu_data: The virtual GPU data
    /// 
    /// Returns:
    /// - An Iced Element showing the virtual GPU name
    fn create_virtual_gpu_name_row(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        self.create_value_row("Name: ", gpu_data.model.clone(), None)
    }

    /// Creates virtual GPU driver version row
    /// 
    /// Displays the driver version for the virtual GPU.
    /// This can be useful for troubleshooting compatibility issues.
    /// 
    /// Arguments:
    /// - gpu_data: The virtual GPU data
    /// 
    /// Returns:
    /// - An Iced Element showing the driver version
    fn create_virtual_gpu_driver_row(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        self.create_value_row("Driver Version: ", gpu_data.driver_version.clone(), None)
    }

    /// Helper function to create labeled value rows
    /// 
    /// This is a utility function that creates consistent-looking rows
    /// throughout the interface. Each row has a label on the left
    /// and a value on the right, with optional color coding.
    /// 
    /// Arguments:
    /// - label: The text label (e.g., "GPU Utilization:")
    /// - value: The value to display (e.g., "75.3%")
    /// - color: Optional color for the value text
    /// 
    /// Returns:
    /// - An Iced Element containing the formatted row
    fn create_value_row(&self, label: &str, value: String, color: Option<Color>) -> Element<'_, AppStateMessage> {
        let label_owned = label.to_string();  // Convert &str to String for ownership
        let label_text = text(label_owned).size(13);  // Create label text with consistent font size
        let value_text = text(value).size(13);        // Create value text with consistent font size
        
        // Apply color to value if provided (used for color-coded metrics)
        let colored_text = if let Some(c) = color {
            value_text.color(c)
        } else {
            value_text  // Use default color if none provided
        };

        // Create a row with label on left and value on right
        row![
            label_text,  // Left-aligned label
            container(colored_text)
                .align_x(iced::alignment::Horizontal::Right)  // Right-align value
                .width(Length::Fill)  // Take remaining space
        ].width(Length::Fill)  // Take full width of container
        .into()
    }

    

    /// Creates GPU utilization row with color coding
    /// 
    /// Displays the current GPU utilization percentage with color coding:
    /// - Green: Low utilization (0-50%)
    /// - Yellow: Medium utilization (50-80%)
    /// - Red: High utilization (80-100%)
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data containing utilization information
    /// 
    /// Returns:
    /// - An Iced Element showing GPU utilization with appropriate color
    fn create_utilization_row_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        if let Some(util) = gpu_data.utilization {
            // Format utilization to 1 decimal place and apply color coding
            self.create_value_row(
                "GPU Utilization:",
                format!("{:.1}%", util),
                Some(utilization_color(util))  // Color based on utilization level
            )
        } else {
            // Show "N/A" if utilization data is not available
            self.create_value_row("GPU Utilization:", "N/A".to_string(), None)
        }
    }

    /// Creates memory utilization row with color coding
    /// 
    /// Displays the current VRAM utilization percentage with color coding.
    /// Memory usage is important for GPU performance monitoring.
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data containing memory utilization information
    /// 
    /// Returns:
    /// - An Iced Element showing memory utilization with appropriate color
    fn create_memory_utilization_row_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        if let Some(mem) = gpu_data.memory_usage {
            // Format memory utilization to 1 decimal place and apply color coding
            self.create_value_row(
                "Memory Utilized:",
                format!("{:.1}%", mem),
                Some(memory_color(mem))  // Color based on memory usage level
            )
        } else {
            // Show "N/A" if memory utilization data is not available
            self.create_value_row("Memory Utilized:", "N/A".to_string(), None)
        }
    }

    /// Creates memory used row showing actual MB used
    /// 
    /// Calculates and displays the actual amount of VRAM used in megabytes.
    /// This provides a more concrete understanding of memory usage than percentage alone.
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data containing memory information
    /// 
    /// Returns:
    /// - An Iced Element showing memory used in megabytes
    fn create_memory_used_row_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        if let Some(mem_percentage) = gpu_data.memory_usage {
            // Calculate actual memory used from percentage and total VRAM
            let used_mb = (mem_percentage / 100.0) * gpu_data.vram_mb as f32;
            self.create_value_row(
                "Memory Used:",
                format!("{:.0} MB", used_mb),  // Show as whole number of MB
                None  // No color coding for absolute values
            )
        } else {
            // Show "N/A" if memory usage data is not available
            self.create_value_row("Memory Used:", "N/A".to_string(), None)
        }
    }

    /// Creates GPU temperature row with color coding
    /// 
    /// Displays the current GPU temperature with color coding:
    /// - Green: Cool (0-65°C)
    /// - Yellow: Warm (65-80°C)
    /// - Red: Hot (80°C+)
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data containing temperature information
    /// 
    /// Returns:
    /// - An Iced Element showing GPU temperature with appropriate color
    fn create_temperature_row_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        if let Some(temp) = gpu_data.temp {
            // Format temperature to 1 decimal place and apply color coding
            self.create_value_row(
                "GPU Temperature:",
                format!("{:.1}°C", temp),
                Some(temperature_color(temp))  // Color based on temperature level
            )
        } else {
            // Show "N/A" if temperature data is not available
            self.create_value_row("GPU Temperature:", "N/A".to_string(), None)
        }
    }

    /// Creates video encoder utilization row with color coding
    /// 
    /// Displays the current video encoder utilization percentage.
    /// This is important for users who do video encoding/streaming.
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data containing encoder utilization information
    /// 
    /// Returns:
    /// - An Iced Element showing video encoder utilization with appropriate color
    fn create_encoder_row_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        if let Some(enc) = gpu_data.encoder {
            // Format encoder utilization to 1 decimal place and apply color coding
            self.create_value_row(
                "Video Encoder:",
                format!("{:.1}%", enc),
                Some(utilization_color(enc))  // Use same color scheme as GPU utilization
            )
        } else {
            // Show "N/A" if encoder utilization data is not available
            self.create_value_row("Video Encoder:", "N/A".to_string(), None)
        }
    }

    /// Creates video decoder utilization row with color coding
    /// 
    /// Displays the current video decoder utilization percentage.
    /// This is important for users who do video playback.
    /// 
    /// Arguments:
    /// - gpu_data: The GPU data containing decoder utilization information
    /// 
    /// Returns:
    /// - An Iced Element showing video decoder utilization with appropriate color
    fn create_decoder_row_for_gpu(&self, gpu_data: &GpuData) -> Element<'_, AppStateMessage> {
        if let Some(dec) = gpu_data.decoder {
            // Format decoder utilization to 1 decimal place and apply color coding
            self.create_value_row(
                "Video Decoder:",
                format!("{:.1}%", dec),
                Some(utilization_color(dec))  // Use same color scheme as GPU utilization
            )
        } else {
            // Show "N/A" if decoder utilization data is not available
            self.create_value_row("Video Decoder:", "N/A".to_string(), None)
        }
    }
}

