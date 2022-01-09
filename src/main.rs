use std::env;

mod kcc3;

#[tokio::main]
async fn main() {
    let kcc3_url = env::var("KCC3_URL").expect("Expected KCC3 URL in the environment");
    let kcc3_token = env::var("KCC3_TOKEN").expect("Expected KCC3 token in the environment");

    let kcc3client = kcc3::Kcc3Client::new(kcc3_url, &kcc3_token).unwrap();
    println!("{:?}", kcc3client.get_chombos().await.unwrap());
}
