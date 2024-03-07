use actix_request_identifier::{RequestId, RequestIdentifier};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use forcedmode::{mock::MockHardware, ConfigureMode, OperateMode, StandbyMode, TransitionError};
use tokio::sync::Mutex;
use tracing::{info, trace, warn};

// async fn dance_hardware<H>(hardware: H, id: &str) -> Result<H, TransitionError<H>>
// where
//     H: HardwareStandby,
async fn dance_hardware<H: StandbyMode>(hardware: H, id: &str) -> Result<H, TransitionError<H>> {
    let config = hardware.configure().await?;
    info!("{id} currently in state {}", config.state());
    // go back to standby
    let hardware = config.standby().await;
    let operate = hardware.operate().await?;
    info!("{id} now in state {}", operate.state());
    let hardware = operate.standby().await;
    info!("{id} currently in state {}", hardware.state());
    Ok(hardware)
}

#[get("/")]
async fn hello(data: SharedAppState, id: RequestId) -> actix_web::Result<HttpResponse> {
    let id = id.as_str();
    trace!("{id} Starting hello");
    // Try to get the hardware from the shared state.
    // It might be in use and if so we return an error.
    let hardware = data.lock().await.hardware.take().ok_or_else(|| {
        warn!("{id} Attempt to use hardware while it is currently in use");
        actix_web::error::ErrorConflict("hardware is currently in use elsewhere")
    })?;

    // At this point we've removed hardware from our shared state.
    // We need to care for it and make sure that we put it back when we're done.

    trace!("{id} Got access hardware");

    // Do a little dance with the hardware.
    // At this point, we transfer ownship of hardware to the dance_hardware function.
    // Again, we need to take care that we don't lose it.  dance_hardware must
    // give it back to us through the return type, either by succeeding or failing.
    match dance_hardware(hardware, id).await {
        Ok(hardware) => {
            // Dance complete
            // now put it back where we found it
            data.lock().await.hardware = Some(hardware);
        }
        Err(e) => {
            // Something went wrong, put the hardware back and report the error
            data.lock().await.hardware = Some(e.owner);
            return Err(actix_web::error::ErrorImATeapot(e.error));
        }
    }

    // Dance complete
    trace!("{id} Finished hello");
    Ok(HttpResponse::Ok().body("Hello world!"))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct AppState {
    hardware: Option<MockHardware>,
}
type SharedAppState = web::Data<Mutex<AppState>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    // Create the shared state.
    // Note, we have to create this outside the closure for the HttpServer
    // so that there is only one and it's not created multiple times.
    let appdata = web::Data::new(Mutex::new(AppState {
        hardware: Some(MockHardware {}),
    }));

    // Configure/run the web server
    HttpServer::new(move || {
        App::new()
            .app_data(appdata.clone())
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .wrap(RequestIdentifier::with_uuid())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
