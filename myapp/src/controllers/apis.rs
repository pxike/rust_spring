use rspring_macro::*;
use rspring::*;
use axum::extract::Path;
use std::sync::Arc;

// Service layer
#[service]
struct UserService {

}

impl UserService {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn find_by_id(&self, id: &str) -> String {
        // Mock implementation for now
        format!("Found user with ID: {}", id)
    }
    
    pub async fn get_greeting(&self) -> String {
        "Hello from UserService!".to_string()
    }
}

#[controller]
struct ApiController {
    user_service: Arc<UserService>,
}

// For now, handler functions must be standalone (not methods)
// TODO: Make #[get] work with methods in impl blocks

#[get("/hello")]
async fn hello() -> String {
    "Hello, world!".to_string()
}

#[get("/bye")]
async fn bye() -> String {
    "Bye!".to_string()
}

#[get("/user/{id}")]
async fn get_user(Path(id): Path<String>) -> String {
    format!("User ID is: {}", id)
}

#[get("/order/{order_id}/item/{item_id}")]
async fn get_item(Path((order_id, item_id)): Path<(u32, u32)>) -> String {
    format!("Order {} Item {}", order_id, item_id)
}
