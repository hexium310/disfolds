use std::{collections::HashMap, env, sync::Arc};

use anyhow::Result;
use serenity::{client::Client, futures::lock::Mutex, model::gateway::GatewayIntents, prelude::TypeMapKey};
use songbird::{
    driver::Bitrate,
    input::{cached::Compressed, File},
    SerenityInit,
};

mod commands;
mod event_handler;
mod utils;
mod voicevox;

struct SoundStore;

impl TypeMapKey for SoundStore {
    type Value = Arc<Mutex<HashMap<String, Compressed>>>;
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("`DISCORD_TOKEN` is not set.");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(event_handler::Handler)
        .register_songbird()
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;

        let mut audio_map = HashMap::new();

        let resources = vec![
            ("CODE", "resources/code.wav"),
            ("URL", "resources/url.wav")
        ];
        for resource in resources {
            let audio = set_up_audio(resource.1).await.unwrap();
            audio_map.insert(resource.0.into(), audio);
        }

        data.insert::<SoundStore>(Arc::new(Mutex::new(audio_map)));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

async fn set_up_audio(path: &'static str) -> Result<Compressed> {
    let url = Compressed::new(File::new(path).into(), Bitrate::BitsPerSecond(128_000)).await?;
    let _ = url.raw.spawn_loader();

    Ok(url)
}