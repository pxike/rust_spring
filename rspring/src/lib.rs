use axum::Router;

pub use inventory;

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

pub struct Route {
    pub path: &'static str,
    pub method: Method,
    pub setup: fn(Router) -> Router, 
}

impl Route {
    pub fn add_to_router(&self, router: Router) -> Router {
        (self.setup)(router)
    }
}

inventory::collect!(Route);


pub struct Application {
    addr: String,
}

mod runtime {
    use axum::Router;
    use tokio::net::TcpListener;
    use crate::{inventory, Route};
    
    pub fn run(addr: String) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            let mut router = Router::new();

            for route in inventory::iter::<Route> {
                println!("[rspring] {:#?} {}", route.method, route.path);
                router = route.add_to_router(router);
            }

            let listener = TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, router).await.unwrap();
        });
    }
}


impl Application {
    pub fn new() -> Self {
        Self {
            addr: "127.0.0.1:3000".into(),
        }
    }

    pub fn bind(mut self, addr: &str) -> Self {
        self.addr = addr.into();
        self
    }

    pub fn run(self) {
        runtime::run(self.addr);
    }
}
