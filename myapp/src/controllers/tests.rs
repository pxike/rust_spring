use rspring_macro::*;
use rspring::*;


#[post("/test")]
async fn test() {
    println!("test");
}

#[delete("/test4")]
async fn test4() {
    println!("test");
}