#[macro_use]
extern crate lazy_static;

use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Once;
use std::time::Duration;

use deno_core::anyhow::Error;
use deno_core::serde_v8;
use deno_core::v8;
use deno_core::JsRuntime;
use deno_core::ModuleSpecifier;
use deno_core::RuntimeOptions;
use deno_simple_module_loader::SimpleModuleLoader;

lazy_static! {
    static ref SERVE_PATH: PathBuf = {
        let mut path = PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/files");
        path
    };
}

static START_SERVER: Once = Once::new();

async fn ensure_server_is_running() {
    let mut started_now = false;
    START_SERVER.call_once(|| {
        started_now = true;
        std::thread::spawn(|| {
            use std::net::SocketAddr;
            use std::str::FromStr;
            use warp::path::Tail;
            use warp::Filter;

            let static_filter = warp::fs::dir(&*SERVE_PATH);
            let redirect_filter =
                warp::path("redirect")
                    .and(warp::path::tail())
                    .map(|tail: Tail| {
                        let redirect_uri =
                            format!("/{}", tail.as_str()).parse::<http::Uri>().unwrap();
                        warp::redirect::found(redirect_uri)
                    });
            let serve_future = warp::serve(redirect_filter.or(static_filter))
                .run(SocketAddr::from_str("127.0.0.1:8888").unwrap());

            tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build()
                .unwrap()
                .block_on(serve_future);
        });
    });

    if started_now {
        // Give the server some time to start.
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

// Server test. It's conditionally compiled out (`any()` is never true).
#[cfg(any())]
#[tokio::test]
async fn server_test() {
    ensure_server_is_running().await;
    // Keep the server running forever.
    std::future::pending::<()>().await;
}

fn url_from_test_path<P: AsRef<Path>>(path: P) -> ModuleSpecifier {
    let resolved = SERVE_PATH.join(path);
    ModuleSpecifier::from_file_path(resolved).unwrap()
}

fn setup_runtime() -> Result<JsRuntime, Error> {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(SimpleModuleLoader)),
        ..Default::default()
    });
    runtime.execute_script(
        "<setup>",
        r#"
            (() => {
                let output = "";

                globalThis.console = {
                    log(...args) {
                        output += args.map(String).join(" ") + "\n";
                    }
                };

                globalThis.getOutput = function getOutput() {
                    return output;
                };
            })();
        "#,
    )?;
    Ok(runtime)
}

fn get_output(runtime: &mut JsRuntime) -> Result<String, Error> {
    let value = runtime.execute_script("<output>", "globalThis.getOutput();")?;
    let scope = &mut runtime.handle_scope();
    let local_value = v8::Local::new(scope, value);
    Ok(serde_v8::from_v8(scope, local_value)?)
}

#[tokio::test]
async fn basic_test() -> Result<(), Error> {
    ensure_server_is_running().await;

    let mut runtime = setup_runtime()?;
    let module_id = runtime
        .load_main_module(&url_from_test_path("basic_main.js"), None)
        .await?;
    let receiver = runtime.mod_evaluate(module_id);
    runtime.run_event_loop(false).await?;
    receiver.await??;

    assert_eq!(
        get_output(&mut runtime)?,
        format!(
            "test1.js http://localhost:8888/test1.js\nbasic_main.js {}\nData URL value: 42\n",
            url_from_test_path("basic_main.js")
        )
    );
    Ok(())
}

#[tokio::test]
async fn http_redirect_test() -> Result<(), Error> {
    ensure_server_is_running().await;

    let mut runtime = setup_runtime()?;
    let module_id = runtime
        .load_main_module(&url_from_test_path("http_redirect_main.js"), None)
        .await?;
    let receiver = runtime.mod_evaluate(module_id);
    runtime.run_event_loop(false).await?;
    receiver.await??;

    assert_eq!(
        get_output(&mut runtime)?,
        "test1.js http://localhost:8888/test1.js\n"
    );
    Ok(())
}
