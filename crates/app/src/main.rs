use std::path::PathBuf;
use std::time::Instant;
use clap::Parser;
use ui::OxideApp;

#[derive(Parser, Debug)]
#[command(name = "oxide", version = "0.1.0", author = "Oxide Team", about = "Next-Generation Rust-Native Code Editor")]
struct Cli {
    /// File or workspace folder path to open
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Measure startup time and exit immediately (for performance benchmarking)
    #[arg(long)]
    bench_startup: bool,
}

fn main() -> eframe::Result<()> {
    let startup_timer = Instant::now();

    let cli = Cli::parse();

    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    let init_duration = startup_timer.elapsed();
    tracing::info!("Oxide core initialization took {:?}", init_duration);

    if cli.bench_startup {
        println!(
            "BENCHMARK_COLD_STARTUP: {:.2} ms",
            init_duration.as_secs_f64() * 1000.0
        );
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Oxide")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([720.0, 480.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Oxide",
        native_options,
        Box::new(move |cc| {
            let mut app = OxideApp::new(cc);
            if let Some(path) = cli.path {
                if path.is_file() {
                    app.open_file(&path);
                } else if path.is_dir() {
                    app.workspace.open_folder(path);
                }
            }
            let total_startup = startup_timer.elapsed();
            println!(
                "🚀 Oxide ready in {:.2} ms",
                total_startup.as_secs_f64() * 1000.0
            );
            Ok(Box::new(app))
        }),
    )
}
