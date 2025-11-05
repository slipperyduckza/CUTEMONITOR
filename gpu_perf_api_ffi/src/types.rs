use std::ffi::c_void;

/// GPUPerfAPI version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPerfApiVersion {
    V3_17,
    V4_1,
}

impl std::fmt::Display for GpuPerfApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuPerfApiVersion::V3_17 => write!(f, "3.17"),
            GpuPerfApiVersion::V4_1 => write!(f, "4.1"),
        }
    }
}

/// GPA status codes (simplified)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaStatus {
    Ok = 0,
    GenericError = -1,
    InvalidParameter = -2,
    ContextNotOpen = -3,
    ContextAlreadyOpen = -4,
    ContextAlreadyCounterEnabled = -5,
    SessionAlreadyOpened = -6,
    SessionNotOpened = -7,
    SessionAlreadyStarted = -8,
    SessionNotStarted = -9,
    SampleAlreadyStarted = -10,
    SampleNotStarted = -11,
    SampleAlreadyEnded = -12,
    SampleNotEnded = -13,
    CounterNotFound = -14,
    CounterAlreadyEnabled = -15,
    CounterNotEnabled = -16,
    CounterResultNotAvailable = -17,
    CounterResultNotReady = -18,
    CounterNotSupported = -19,
    DeviceNotSupported = -20,
    InvalidApiType = -21,
    InvalidCounter = -22,
    InvalidSession = -23,
    InvalidSample = -24,
    InvalidContext = -25,
    InvalidDevice = -26,
    InvalidCommandList = -27,
    CommandListAlreadyClosed = -28,
    CommandListNotClosed = -29,
    InvalidPass = -30,
    PassAlreadyEnded = -31,
    PassNotEnded = -32,
    InvalidSampleType = -33,
    InvalidContextFlags = -34,
    UnexpectedApiType = -35,
    UnexpectedCounterType = -36,
    UnexpectedDataType = -37,
    UnexpectedUsageType = -38,
    UnexpectedResultType = -39,
    UnexpectedUuid = -40,
    UnexpectedDeviceId = -41,
    UnexpectedRevisionId = -42,
    UnexpectedVendorId = -43,
    UnexpectedGpuIndex = -44,
    UnexpectedNumAdapters = -45,
    UnexpectedNumCounters = -46,
    UnexpectedCounterIndex = -47,
    UnexpectedCounterResultIndex = -48,
    UnexpectedCounterResultCount = -49,
    UnexpectedCounterResultSize = -50,
    UnexpectedCounterResultType = -51,
    UnexpectedCounterResultUuid = -52,
    UnexpectedCounterResultDeviceId = -53,
    UnexpectedCounterResultRevisionId = -54,
    UnexpectedCounterResultVendorId = -55,
    UnexpectedCounterResultGpuIndex = -56,
    UnexpectedCounterResultNumAdapters = -57,
    UnexpectedCounterResultNumCounters = -58,
    ErrorGpaAlreadyInitialized = -100,
    ErrorHardwareNotSupported = -101,
    UnexpectedCounterNotSupportedLegacy = -318,
    UnknownError = -999,
}

/// GPA context flags
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpaOpenContextFlags {
    pub bits: u32,
}

/// GPA initialize flags
pub type GpaInitializeFlags = u32;

/// GPA initialize flag constants
pub const GPA_INITIALIZE_DEFAULT_BIT: GpaInitializeFlags = 0;  ///< Initialize GPA using all default options.

/// GPA context flag constants
pub const GPA_OPEN_CONTEXT_DEFAULT_BIT: GpaOpenContextFlags = GpaOpenContextFlags { bits: 0x00000000 };

/// Session sample type constants for easy access
pub const GPA_SESSION_SAMPLE_TYPE_DISCRETE_COUNTER: GpaSessionSampleType = GpaSessionSampleType::DiscreteCounter;
pub const GPA_SESSION_SAMPLE_TYPE_CUMULATIVE_COUNTER: GpaSessionSampleType = GpaSessionSampleType::CumulativeCounter;
pub const GPA_SESSION_SAMPLE_TYPE_SOFTWARE: GpaSessionSampleType = GpaSessionSampleType::Software;

