use anyhow::{Context, Result, anyhow};
use app::App;
use clap::{Parser, Subcommand};
use domain::ids::ObservationId;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "crabtrap", version, about)]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Migrate,

    IngestText {
        #[arg(long)]
        content: Option<String>,

        #[arg(long)]
        file: Option<PathBuf>,

        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        source_url: Option<String>,
    },

    GetObservation {
        id: ObservationId,
    },

    Chunk {
        observation_id: ObservationId,

        #[arg(long, default_value_t = 1000)]
        chunk_size: usize,
    },

    ListChunks {
        observation_id: ObservationId,
    },
}

fn resolve_content(content: Option<String>, file: Option<PathBuf>) -> Result<String> {
    match (content, file) {
        (Some(content), None) => Ok(content),
        (None, Some(path)) => {
            std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
        }
        (None, None) => Err(anyhow!("provide either --content or --file")),
        (Some(_), Some(_)) => Err(anyhow!("provide only one of --content or --file")),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let app = App::connect(&cli.database_url).await?;

    match cli.command {
        Command::Migrate => {
            app.migrate().await?;
            println!("ok: migrated");
        }

        Command::IngestText {
            content,
            file,
            title,
            source_url,
        } => {
            let content = resolve_content(content, file)?;
            let (id, inserted) = app.ingest_text(content, title, source_url).await?;
            if inserted {
                println!("ok: inserted observation {id}");
            } else {
                println!("ok: existing observation {id}");
            }
        }

        Command::GetObservation { id } => {
            let Some(obs) = app.get_observation(id).await? else {
                println!("not found: observation {id}");
                return Ok(());
            };

            println!("id: {}", obs.id());
            println!("hash: {}", obs.content_hash());
            println!("source_kind: {}", obs.source_kind().as_str());
            println!("created_at: {}", obs.created_at());
            if let Some(published_at) = obs.published_at() {
                println!("published_at: {published_at}");
            }
            if let Some(title) = obs.title() {
                println!("title: {title}");
            }
            if let Some(source_url) = obs.source_url() {
                println!("source_url: {source_url}");
            }
            println!("content_bytes: {}", obs.content().len());
        }

        Command::Chunk {
            observation_id,
            chunk_size,
        } => {
            let Some(n) = app.chunk_observation(observation_id, chunk_size).await? else {
                println!("not found: observation {observation_id}");
                return Ok(());
            };
            println!("ok: upserted {n} chunks");
        }

        Command::ListChunks { observation_id } => {
            let chunks = app.list_chunks(observation_id).await?;
            for c in chunks {
                println!(
                    "{} idx={} bytes={} start={} end={} tokens={}",
                    c.id(),
                    c.index(),
                    c.text().len(),
                    c.start_offset(),
                    c.end_offset(),
                    c.token_estimate()
                );
            }
        }
    }

    Ok(())
}
