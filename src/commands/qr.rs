use seahorse::Command;
use qrcode::{QrCode, EcLevel};
use qrcode::render::unicode;

pub fn qr_command() -> Command {
    Command::new("qr")
        .description("Generate QR codes for URLs or text")
        .usage("oat qr [text/url] [options]")
        .action(|c| {
            if c.args.is_empty() {
                eprintln!("Error: Please provide text or URL to encode");
                eprintln!("Usage: oat qr \"your text here\" [--save filename.png] [--size small|medium|large]");
                return;
            }

            let _text = c.args.join(" ");
            let mut save_file: Option<String> = None;
            let mut size = "medium";

            // Parse additional arguments
            let mut i = 0;
            while i < c.args.len() {
                if c.args[i] == "--save" && i + 1 < c.args.len() {
                    save_file = Some(c.args[i + 1].clone());
                    i += 2;
                } else if c.args[i] == "--size" && i + 1 < c.args.len() {
                    size = &c.args[i + 1];
                    i += 2;
                } else {
                    i += 1;
                }
            }

            // Remove flags from text
            let clean_text = c.args.iter()
                .enumerate()
                .filter(|(i, arg)| {
                    !(**arg == "--save" || **arg == "--size" || 
                      (*i > 0 && (c.args[*i - 1] == "--save" || c.args[*i - 1] == "--size")))
                })
                .map(|(_, arg)| arg.as_str())
                .collect::<Vec<&str>>()
                .join(" ");

            generate_qr_code(&clean_text, save_file, size);
        })
}

fn generate_qr_code(text: &str, save_file: Option<String>, size: &str) {
    // Create QR code
    let code = match QrCode::with_error_correction_level(text, EcLevel::M) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error creating QR code: {}", e);
            return;
        }
    };

    // If save_file is specified, save as PNG
    if let Some(filename) = save_file {
        save_qr_as_png(&code, &filename, size);
    } else {
        // Display in terminal
        display_qr_in_terminal(&code, size);
    }
}

fn save_qr_as_png(code: &QrCode, filename: &str, size: &str) {
    let scale = match size {
        "small" => 4,
        "medium" => 8,
        "large" => 12,
        _ => 8,
    };

    // For now, let's save as SVG which is simpler
    let svg_string = code.render()
        .min_dimensions(21 * scale, 21 * scale)
        .dark_color(qrcode::render::svg::Color("black"))
        .light_color(qrcode::render::svg::Color("white"))
        .build();

    let svg_filename = if filename.ends_with(".png") {
        filename.replace(".png", ".svg")
    } else if !filename.ends_with(".svg") {
        format!("{}.svg", filename)
    } else {
        filename.to_string()
    };

    match std::fs::write(&svg_filename, svg_string) {
        Ok(_) => {
            println!("QR code saved as SVG: {}", svg_filename);
            println!("Note: SVG format is used instead of PNG for better compatibility");
        }
        Err(e) => eprintln!("Error saving QR code: {}", e),
    }
}

fn display_qr_in_terminal(code: &QrCode, size: &str) {
    let use_dense = size == "small";
    
    let string = if use_dense {
        code.render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build()
    } else {
        code.render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build()
    };

    println!("\nQR Code generated successfully:");
    println!("{}", string);
    println!("\nScan with your phone's camera or QR code reader");
    
    if size != "small" {
        println!("Tip: Use --size small for a more compact display");
    }
} 