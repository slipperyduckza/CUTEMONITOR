fn main() {
    println!("cargo:rustc-link-arg=-Wl,--subsystem,windows");

    if cfg!(target_os = "windows") {
        // Embed the SVG
        let svg_data = include_bytes!("graphs.svg");

        // Parse SVG
        let opt = usvg::Options::default();
        let opt_ref = opt.to_ref();
        let tree = usvg::Tree::from_data(svg_data, &opt_ref).expect("Failed to parse SVG");

        // Sizes
        let sizes = [256, 128, 48, 32, 24, 16];

        // Create ICO
        let mut ico = ico::IconDir::new(ico::ResourceType::Icon);

        for &size in &sizes {
            let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();
            resvg::render(
                &tree,
                usvg::FitTo::Size(size, size),
                tiny_skia::Transform::identity(),
                pixmap.as_mut(),
            )
            .expect("Failed to render");

            // Convert to RGBA
            let rgba = pixmap.data();

            // Create icon image
            let image = ico::IconImage::from_rgba_data(size, size, rgba.to_vec());
            ico.add_entry(ico::IconDirEntry::encode(&image).unwrap());
        }

        // Write ICO
        std::fs::create_dir_all("target").expect("Failed to create target directory");
        let mut ico_data = Vec::new();
        ico.write(&mut ico_data).expect("Failed to write ICO data");
        std::fs::write("target/icon.ico", ico_data).expect("Failed to write ICO");

        // Create resource.rc
        let rc_content = "1 ICON \"target/icon.ico\"\r\n1 24 \"src/app.manifest\"\r\n";
        std::fs::write("resource.rc", rc_content).expect("Failed to write resource.rc");

        // Compile and embed resources
        embed_resource::compile("resource.rc", &[] as &[&str]);
    }
}