impl GpaOpenContextFlags {
    pub const NONE: Self = Self { bits: 0 };
    pub const ENABLE_HARDWARE_COUNTERS: Self = Self { bits: 0x00000001 };
    pub const ENABLE_SOFTWARE_COUNTERS: Self = Self { bits: 0x00000002 };
    pub const CONTEXT_ENABLE_COUNTER_DEMUX: Self = Self { bits: 0x00000004 };
    pub const CONTEXT_ENABLE_TERTIARY_COUNTERS: Self = Self { bits: 0x00000008 };
}

/// GPA session sample type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaSessionSampleType {
    DiscreteCounter = 0,
    CumulativeCounter = 1,
    Software = 2,
}

/// GPA session sample type flags for GpaGetSupportedSampleTypes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpaContextSampleTypeFlags {
    pub bits: u32,
}

impl GpaContextSampleTypeFlags {
    pub const NONE: Self = Self { bits: 0 };
    pub const DISCRETE_COUNTER: Self = Self { bits: 0x00000001 };
    pub const CUMULATIVE_COUNTER: Self = Self { bits: 0x00000002 };
    pub const SOFTWARE: Self = Self { bits: 0x00000004 };
    pub const LAST: Self = Self { bits: 0x00000008 };
}

/// Sample result structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GpaSampleResult {
    pub sample_id: GpaUInt32,
    pub counter_index: GpaUInt32,
    pub result: GpaUInt64,
    pub result_type: GpaResultType,
}

/// Command list types for GPA 4.0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GpaCommandListId(pub *mut c_void);

/// GPA context and session types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GpaContextId(pub *mut c_void);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GpaSessionId(pub *mut c_void);

pub type GpaUInt32 = u32;
pub type GpaUInt64 = u64;

/// GPA counter sample type

// Implement Send for GPUPerfAPI wrapper types since they are only used through synchronized APIs
// This is safe because actual FFI calls are thread-safe in GPUPerfAPI
unsafe impl Send for GpaContextId {}
unsafe impl Send for GpaSessionId {}
unsafe impl Send for GpaCommandListId {}

/// GPA counter sample type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaCounterSampleType {
    Discrete = 0,
    Cumulative = 1,
}

/// GPA logging type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaLoggingType {
    Error = 0,
    Warning = 1,
    Message = 2,
    Trace = 3,
}

/// GPA UUID structure (128-bit)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpaUuid {
    pub data: [u8; 16],
}

/// GPU adapter information
#[derive(Debug, Clone)]
pub struct GpuAdapterInfo {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub hardware_generation: Option<String>,
}

/// GPA 3.17 Device Information
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GpaDeviceInfo {
    pub device_id: GpaUInt32,
    pub device_name: [i8; 256],
    pub vendor_id: GpaUInt32,
    pub revision_id: GpaUInt32,
    pub device_index: GpaUInt32,
}

/// GPUPerfAPI 4.0+ Function Table
#[repr(C)]
#[derive(Debug)]
pub struct GpaFunctionTable {
    pub major_version: GpaUInt32,
    pub minor_version: GpaUInt32,
    
