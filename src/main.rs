use anyhow::anyhow;
use chrono::Local;
use clap::Parser;
use druid::piet::ImageFormat;
use druid::widget::{Button, Flex, Image, Label, Painter};
use druid::{
    AppLauncher, Color, Data, ImageBuf, Lens, LensExt, LinearGradient, LocalizedString, PlatformError, RenderContext, UnitPoint, Widget, WidgetExt, WindowDesc
};

use colored::*;
use fern::Dispatch;
use image::ImageReader;
use log::{debug, error, info, warn};
use regex::Regex;
use std::error::Error;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex as SyncMutex;
use std::sync::{Arc, Mutex};

const TPG_COLOR_BARS_SOF: &str = "tpg_colour_bars.sof";
const TPG_GRAYSCALE_BARS_SOF: &str = "tpg_grayscale_bars.sof";

#[derive(Parser, Debug)]
#[command(name = "Platform Attestation Demo")]
#[command(about = "Program for interacting with Quartus", long_about = None)]
struct CliArgs {
    #[arg(long, default_value_t = 0)]
    device: usize,

    #[arg(long, action = clap::ArgAction::SetFalse, help = "Enable debug mode to print stdout and stderr")]
    debug: bool,
}

#[derive(Clone, Data, Lens)]
struct AppStateCtx {
    current_image: ImageBuf,
}

fn get_sof_file_path(sof_file: &str) -> String {
    let mut path = Path::new("sofs").to_path_buf();
    path.push(sof_file);
    path.to_str().unwrap_or_default().to_string()
}

fn load_image(path: &str) -> Result<ImageBuf, Box<dyn std::error::Error>> {
    debug!("Attempting to load image from path: {}", path);

    match ImageReader::open(path) {
        Ok(reader) => match reader.decode() {
            Ok(img) => {
                info!("Image successfully loaded.");
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                Ok(ImageBuf::from_raw(
                    rgba.into_raw(),
                    druid::piet::ImageFormat::RgbaSeparate,
                    width as usize,
                    height as usize,
                ))
            }
            Err(e) => {
                error!("Error decoding image: {}", e);
                Err(Box::new(e))
            }
        },
        Err(e) => {
            error!("Error opening image file: {}", e);
            Err(Box::new(e))
        }
    }
}

fn build_ui(
    device: String,
    _debug: bool,
    child: Arc<Mutex<std::process::Child>>,
) -> impl Widget<AppStateCtx> {

    let child_clone = child.clone();

    let trusted_image_buf = load_image("images/trusted.png").unwrap_or_else(|_| {
        warn!("Using default empty image due to error in loading.");
        ImageBuf::from_raw(vec![], druid::piet::ImageFormat::RgbaSeparate, 0, 0)
    });
    //let trusted_image = Image::new(trusted_image_buf)

    let color_button = Button::new("ORIGINAL").on_click(move |ctx, data: &mut AppStateCtx, _env| {

        info!("ORIGINAL pressed");
        send_uart_command(&child_clone, "4");// 4

        let trusted_image_buf = load_image("images/trusted.png").unwrap_or_else(|_| {
            warn!("Using default empty image due to error in loading.");
            ImageBuf::from_raw(vec![], druid::piet::ImageFormat::RgbaSeparate, 0, 0)
        });

        data.current_image = trusted_image_buf.clone();
        ctx.request_paint();
    });

    let grey_button = Button::new("MALICIOUS").on_click( move |ctx, data: &mut AppStateCtx, _env| {
        info!("MALICIOUS pressed");
        send_uart_command(&child, "3");//3
        let untrusted_image_buf = load_image("images/malware.png").unwrap_or_else(|_| {
            warn!("Using default empty image due to error in loading.");
            ImageBuf::from_raw(vec![], druid::piet::ImageFormat::RgbaSeparate, 0, 0)
        });
        data.current_image = untrusted_image_buf.clone();
        ctx.request_paint();
    });

    let gradient_painter = Painter::new(|ctx, _, _| {
        let bounds = ctx.size().to_rect();
        let gradient = LinearGradient::new(
            UnitPoint::LEFT,
            UnitPoint::RIGHT,
            (Color::rgb8(255, 0, 0), Color::rgb8(0, 122, 255)),
        );
        ctx.fill(bounds.to_rounded_rect(20.0), &gradient);
    });

    let grey_painter = Painter::new(|ctx, _, _| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds.to_rounded_rect(20.0), &Color::grey8(160));
    });

    let color_button = color_button
        .background(gradient_painter)
        .fix_size(150.0, 80.0);

    let grey_button = grey_button.background(grey_painter).fix_size(150.0, 80.0);

    let device_label = Label::new(format!("Device: {}", device))
        .with_text_size(10.0)
        .center();

    let image_widget = Image::new(trusted_image_buf)
        .fix_width(150.0)
        .fix_height(150.0)
        .center();

    Flex::column()
        .with_child(Label::new("Choose a bitstream:"))
        .with_spacer(20.0)
        .with_child(color_button)
        .with_spacer(10.0)
        .with_child(grey_button)
        .with_spacer(10.0)
        .with_child(image_widget)
        .with_spacer(10.0)
        .with_flex_spacer(1.0)
        .with_child(device_label)
}

