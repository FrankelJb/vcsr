#[macro_use]
extern crate log;
extern crate vcsr;

use vcsr::{args, process_file};

use indicatif::MultiProgress;
use std::{
    error::Error,
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};
use walkdir::WalkDir;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = args::application_args();
    let level = match &args.verbose {
        true => tracing::Level::DEBUG,
        false => tracing::Level::INFO,
    };
    tracing_subscriber::fmt().with_max_level(level).init();

    debug!("{:?}", args);

    // match &args.verbose {
    // true => {
    //     Logger::try_with_str(String::from("debug,info,warn,error"))?
    //         .format(opt_format)
    //         .start()?;
    // }
    // false => {
    //     Logger::try_with_str(String::from("info,warn,error"))?
    //         .log_to_file(
    //             FileSpec::default()
    //                 .directory(dirs::home_dir().unwrap().join(".vcsr").join("logs")),
    //         )
    //         .rotate(Criterion::Size(20000), Naming::Numbers, Cleanup::Never)
    //         .format(opt_format)
    //         .start()?;
    // }
    // };

    let multi = MultiProgress::new();

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
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get() * 2)
        .build()
        .unwrap();

    for path in &args.filenames {
        debug!("processing path: {}", path);
        if !Path::new(path).exists() {
            error!("File does not exist, trying next: {}", path);
            debug!("File exists, continuing");
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
                    info!("Excluded extension {}. Skipping.", extension);
                    false
                } else {
                    true
                }
            })
        {
            let mut current_args = args.clone();
            let entry_copy = entry.clone();
            match process_file(&entry, &mut current_args, &multi) {
                Ok(file_name) => {
                    let m = format!(
                        "succesfully created {}",
                        file_name.file_name().unwrap().to_string_lossy()
                    );
                    debug!("{}", &m);
                }
                Err(err) => {
                    error!(
                        "Skipped {}: {}",
                        entry_copy.file_name().to_string_lossy().to_owned(),
                        err.to_string()
                    );
                }
            };
        }
    }

    std::process::exit(exitcode::OK);
}