    // GPA 3.17 Context Management (Legacy)
    pub gpa_initialize: Option<unsafe extern "C" fn(GpaInitializeFlags) -> GpaStatus>,
    pub gpa_destroy: Option<unsafe extern "C" fn() -> GpaStatus>,
    pub gpa_get_device_count: Option<unsafe extern "C" fn(*mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_device_index: Option<unsafe extern "C" fn(*const i8, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_device_info: Option<unsafe extern "C" fn(GpaUInt32, *mut GpaDeviceInfo) -> GpaStatus>,
    pub gpa_open_context_on_device: Option<unsafe extern "C" fn(GpaUInt32, *mut GpaContextId) -> GpaStatus>,
    
    // GPA 3.17 Counter Discovery (Context-based)
    pub gpa_get_num_counters_317: Option<unsafe extern "C" fn(GpaContextId, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_counter_name_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32, *mut *const i8) -> GpaStatus>,
    pub gpa_get_counter_description_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32, *mut *const i8) -> GpaStatus>,
    pub gpa_get_counter_group_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32, *mut *const i8) -> GpaStatus>,
    pub gpa_get_counter_data_type_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32, *mut GpaDataType) -> GpaStatus>,
    pub gpa_get_counter_usage_type_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32, *mut GpaUsageType) -> GpaStatus>,
    pub gpa_get_counter_sample_type_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32, *mut GpaCounterSampleType) -> GpaStatus>,
    
    // GPA 3.17 Counter Enable/Disable (Context-based)
    pub gpa_enable_counter_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32) -> GpaStatus>,
    pub gpa_disable_counter_317: Option<unsafe extern "C" fn(GpaContextId, GpaUInt32) -> GpaStatus>,
    pub gpa_enable_counter_by_name_317: Option<unsafe extern "C" fn(GpaContextId, *const i8) -> GpaStatus>,
    pub gpa_disable_counter_by_name_317: Option<unsafe extern "C" fn(GpaContextId, *const i8) -> GpaStatus>,
    pub gpa_enable_all_counters_317: Option<unsafe extern "C" fn(GpaContextId) -> GpaStatus>,
    pub gpa_disable_all_counters_317: Option<unsafe extern "C" fn(GpaContextId) -> GpaStatus>,
    
    // GPA 3.17 Session Management
    pub gpa_create_session_317: Option<unsafe extern "C" fn(GpaContextId, GpaSessionSampleType, *mut GpaSessionId) -> GpaStatus>,
    pub gpa_delete_session_317: Option<unsafe extern "C" fn(GpaSessionId) -> GpaStatus>,
    pub gpa_begin_session_317: Option<unsafe extern "C" fn(GpaSessionId) -> GpaStatus>,
    pub gpa_end_session_317: Option<unsafe extern "C" fn(GpaSessionId) -> GpaStatus>,
    
    // GPA 3.17 Sample Management
    pub gpa_begin_sample_317: Option<unsafe extern "C" fn(GpaSessionId, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_end_sample_317: Option<unsafe extern "C" fn(GpaSessionId, GpaUInt32) -> GpaStatus>,
    pub gpa_get_pass_count_317: Option<unsafe extern "C" fn(GpaSessionId, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_is_session_complete_317: Option<unsafe extern "C" fn(GpaSessionId, *mut bool) -> GpaStatus>,
    pub gpa_is_pass_complete_317: Option<unsafe extern "C" fn(GpaSessionId, GpaUInt32, *mut bool) -> GpaStatus>,
    
    // GPA 3.17 Result Retrieval
    pub gpa_get_sample_count_317: Option<unsafe extern "C" fn(GpaSessionId, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_sample_result_size_317: Option<unsafe extern "C" fn(GpaSessionId, GpaUInt32, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_sample_result_317: Option<unsafe extern "C" fn(GpaSessionId, GpaUInt32, *mut GpaSampleResult) -> GpaStatus>,
    
    // GPA 4.0+ Context Management
    pub gpa_open_context: Option<unsafe extern "C" fn(*const c_void, GpaOpenContextFlags, *mut *mut c_void) -> GpaStatus>,
    pub gpa_close_context: Option<unsafe extern "C" fn(*mut c_void) -> GpaStatus>,
    
    // Device Information
    pub gpa_get_supported_sample_types: Option<unsafe extern "C" fn(*mut c_void, *mut GpaContextSampleTypeFlags) -> GpaStatus>,
    pub gpa_get_device_and_revision_id: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_device_name: Option<unsafe extern "C" fn(*mut c_void, *mut *const i8) -> GpaStatus>,
    pub gpa_get_device_generation: Option<unsafe extern "C" fn(*mut c_void, *mut *const i8) -> GpaStatus>,
    pub gpa_get_device_max_wave_slots: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_device_max_vgprs: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32) -> GpaStatus>,
    
    // Session Management
    pub gpa_create_session: Option<unsafe extern "C" fn(*mut c_void, GpaSessionSampleType, *mut *mut c_void) -> GpaStatus>,
    pub gpa_delete_session: Option<unsafe extern "C" fn(*mut c_void) -> GpaStatus>,
    pub gpa_begin_session: Option<unsafe extern "C" fn(*mut c_void) -> GpaStatus>,
    pub gpa_end_session: Option<unsafe extern "C" fn(*mut c_void) -> GpaStatus>,
    
    // Counter Discovery (Session-based in 4.0+)
    pub gpa_get_num_counters: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_counter_name: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut *const i8) -> GpaStatus>,
    pub gpa_get_counter_index: Option<unsafe extern "C" fn(*mut c_void, *const i8, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_counter_group: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut *const i8) -> GpaStatus>,
    pub gpa_get_counter_description: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut *const i8) -> GpaStatus>,
    pub gpa_get_counter_data_type: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut GpaDataType) -> GpaStatus>,
    pub gpa_get_counter_usage_type: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut GpaUsageType) -> GpaStatus>,
    pub gpa_get_counter_sample_type: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut GpaCounterSampleType) -> GpaStatus>,
    pub gpa_get_counter_uuid: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut GpaUuid) -> GpaStatus>,
    
    // Counter Enable/Disable
    pub gpa_enable_counter: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32) -> GpaStatus>,
    pub gpa_disable_counter: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32) -> GpaStatus>,
    pub gpa_enable_counter_by_name: Option<unsafe extern "C" fn(*mut c_void, *const i8) -> GpaStatus>,
    pub gpa_disable_counter_by_name: Option<unsafe extern "C" fn(*mut c_void, *const i8) -> GpaStatus>,
    pub gpa_enable_all_counters: Option<unsafe extern "C" fn(*mut c_void) -> GpaStatus>,
    pub gpa_disable_all_counters: Option<unsafe extern "C" fn(*mut c_void) -> GpaStatus>,
    
    // Counter Scheduling
    pub gpa_get_pass_count: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_num_enabled_counters: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_enabled_index: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_is_counter_enabled: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut bool) -> GpaStatus>,
    
    // Sample Management
    pub gpa_begin_command_list: Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> GpaStatus>,
    pub gpa_end_command_list: Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> GpaStatus>,
    pub gpa_begin_sample: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_end_sample: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32) -> GpaStatus>,
    pub gpa_continue_sample_on_command_list: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut c_void) -> GpaStatus>,
    pub gpa_copy_secondary_samples: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> GpaStatus>,
    
    // Result Retrieval
    pub gpa_get_sample_count: Option<unsafe extern "C" fn(*mut c_void, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_sample_result_size: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut GpaUInt32) -> GpaStatus>,
    pub gpa_get_sample_result: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut GpaSampleResult) -> GpaStatus>,
    pub gpa_is_session_complete: Option<unsafe extern "C" fn(*mut c_void, *mut bool) -> GpaStatus>,
    pub gpa_is_pass_complete: Option<unsafe extern "C" fn(*mut c_void, GpaUInt32, *mut bool) -> GpaStatus>,
    
    // Logging and Status
    pub gpa_register_logging_callback: Option<unsafe extern "C" fn(unsafe extern "C" fn(GpaLoggingType, *const i8)) -> GpaStatus>,
    pub gpa_get_status_as_str: Option<unsafe extern "C" fn(GpaStatus, *mut *const i8) -> GpaStatus>,
    
    // Helper functions
    pub gpa_get_data_type_as_str: Option<unsafe extern "C" fn(GpaDataType, *mut *const i8) -> GpaStatus>,
    pub gpa_get_usage_type_as_str: Option<unsafe extern "C" fn(GpaUsageType, *mut *const i8) -> GpaStatus>,
}