fn run_quartus_pgm() -> Vec<String> {
    let output = Command::new("quartus_pgm")
        .arg("--auto")
        .output()
        .expect("Failed to execute quartus_pgm");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let re = Regex::new(r"^\d+\)\s*(.+)").unwrap();

    stdout
        .lines()
        .filter_map(|line| re.captures(line).map(|caps| caps[1].to_string()))
        .collect()
}

fn program_fpga(cable_name: &str, sof_filename: &str, debug: bool) -> Result<(), String> {
    let command = format!(
        "quartus_pgm --cable=\"{}\" --mode=JTAG -o \"p;{}\"",
        cable_name, sof_filename
    );

    info!("Running command: {}", command);

    let output = Command::new("quartus_pgm")
        .arg(format!("--cable={}", cable_name))
        .arg("--mode=JTAG")
        .arg("-o")
        .arg(format!("p;{}", sof_filename))
        .output()
        .map_err(|e| format!("Failed to execute quartus_pgm: {}", e))?;

    if debug {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        info!("stdout: {}", stdout);
        info!("stderr: {}", stderr);
    }

    // Check if the command was successful based on exit status
    if !output.status.success() {
        return Err(format!(
            "Error during FPGA programming. Exit status: {:?}",
            output.status
        ));
    }

    // Regex for matching successful message and error
    let success_regex = Regex::new(r"(?i)Programmer was successful").unwrap();
    let error_regex = Regex::new(r"(?i)error").unwrap();

    // Check for success message and absence of error in stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if success_regex.is_match(&stdout)
        && !error_regex.is_match(&stdout)
        && !error_regex.is_match(&stderr)
    {
        Ok(())
    } else {
        Err(format!(
            "Quartus programming failed.\nstdout: {}\nstderr: {}",
            stdout, stderr
        ))
    }
}

async fn start_juart_terminal() -> std::process::Child {
    let mut child = Command::new("juart-terminal.exe")
        .args(&["--instance", "0", "-d", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start juart-terminal.exe");

    info!("juart-terminal.exe started");
    let mut stdout = child.stdout.take().expect("Failed to capture stdout");

    // Regex to match log levels
    let re_debug = Regex::new(r"^\[DEBUG\]   - ").unwrap();
    let re_info = Regex::new(r"^\[INFO\]    - ").unwrap();
    let re_error = Regex::new(r"^\[ERROR\]   - ").unwrap();

    tokio::spawn(async move {
        let mut buffer = vec![0; 4096];
        let mut line = String::new();
        loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    line.push_str(&String::from_utf8_lossy(&buffer[..n]));

                    while let Some(pos) = line.find('\n') {
                        let full_line = line[..pos].to_string();
                        line = line[pos + 1..].to_string(); 
                        handle_log_line(&full_line, &re_debug, &re_info, &re_error);

                    }
                }
                Err(e) => {
                    error!("Error reading from stdout: {}", e);
                    break;
                }
            }
        }
    });

    child
}

fn handle_log_line(line: &str, re_debug: &Regex, re_info: &Regex, re_error: &Regex) {
    if let Some(log) = re_debug.find(line) {
        let log_message = &line[log.end()..];
        info!("{}", log_message); // INFO log
    } else if let Some(log) = re_info.find(line) {
        let log_message = &line[log.end()..];
        info!("{}", log_message); // INFO log
    } else if let Some(log) = re_error.find(line) {
        let log_message = &line[log.end()..];
        error!("{}", log_message); // ERROR log
    } else {
        info!("{}", line); // DEBUG log
    }
}

fn send_uart_command(child: &Arc<Mutex<Child>>, command: &str) {
    let mut child = child.lock().unwrap();
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(command.as_bytes())
            .expect("Failed to write to juart-terminal");
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            let level = match record.level() {
                log::Level::Error => record.level().to_string().red(),
                log::Level::Warn => record.level().to_string().yellow(),
                log::Level::Info => record.level().to_string().green(),
                log::Level::Debug => record.level().to_string().blue(),
                log::Level::Trace => record.level().to_string().magenta(),
            };
            out.finish(format_args!(
                "[{}] [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                level,
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PlatformError> {
    setup_logger().expect("Logger cofiguration error");

    let trusted_image_buf = load_image("images/trusted.png").unwrap_or_else(|_| {
        warn!("Using default empty image due to error in loading.");
        ImageBuf::from_raw(vec![], druid::piet::ImageFormat::RgbaSeparate, 0, 0)
    });

    let state: AppStateCtx = AppStateCtx {
        current_image: trusted_image_buf,
    };

    let cli_args = CliArgs::parse();

    let devices = run_quartus_pgm();
    for device in devices.clone() {
        info!("{}", device);
    }

    if cli_args.device >= devices.len() {
        error!(
            "Invalid device index: {}. Available devices: {}",
            cli_args.device,
            devices.len()
        );
        return Err(PlatformError::Other(Arc::new(anyhow!(
            "Invalid device index: {}. Available devices: {}",
            cli_args.device,
            devices.len()
        ))));
    }

    let empty_string = String::new();
    let selected_device = devices.get(cli_args.device).unwrap_or(&empty_string);

    let child = Arc::new(Mutex::new(start_juart_terminal().await));

    let window = WindowDesc::new(build_ui(selected_device.clone(), cli_args.debug, child))
        .title(LocalizedString::new("HDMI"))
        .window_size((400.0, 500.0))
        .resizable(false)
        .title("Platform Attestation Demo");

    let _ = AppLauncher::with_window(window).launch(state);

    std::process::exit(0);
}
