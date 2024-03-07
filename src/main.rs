use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use forcedmode::{
    HardwareConfigure, HardwareOperate, HardwareStandby, MockHardware, TransitionError,
};
use tokio::sync::Mutex;

fn dance_hardware<H>(hardware: H) -> Result<H, TransitionError<H>>
where
    H: HardwareStandby,
{
    let config = hardware.configure()?;
    println!("currently in state {}", config.state());
    // go back to standby
    let hardware = config.standby();
    let operate = hardware.operate()?;
    println!("now in state {}", operate.state());
    let hardware = operate.standby();
    println!("currently in state {}", hardware.state());
    Ok(hardware)
}

#[get("/")]
async fn hello(data: SharedAppState) -> actix_web::Result<HttpResponse> {
    // Try to get the hardware from the shared state.
    // It might be in use and if so we return an error.
    let hardware =
        data.lock().await.hardware.take().ok_or_else(|| {
            actix_web::error::ErrorConflict("hardware is currently in use elsewhere")
        })?;

    // Do a little dance with the hardware
    match dance_hardware(hardware) {
        Ok(hardware) => {
            // Dance complete
            // now put it back where we found it
            data.lock().await.hardware = Some(hardware);
        }
        Err(e) => {
            // Something went wrong, put the hardware back and report the error
            data.lock().await.hardware = Some(e.me);
            return Err(actix_web::error::ErrorImATeapot(e.error));
        }
    }

    // Dance complete
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
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Mutex::new(AppState {
                hardware: Some(MockHardware::new()),
            })))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
