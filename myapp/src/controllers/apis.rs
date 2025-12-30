use rspring_macro::*;
use rspring::*;
use axum::extract::Path;
use std::sync::Arc;

// -------------------- CORE SERVICES -------------------- //

struct UserService;
#[service]
impl UserService {
    pub fn new() -> Self { Self {} }

    pub async fn find_by_id(&self, id: &str) -> String {
        format!("Found user with ID: {}", id)
    }

    pub async fn get_greeting(&self) -> String {
        "Hello from UserService!".to_string()
    }
}

struct AuthService {
    user_service: Arc<UserService>,
}
#[service]
impl AuthService {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }

    pub async fn login(&self, username: &str, password: &str) -> bool {
        let _ = self.user_service.find_by_id(username).await; // pretend check
        username == "admin" && password == "1234"
    }

    pub async fn logout(&self, username: &str) -> String {
        format!("User {} logged out", username)
    }
}

// -------------------- ORDER ECOSYSTEM -------------------- //

struct InventoryService;
#[service]
impl InventoryService {
    pub fn new() -> Self { Self {} }

    pub async fn check_stock(&self, item_id: u32) -> bool {
        item_id % 2 == 0 // mock: even items in stock
    }

    pub async fn reserve_item(&self, item_id: u32) -> String {
        if self.check_stock(item_id).await {
            format!("Item {} reserved", item_id)
        } else {
            format!("Item {} out of stock", item_id)
        }
    }
}

struct OrderService {
    inventory_service: Arc<InventoryService>,
    user_service: Arc<UserService>,
}
#[service]
impl OrderService {
    pub fn new(
        inventory_service: Arc<InventoryService>,
        user_service: Arc<UserService>
    ) -> Self {
        Self { inventory_service, user_service }
    }

    pub async fn get_order(&self, order_id: u32) -> String {
        format!("Order {} details for user {}", order_id, self.user_service.get_greeting().await)
    }

    pub async fn place_order(&self, order_id: u32, item_id: u32) -> String {
        let reservation = self.inventory_service.reserve_item(item_id).await;
        format!("Placed order {}: {}", order_id, reservation)
    }
}

// -------------------- CONTROLLERS -------------------- //

struct ApiController {
    user_service: Arc<UserService>,
}
#[controller]
impl ApiController {
    pub fn new(user_service: Arc<UserService>) -> Self { Self { user_service } }

    #[get("/hello")]
    async fn hello(&self) -> String { self.user_service.get_greeting().await }

    #[get("/user/{id}")]
    async fn get_user(&self, Path(id): Path<String>) -> String {
        self.user_service.find_by_id(&id).await
    }
}

struct AuthController {
    auth_service: Arc<AuthService>,
}
#[controller]
impl AuthController {
    pub fn new(auth_service: Arc<AuthService>) -> Self { Self { auth_service } }

    #[post("/login/{username}/{password}")]
    async fn login(&self, Path((username, password)): Path<(String, String)>) -> String {
        if self.auth_service.login(&username, &password).await {
            format!("User {} logged in!", username)
        } else { "Login failed".to_string() }
    }

    #[post("/logout/{username}")]
    async fn logout(&self, Path(username): Path<String>) -> String {
        self.auth_service.logout(&username).await
    }
}

struct OrderController {
    order_service: Arc<OrderService>,
}
#[controller]
impl OrderController {
    pub fn new(order_service: Arc<OrderService>) -> Self { Self { order_service } }

    #[get("/order/{id}")]
    async fn get_order(&self, Path(id): Path<u32>) -> String {
        self.order_service.get_order(id).await
    }

    #[post("/order/{order_id}/item/{item_id}")]
    async fn place_order(&self, Path((order_id, item_id)): Path<(u32, u32)>) -> String {
        self.order_service.place_order(order_id, item_id).await
    }
}