impl Default for GpaFunctionTable {
    fn default() -> Self {
        Self {
            major_version: 0,
            minor_version: 0,
            // GPA 3.17 Legacy Functions
            gpa_initialize: None,
            gpa_destroy: None,
            gpa_get_device_count: None,
            gpa_get_device_index: None,
            gpa_get_device_info: None,
            gpa_open_context_on_device: None,
            gpa_get_num_counters_317: None,
            gpa_get_counter_name_317: None,
            gpa_get_counter_description_317: None,
            gpa_get_counter_group_317: None,
            gpa_get_counter_data_type_317: None,
            gpa_get_counter_usage_type_317: None,
            gpa_get_counter_sample_type_317: None,
            gpa_enable_counter_317: None,
            gpa_disable_counter_317: None,
            gpa_enable_counter_by_name_317: None,
            gpa_disable_counter_by_name_317: None,
            gpa_enable_all_counters_317: None,
            gpa_disable_all_counters_317: None,
            gpa_create_session_317: None,
            gpa_delete_session_317: None,
            gpa_begin_session_317: None,
            gpa_end_session_317: None,
            gpa_begin_sample_317: None,
            gpa_end_sample_317: None,
            gpa_get_pass_count_317: None,
            gpa_is_session_complete_317: None,
            gpa_is_pass_complete_317: None,
            gpa_get_sample_count_317: None,
            gpa_get_sample_result_size_317: None,
            gpa_get_sample_result_317: None,
            // GPA 4.0+ Functions
            gpa_open_context: None,
            gpa_close_context: None,
            gpa_get_supported_sample_types: None,
            gpa_get_device_and_revision_id: None,
            gpa_get_device_name: None,
            gpa_get_device_generation: None,
            gpa_get_device_max_wave_slots: None,
            gpa_get_device_max_vgprs: None,
            gpa_create_session: None,
            gpa_delete_session: None,
            gpa_begin_session: None,
            gpa_end_session: None,
            gpa_get_num_counters: None,
            gpa_get_counter_name: None,
            gpa_get_counter_index: None,
            gpa_get_counter_group: None,
            gpa_get_counter_description: None,
            gpa_get_counter_data_type: None,
            gpa_get_counter_usage_type: None,
            gpa_get_counter_sample_type: None,
            gpa_get_counter_uuid: None,
            gpa_enable_counter: None,
            gpa_disable_counter: None,
            gpa_enable_counter_by_name: None,
            gpa_disable_counter_by_name: None,
            gpa_enable_all_counters: None,
            gpa_disable_all_counters: None,
            gpa_get_pass_count: None,
            gpa_get_num_enabled_counters: None,
            gpa_get_enabled_index: None,
            gpa_is_counter_enabled: None,
            gpa_begin_command_list: None,
            gpa_end_command_list: None,
            gpa_begin_sample: None,
            gpa_end_sample: None,
            gpa_continue_sample_on_command_list: None,
            gpa_copy_secondary_samples: None,
            gpa_get_sample_count: None,
            gpa_get_sample_result_size: None,
            gpa_get_sample_result: None,
            gpa_is_session_complete: None,
            gpa_is_pass_complete: None,
            gpa_register_logging_callback: None,
            gpa_get_status_as_str: None,
            gpa_get_data_type_as_str: None,
            gpa_get_usage_type_as_str: None,
        }
    }
}

