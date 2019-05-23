use crate::{sub_command::SubCommand, watch::Watch, Result};
use futures::channel::oneshot;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::thread;
use structopt::StructOpt;

/// Serve a fart project over a local server, watch it for changes, and re-build
/// and re-un it as necessary.
#[derive(Clone, Debug, StructOpt)]
pub struct Serve {
    /// The project to serve.
    #[structopt(parse(from_os_str), default_value = ".")]
    project: PathBuf,

    /// The port to serve locally on.
    #[structopt(short = "p", long = "port", default_value = "9090")]
    port: u16,

    /// Extra arguments passed along to each invocation of `cargo run`.
    #[structopt(long = "")]
    extra: Vec<String>,
}

impl Serve {
    fn app_data(&mut self) -> AppData {
        AppData {
            project: self.project.clone(),
        }
    }
}

impl SubCommand for Serve {
    fn set_extra(&mut self, extra: &[String]) {
        assert!(self.extra.is_empty());
        self.extra = extra.iter().cloned().collect();
    }

    fn run(mut self) -> Result<()> {
        let project = self.project.clone();
        let extra = self.extra.clone();
        thread::spawn(move || {
            Watch::new(project, extra).run().unwrap();
        });

        let data = self.app_data();

        let mut app = tide::App::new(data);
        app.at("/").get(index);
        app.at("/images/:image").get(image);
        app.serve(format!("127.0.0.1:{}", self.port))?;

        Ok(())
    }
}

struct AppData {
    project: PathBuf,
}

async fn index(_cx: tide::Context<AppData>) -> tide::http::Response<&'static str> {
    tide::http::Response::builder()
        .header(tide::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .status(tide::http::StatusCode::OK)
        .body(include_str!("index.html"))
        .unwrap()
}

async fn image(cx: tide::Context<AppData>) -> tide::EndpointResult<tide::http::Response<Vec<u8>>> {
    let image = cx.param::<PathBuf>("image").unwrap();
    if image.extension() != Some(OsStr::new("svg")) {
        return Ok(tide::http::Response::builder()
            .status(tide::http::StatusCode::NOT_FOUND)
            .body(vec![])
            .unwrap());
    }
    let path = cx.app_data().project.join("images").join(image);
    serve_static_file(path).await
}

async fn serve_static_file(path: PathBuf) -> tide::EndpointResult<tide::http::Response<Vec<u8>>> {
    match read_file(path).await {
        Ok(contents) => Ok(tide::http::Response::builder()
            .status(tide::http::StatusCode::OK)
            .body(contents)
            .unwrap()),
        Err(e) => Ok(tide::http::Response::builder()
            .status(if e.kind() == io::ErrorKind::NotFound {
                tide::http::StatusCode::NOT_FOUND
            } else {
                tide::http::StatusCode::INTERNAL_SERVER_ERROR
            })
            .body(e.to_string().into())
            .unwrap()),
    }
}

async fn read_file(path: PathBuf) -> io::Result<Vec<u8>> {
    // lol... Don't want to pull in all of tokio-fs and do futures 0.1 and 0.3
    // compat shimming... So just spawn a thread. Eventually the async WG will
    // hopefully provide a proper solution.
    let (sender, receiver) = oneshot::channel();
    let _ = thread::spawn(move || {
        let _ = sender.send(fs::read(path));
    });

    receiver.await.unwrap()
}
