mod events;

use crate::{
    command_ext::CommandExt, output::Output, sub_command::SubCommand, watcher::Watcher, Result,
};
use failure::ResultExt;
use futures::channel::{mpsc, oneshot};
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
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
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            consts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl SubCommand for Serve {
    fn set_extra(&mut self, extra: &[String]) {
        assert!(self.extra.is_empty());
        self.extra = extra.iter().cloned().collect();
    }

    fn run(mut self) -> Result<()> {
        let app_data = self.app_data();

        let subscribers = app_data.subscribers.clone();
        let project = self.project.clone();
        let extra = self.extra.clone();
        thread::spawn(move || {
            Watcher::new(project)
                .extra(extra)
                .on_output({
                    let subscribers = subscribers.clone();
                    move |output| {
                        let send_output = || -> Result<()> {
                            let event = events::Event::new("output".into(), output)
                                .context("failed to serialize output event")?;
                            futures::executor::block_on(events::broadcast(&subscribers, event))?;
                            Ok(())
                        };
                        if let Err(e) = send_output() {
                            eprintln!("warning: {}", e);
                        }
                    }
                })
                .on_start({
                    let subscribers = subscribers.clone();
                    move || {
                        let send_rerun = || -> Result<()> {
                            let event = events::Event::new("start".into(), &())
                                .context("failed to serialize rerun event")?;
                            futures::executor::block_on(events::broadcast(&subscribers, event))?;
                            Ok(())
                        };
                        if let Err(e) = send_rerun() {
                            eprintln!("warning: {}", e);
                        }
                    }
                })
                .on_finish({
                    let subscribers = subscribers.clone();
                    move || {
                        let send_rerun = || -> Result<()> {
                            let event = events::Event::new("finish".into(), &())
                                .context("failed to serialize rerun event")?;
                            futures::executor::block_on(events::broadcast(&subscribers, event))?;
                            Ok(())
                        };
                        if let Err(e) = send_rerun() {
                            eprintln!("warning: {}", e);
                        }
                    }
                })
                .watch()
                .unwrap();
        });

        let mut app = tide::App::with_state(app_data);
        app.at("/").get(serve_from_memory(
            "text/html",
            include_str!("static/index.html"),
        ));
        app.at("/styles.css").get(serve_from_memory(
            "text/css",
            include_str!("static/styles.css"),
        ));
        app.at("/script.js").get(serve_from_memory(
            "text/javascript",
            include_str!("static/script.js"),
        ));
        app.at("/events").get(events);
        app.at("/rerun").post(rerun);
        app.at("/images/:image").get(image);
        futures::executor::block_on(app.serve(format!("127.0.0.1:{}", self.port)))?;

        Ok(())
    }
}

struct AppData {
    project: PathBuf,
    subscribers: Arc<Mutex<HashMap<usize, mpsc::Sender<events::Event>>>>,
    consts: Arc<Mutex<HashMap<String, String>>>,
}

fn serve_from_memory(
    content_type: &'static str,
    body: &'static str,
) -> impl tide::Endpoint<AppData> {
    return ServeFromMemory { content_type, body };

    struct ServeFromMemory {
        content_type: &'static str,
        body: &'static str,
    }

    impl<T> tide::Endpoint<T> for ServeFromMemory {
        type Fut = futures::future::Ready<tide::http::Response<http_service::Body>>;

        fn call(&self, _cx: tide::Context<T>) -> Self::Fut {
            futures::future::ready(
                tide::http::Response::builder()
                    .header(
                        tide::http::header::CONTENT_TYPE,
                        format!("{}; charset=utf-8", self.content_type),
                    )
                    .status(tide::http::StatusCode::OK)
                    .body(http_service::Body::from(self.body))
                    .unwrap(),
            )
        }
    }
}

async fn events(
    cx: tide::Context<AppData>,
) -> tide::EndpointResult<tide::http::Response<http_service::Body>> {
    let events = events::EventStream::new(cx.state().subscribers.clone());
    let body = http_service::Body::from_stream(events);
    Ok(tide::http::Response::builder()
        .header(tide::http::header::CONTENT_TYPE, "text/event-stream")
        .header("X-Accel-Buffering", "no")
        .header("Cache-Control", "no-cache")
        .body(body)
        .unwrap())
}

async fn rerun(mut cx: tide::Context<AppData>) -> tide::http::Response<String> {
    let mut response = tide::http::Response::builder();
    response.header(tide::http::header::CONTENT_TYPE, "text/text; charset=utf-8");

    let vars: HashMap<String, String> = match cx.body_json().await {
        Ok(vars) => vars,
        Err(e) => {
            return response
                .status(tide::http::StatusCode::BAD_REQUEST)
                .body(e.to_string().into())
                .unwrap();
        }
    };

    let touched = {
        let mut consts = cx.state().consts.lock().unwrap();

        for (k, v) in vars {
            let k = format!("FART_USER_CONST_{}", k);
            env::set_var(&k, &v);
            consts.insert(k, v);
        }

        let mut vars = "# fart user consts\n\
                        #\n\
                        # To re-establish this user const environment, run:\n\
                        #\n\
                        #    $ source user_consts.sh\n\n\
                        "
        .to_string();
        for (k, v) in consts.iter() {
            vars.push_str(&format!("export {}={}\n", k, v));
        }

        let vars_path = cx.state().project.join("user_consts.sh");
        let wrote_consts =
            fs::write(vars_path, vars.as_bytes()).map_err(|e| failure::Error::from(e));

        wrote_consts.and_then(|_| {
            // Touch the `src` directory to get the watcher to rebuild. Kinda hacky but
            // it works!
            let src = cx.state().project.join("src");
            Command::new("touch")
                .arg(src)
                .run_result(&mut Output::Inherit)
        })
    };

    match touched {
        Ok(_) => response
            .status(tide::http::StatusCode::OK)
            .body("".into())
            .unwrap(),
        Err(e) => response
            .status(tide::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(e.to_string().into())
            .unwrap(),
    }
}

async fn image(cx: tide::Context<AppData>) -> tide::http::Response<Vec<u8>> {
    let image = cx.param::<PathBuf>("image").unwrap();
    if image.extension() != Some(OsStr::new("svg")) {
        return tide::http::Response::builder()
            .status(tide::http::StatusCode::NOT_FOUND)
            .body(vec![])
            .unwrap();
    }
    let path = cx.state().project.join("images").join(image);
    serve_static_file(path).await
}

async fn serve_static_file(path: PathBuf) -> tide::http::Response<Vec<u8>> {
    match read_file(path).await {
        Ok(contents) => tide::http::Response::builder()
            .status(tide::http::StatusCode::OK)
            .body(contents)
            .unwrap(),
        Err(e) => tide::http::Response::builder()
            .status(if e.kind() == io::ErrorKind::NotFound {
                tide::http::StatusCode::NOT_FOUND
            } else {
                tide::http::StatusCode::INTERNAL_SERVER_ERROR
            })
            .body(e.to_string().into())
            .unwrap(),
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
