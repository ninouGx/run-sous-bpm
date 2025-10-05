use run_sous_bpm_core::database::establish_db_connection;
use run_sous_bpm_core::services::oauth::{ spotify_test_oauth, strava_test_oauth };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //spotify_test_oauth().await?;
    strava_test_oauth().await;

    Ok(())
}