/// Counter information
#[derive(Debug, Clone)]
pub struct CounterInfo {
    pub name: String,
    pub group: String,
    pub description: String,
    pub data_type: GpaDataType,
    pub usage_type: GpaUsageType,
    pub result_type: GpaResultType,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaDataType {
    Float32 = 0,
    Float64 = 1,
    UInt32 = 2,
    UInt64 = 3,
    Int32 = 4,
    Int64 = 5,
    Double = 6,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaUsageType {
    Ratio = 0,
    Percentage = 1,
    Kilobytes = 2,
    Bytes = 3,
    Megabytes = 4,
    Gigabytes = 5,
    Terabytes = 6,
    KiloBytesPerSecond = 7,
    MegaBytesPerSecond = 8,
    GigaBytesPerSecond = 9,
    TeraBytesPerSecond = 10,
    Cycles = 11,
    Milliseconds = 12,
    Nanoseconds = 13,
    PercentageOfPeak = 14,
    Items = 15,
    Count = 16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaResultType {
    Bool = 0,
    Int64 = 1,
    Float32 = 2,
    Float64 = 3,
    Uint64 = 4,
    String = 5,
}

/// Result type for GPA operations
pub type GpaResult<T> = Result<T, GpaError>;

#[derive(Debug, thiserror::Error)]
pub enum GpaError {
    #[error("GPA Error: {status:?}")]
    Status { status: GpaStatus },
    #[error("Library loading error: {0}")]
    LibraryLoad(#[from] libloading::Error),
    #[error("Invalid version: {0}")]
    InvalidVersion(String),
    #[error("Unsupported operation for version: {version}")]
    UnsupportedOperation { version: GpuPerfApiVersion },
    #[error("Null pointer encountered")]
    NullPointer,
    #[error("Invalid parameter")]
    InvalidParameter,
    #[error("String conversion error: {0}")]
    StringConversion(#[from] std::ffi::NulError),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Conversion(#[from] std::string::FromUtf8Error),
}

impl From<GpaStatus> for GpaError {
    fn from(status: GpaStatus) -> Self {
        GpaError::Status { status }
    }
}