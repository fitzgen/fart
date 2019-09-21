mod events;

use crate::{
    command_ext::CommandExt, output::Output, sub_command::SubCommand, watcher::Watcher, Result,
};
use failure::ResultExt;
use futures::{channel::mpsc, FutureExt, TryFutureExt};
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
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

        let mut app = tide::Server::with_state(app_data);
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
        async_std::task::block_on(
            app.listen(format!("127.0.0.1:{}", self.port))
                .map_err(|_| ())
                .boxed(),
        )
        .map_err(|()| failure::format_err!("failed to listen on port {}", self.port))?;

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
        type Fut = futures::future::Ready<tide::Response>;

        fn call(&self, _cx: tide::Request<T>) -> Self::Fut {
            futures::future::ready(
                tide::Response::new(200)
                    .body_string(self.body.to_string())
                    .set_header("Content-Type", self.content_type),
            )
        }
    }
}

async fn events(cx: tide::Request<AppData>) -> tide::Response {
    let events = events::EventStream::new(cx.state().subscribers.clone());
    tide::Response::with_reader(200, events)
        .set_header("Content-Type", "text/event-stream")
        .set_header("X-Accel-Buffering", "no")
        .set_header("Cache-Control", "no-cache")
}

async fn rerun(mut cx: tide::Request<AppData>) -> tide::Response {
    let response = tide::Response::new(200);

    let vars: HashMap<String, String> = match cx.body_json().await {
        Ok(vars) => vars,
        Err(e) => {
            return response
                .set_status(tide::http::StatusCode::BAD_REQUEST)
                .body_string(e.to_string())
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
        Ok(_) => response.body_string("".to_string()),
        Err(e) => response
            .body_string(e.to_string())
            .set_status(tide::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn image(cx: tide::Request<AppData>) -> tide::Response {
    let image = cx.param::<PathBuf>("image").unwrap();
    if image.extension() != Some(OsStr::new("svg")) {
        return tide::Response::new(404);
    }
    let path = cx.state().project.join("images").join(image);
    serve_static_file(path).await
}

async fn serve_static_file(path: PathBuf) -> tide::Response {
    match async_std::fs::File::open(path).await {
        Ok(file) => tide::Response::with_reader(200, async_std::io::BufReader::new(file)),
        Err(e) => tide::Response::new(500).body_string(e.to_string()),
    }
}
