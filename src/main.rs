use std::sync::Arc;

use tokio::io::{self, AsyncWriteExt};
use tokio::fs::File;

use novelai_api::api::{ai_generate_voice, process_string_for_voice_generation};
use novelai_api::model::Configuration;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let prompt = include_str!("../TestStory.txt");
    let prompt = process_string_for_voice_generation(prompt);
    let prompt_len = prompt.len();
    let mut tasks = vec![];

    let sem = Arc::new(Semaphore::new(10));
    
    for (i, text) in prompt.iter().enumerate() {
        let text = text.clone();
        let permit = Arc::clone(&sem).acquire_owned().await;
        println!("{}", text);
        break;
        
        let task = tokio::spawn(async move {
            let _permit = permit;
            println!("{}/{}", i, prompt_len);
            let mut conf = Configuration::new();
            conf.bearer_access_token =
                Some("Token".to_string());
            let output = ai_generate_voice(&conf, &text, "Aini", -1.0, true, "v2")
                .await
                .unwrap();

            File::create(format!("./output/{:05}.webm", i)).await.unwrap().write_all(&output).await.unwrap();
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await;
    }
    Ok(())
}
