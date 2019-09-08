#[macro_use]
extern crate log;
extern crate vcsr;

use vcsr::{args, process_file};

use flexi_logger::{opt_format, Cleanup, Criterion, Logger, Naming};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};
use walkdir::WalkDir;

pub fn main() {
    let args = args::application_args();

    let log_level = match &args.verbose {
        true => String::from("debug,info,warn,error"),
        false => String::from("info,warn,error"),
    };
    Logger::with_str(log_level)
        .log_to_file()
        .directory(dirs::home_dir().unwrap().join(".vcsr").join("logs"))
        .rotate(Criterion::Size(20000), Naming::Numbers, Cleanup::Never)
        .format(opt_format)
        .start()
        .unwrap();

    let multi = MultiProgress::new();
    let bar_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");

    match Command::new("ffmpeg")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(_) => info!("ffmpeg installed. Continuing."),
        Err(_) => {
            error!("ffmpeg not installed. Exiting.");
            std::process::exit(exitcode::SOFTWARE)
        }
    };
    let mut walker: WalkDir;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get() * 2)
        .build()
        .unwrap();

    for path in &args.filenames {
        if !Path::new(path).exists() {
            error!("File does not exist, trying next: {}", path);
            continue;
        }
        if args.recursive {
            walker = WalkDir::new(path);
        } else {
            walker = WalkDir::new(path).max_depth(1);
        }
        for entry in walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension() != None)
            .filter(|e| {
                let extension = e.path().extension().and_then(OsStr::to_str).unwrap();
                if args.exclude_extensions.contains(&String::from(extension)) {
                    warn!("Excluded extension {}. Skipping.", extension);
                    false
                } else {
                    true
                }
            })
        {
            let bar = multi.add(ProgressBar::new(1));
            bar.set_style(bar_style.clone());
            let mut current_args = args.clone();
            let entry_copy = entry.clone();
            let _ = pool.spawn(move || match process_file(&entry, &mut current_args) {
                Ok(file_name) => {
                    let m = format!(
                        "succesfully created {}",
                        file_name.file_name().unwrap().to_string_lossy()
                    );

                    info!("{}", &m);
                    bar.finish_with_message(&m);
                }
                Err(err) => {
                    error!(
                        "Skipped {}: {}",
                        entry_copy.file_name().to_string_lossy().to_owned(),
                        err.to_string()
                    );
                }
            });
        }
    }

    multi.join().unwrap();
    std::process::exit(exitcode::OK);
}
