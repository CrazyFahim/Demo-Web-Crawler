use anyhow::Result;
use std::process;

#[tokio::main]

async fn main(){
    match  try_main().await{
        Ok(_) => {
            log::info!("Finished!");
        },
        Err(e) => {
            log::error!("Cooked! {:?}", e);
            process::exit(-1);

        }
    }
}

async fn try_main()-> Result<()> {
    env_logger::init();

    let resp = reqwest::get("https://google.com")
        .await?;
    println!("{:?}", resp.text().await);

    log::info!("Hello, World!");
    Ok(())
}


