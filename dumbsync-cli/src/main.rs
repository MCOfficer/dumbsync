#![forbid(unsafe_code)]
use dumbsync;
use std::borrow::Borrow;
use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "dumbsync-cli")]
enum Opt {
    Generate {
        /// The directory to process
        #[structopt(name = "DIR", parse(from_os_str))]
        dir: PathBuf,
    },
    Download {
        /// THe remote directory to sync from
        #[structopt(name = "URL")]
        url: String,

        /// Directory to sync to
        #[structopt(name = "DIR", parse(from_os_str))]
        target: PathBuf,

        /// Purge files that don't exist on the server
        #[structopt(long)]
        purge: bool,
    },
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Generate { dir } => {
            dumbsync::generate(&dir).unwrap();
        }
        Opt::Download { url, target, purge } => {
            dumbsync::download(
                &url,
                &target,
                dumbsync::aggregate(&url, &target).unwrap().borrow(),
                &purge,
            )
            .unwrap();
        }
    }
}
