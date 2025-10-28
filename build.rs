fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set("FileDescription", "LibreHardware Prototype");
        res.set("ProductName", "LibreHardware Prototype");
        res.set("CompanyName", "Your Company");
        res.set_version_info(winresource::VersionInfo::FILEVERSION, 0x0000000000010000);
        res.set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0000000000010000);
        res.set_icon("cutemonitor.ico");
        res.set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#,
        );
        res.compile().unwrap();
    }
}
