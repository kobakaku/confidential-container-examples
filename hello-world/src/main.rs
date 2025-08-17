use actix_web::{web, App, HttpResponse, HttpServer, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

async fn index() -> Result<HttpResponse> {
    let verbose_report = get_verbose_report();
    let html = get_html_str("my-rust-app", &verbose_report);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn get_verbose_report() -> String {
    let verbose_report_path = "./verbose-report";

    // Check if file exists and make it executable
    if let Ok(metadata) = fs::metadata(verbose_report_path) {
        let mut perms = metadata.permissions();
        perms.set_mode(perms.mode() | 0o111); // Add execute permission
        let _ = fs::set_permissions(verbose_report_path, perms);

        // Run the verbose report
        if let Ok(output) = Command::new(verbose_report_path).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // If stdout is empty but stderr has content, return appropriate message
            if stdout.trim().is_empty() && !stderr.trim().is_empty() {
                return String::from("Verbose report not available in local environment");
            }

            return stdout.to_string();
        }
    }

    String::from("Verbose report not available")
}

fn get_html_str(service_name: &str, verbose_report: &str) -> String {
    let formatted_text = format_verbose_report(verbose_report);

    let style = r#"
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            margin: 0;
            padding: 0;
            background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            background: rgba(255, 255, 255, 0.95);
            border-radius: 20px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            padding: 40px;
            max-width: 800px;
            margin: 20px;
            text-align: center;
        }
        h1 {
            color: #1e3c72;
            font-size: 2.5em;
            margin-bottom: 20px;
            text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.1);
        }
        .attestation-container {
            background: #f8f9fa;
            border-radius: 10px;
            padding: 20px;
            margin-top: 30px;
            font-family: 'Courier New', monospace;
            text-align: left;
            max-height: 400px;
            overflow-y: auto;
            box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
        }
        .attestation-container h3 {
            color: #1e3c72;
            margin-top: 0;
            margin-bottom: 15px;
            font-size: 1.2em;
            text-align: center;
        }
        .attestation-container::-webkit-scrollbar {
            width: 8px;
        }
        .attestation-container::-webkit-scrollbar-track {
            background: #e9ecef;
            border-radius: 4px;
        }
        .attestation-container::-webkit-scrollbar-thumb {
            background: #6c757d;
            border-radius: 4px;
        }
        .attestation-container strong {
            color: #495057;
            display: block;
            margin-top: 15px;
            margin-bottom: 5px;
            font-size: 1.1em;
        }
        .info-badge {
            display: inline-block;
            background: #17a2b8;
            color: white;
            padding: 5px 15px;
            border-radius: 20px;
            font-size: 0.9em;
            margin-bottom: 20px;
        }
        .powered-by {
            margin-top: 30px;
            color: #6c757d;
            font-size: 0.9em;
        }
        .powered-by a {
            color: #007bff;
            text-decoration: none;
        }
        .powered-by a:hover {
            text-decoration: underline;
        }
    </style>
    "#;

    let info_badge = if verbose_report.contains("not available") {
        r#"<div class="info-badge">🏠 Running in Local Environment</div>"#
    } else {
        r#"<div class="info-badge">☁️ Running in Azure Confidential Container</div>"#
    };

    let attestation_section = if !formatted_text.trim().is_empty() {
        format!(
            r#"<div class="attestation-container"><h3>📋 Attestation Report</h3>{}</div>"#,
            formatted_text
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Confidential Container - {}</title>
            {}
        </head>
        <body>
            <div class="container">
                <h1>🔒 Welcome to Confidential {}!</h1>
                {}
                {}
                <div class="powered-by">
                    Powered by <a href="https://www.rust-lang.org/" target="_blank">Rust</a> 
                    & <a href="https://actix.rs/" target="_blank">Actix Web</a>
                </div>
            </div>
        </body>
        </html>"#,
        service_name, style, service_name, info_badge, attestation_section
    )
}

fn format_verbose_report(report: &str) -> String {
    let words: Vec<&str> = report.split_whitespace().collect();
    let mut html_output = Vec::new();
    let mut temp_out = vec!["<br>"];
    let mut counter = 0;

    for word in words {
        if word.ends_with(':') {
            temp_out.push(word);
            temp_out.push("<br>");
            html_output.push(format!("<strong>{}</strong>", temp_out.join(" ")));
            temp_out = vec!["<br>"];
            counter = 0;
        } else if !word.chars().all(|c| c.is_ascii_hexdigit()) {
            temp_out.push(word);
            counter = 0;
        } else {
            if counter == 2 {
                html_output.push("<br>".to_string());
                counter = 0;
            }
            html_output.push(word.to_string());
            counter += 1;
        }
    }

    // Add any remaining content in temp_out
    if temp_out.len() > 1 {
        html_output.push(temp_out.join(" "));
    }

    html_output.join(" ")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Hello World Confidential Container Rust server on port 80");

    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind("0.0.0.0:80")?
        .run()
        .await
}
