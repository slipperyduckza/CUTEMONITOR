# Agent Guidelines

In Rust programming, snake case is a naming convention where words are written in all lowercase letters and separated by underscores (_). It is a widely adopted convention in Rust for various identifiers, particularly for: 
Variables, eg: my_variable, total_count
Functions and methods, eg: calculate_sum(), print_message()
Modules eg: my_module, utility_functions
Key characteristics of snake case:
All lowercase: All letters in the identifier are in lowercase.
Underscore separation: Spaces between words are replaced with underscores.
No leading/trailing underscores: Identifiers typically do not start or end with an underscore.
Rust's style guidelines and tools like rustfmt and Clippy actively promote the use of snake case for these elements, ensuring consistency and readability within the Rust ecosystem. While other casing styles exist, snake case is the standard for most identifiers in Rust, with exceptions like PascalCase for structs and enums, and SCREAMING_SNAKE_CASE for constants.

## Windows 11 Compatibility Notice

**IMPORTANT**: Windows 11 latest edition has retired WMI (Windows Management Instrumentation) in favor of PowerShell CIM (Common Information Model) cmdlets. 

**Do NOT use WMI in new code**.
This applies to all Windows projects going forward to ensure compatibility with Windows 11 and future Windows versions.

I do not mind about long build times for this app, so please extend your command timeout for this project