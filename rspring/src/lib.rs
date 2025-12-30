use axum::Router;
use std::sync::Arc;
use std::collections::HashMap;
use std::any::{Any, TypeId};

pub use axum;
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

// Component metadata - no factories!
pub struct Component {
    pub name: &'static str,
    pub type_id: TypeId,
    pub dependencies: &'static [TypeId],
    pub build: fn(&ServiceContainer) -> Arc<dyn Any + Send + Sync>,
}

inventory::collect!(Component);

// Container to hold all components
pub struct ServiceContainer {
    components: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl ServiceContainer {
    fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    pub fn build() -> Arc<Self> {
        let mut container = Self::new();
        let all_components: Vec<&Component> = inventory::iter::<Component>().collect();
        
        println!("[rspring] Building {} components...", all_components.len());
        
        // Build in dependency order
        let mut remaining = all_components;
        let mut attempts = 0;
        
        while !remaining.is_empty() && attempts < 100 {
            let mut built = Vec::new();
            
            for (i, comp) in remaining.iter().enumerate() {
                // Can we build this? Check if all deps are ready
                let ready = comp.dependencies.iter()
                    .all(|dep| container.components.contains_key(dep));
                
                if ready {
                    println!("[rspring]   ✓ {}", comp.name);
                    let instance = (comp.build)(&container);
                    container.components.insert(comp.type_id, instance);
                    built.push(i);
                }
            }
            
            // Remove built ones
            for &i in built.iter().rev() {
                remaining.remove(i);
            }
            
            attempts += 1;
        }
        
        if !remaining.is_empty() {
            panic!("Circular dependency! Can't build: {:?}", 
                remaining.iter().map(|c| c.name).collect::<Vec<_>>());
        }
        
        Arc::new(container)
    }
    
    pub fn get<T: 'static + Send + Sync>(&self) -> Arc<T> {
        self.components
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("{} not found in container", std::any::type_name::<T>()))
            .clone()
            .downcast::<T>()
            .expect("Downcast failed")
    }
}


pub struct Application {
    addr: String,
}

mod runtime {
    use axum::Router;
    use tokio::net::TcpListener;
    use crate::{inventory, Route, ServiceContainer};
    use axum::Extension;
    
    pub fn run(addr: String) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            // Build all components
            let container = ServiceContainer::build();
            
            let mut router = Router::new();

            for route in inventory::iter::<Route> {
                println!("[rspring] {:#?} {}", route.method, route.path);
                router = route.add_to_router(router);
            }

            // Add the container as an extension so handlers can access it
            router = router.layer(Extension(container));

            let listener = TcpListener::bind(&addr).await.unwrap();
            println!("[rspring] Server running on http://{}", addr);
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
