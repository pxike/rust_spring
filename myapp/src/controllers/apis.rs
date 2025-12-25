use rspring_macro::*;
use rspring::*;
use axum::extract::Path;

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
