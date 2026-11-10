//! `ag workers` commands (ag-workers).

use super::schema::read_schema;
#[allow(unused_imports)]
use crate::cli::*;
use std::path::Path;

/// Lists the background workers declared in a schema. Reads and compiles the schema;
/// requires no database or running backend.
pub(crate) fn cmd_workers_list(schema_path: &Path) -> Result<(), String> {
    let source = read_schema(schema_path)?;
    let schema = ag_dsl::compile(&source).map_err(|diags| {
        let mut msg = format!(
            "{} error(s) in '{}':\n",
            diags.iter().filter(|d| d.is_error()).count(),
            schema_path.display()
        );
        for d in &diags {
            msg.push_str(&format!("  {}\n", d.display(&source)));
        }
        msg
    })?;

    if schema.workers.is_empty() {
        println!("No workers declared in '{}'.", schema_path.display());
        return Ok(());
    }

    println!("Workers declared in '{}':", schema_path.display());
    for w in &schema.workers {
        let queue = w.queue.as_deref().unwrap_or("default");
        let mode = w.mode.as_deref().unwrap_or("static");
        let concurrency = w
            .concurrency
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_owned());
        println!(
            "  - {} (queue: {}, mode: {}, concurrency: {})",
            w.name.value, queue, mode, concurrency
        );
        if !w.input.is_empty() {
            let fields: Vec<String> = w.input.iter().map(|f| f.name.value.clone()).collect();
            println!("      input: {}", fields.join(", "));
        }
        if let Some(retry) = &w.retry {
            if let Some(max) = retry.max_attempts {
                println!("      retry: max_attempts {max}");
            }
        }
    }
    Ok(())
}

/// Explains that worker execution is hosted by the application binary (RFC-0012
/// §17.2): the generic `ag` CLI does not have the user's compiled handler registry.
#[cfg(feature = "workers-runtime")]
pub(crate) fn cmd_workers_run() -> Result<(), String> {
    println!(
        "Worker execution is hosted by your application binary, not the generic `ag` CLI:\n\
         the worker pool needs your compiled JobHandler registry, which lives in your app.\n\n\
         In your application, build the backend and start a pool:\n\n    \
         use ag_workers::{{WorkerPool, RuntimeConfig, Shutdown, QueueName}};\n    \
         let pool = WorkerPool::start(backend, registry, config, shutdown, QueueName::new(\"mail\"), 4);\n\n\
         The generic `ag workers` commands cover operations that need no handlers:\n\
         enqueue, queues, dlq (list/inspect/retry/purge), doctor."
    );
    Ok(())
}

/// Builds a durable backend from `DATABASE_URL` for the operational commands.
#[cfg(feature = "workers-runtime")]
pub(crate) async fn workers_backend() -> Result<ag_workers::PostgresQueue, String> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is required for `ag workers` runtime commands".to_string())?;
    let pool = sqlx::PgPool::connect(&url)
        .await
        .map_err(|e| format!("could not connect to the database: {e}"))?;
    Ok(ag_workers::PostgresQueue::new(pool))
}

/// Async dispatcher for the durable-backend operational commands.
#[cfg(feature = "workers-runtime")]
pub(crate) async fn cmd_workers_runtime(command: WorkersCommands) -> Result<(), String> {
    use ag_workers::QueueBackend;
    match command {
        WorkersCommands::Enqueue {
            kind,
            queue,
            payload,
            payload_version,
        } => {
            let bytes = std::fs::read(&payload)
                .map_err(|e| format!("could not read payload file {}: {e}", payload.display()))?;
            let backend = workers_backend().await?;
            let job = ag_workers::NewJob::new(
                kind.as_str(),
                queue.as_str(),
                ag_workers::PayloadBytes::from_vec(bytes),
                payload_version,
            );
            let id = backend.enqueue(job).await.map_err(|e| e.to_string())?;
            println!("enqueued {id} onto queue '{queue}' (kind {kind})");
            Ok(())
        }
        WorkersCommands::Queues => {
            let backend = workers_backend().await?;
            let names = backend.queue_names().await.map_err(|e| e.to_string())?;
            if names.is_empty() {
                println!("no queues found");
                return Ok(());
            }
            println!(
                "{:<24} {:>8} {:>10} {:>8}",
                "QUEUE", "READY", "SCHEDULED", "LEASED"
            );
            for name in names {
                let q = ag_workers::QueueName::new(name.clone());
                let d = backend.depth(&q).await.map_err(|e| e.to_string())?;
                println!(
                    "{:<24} {:>8} {:>10} {:>8}",
                    name, d.ready, d.scheduled, d.leased
                );
            }
            Ok(())
        }
        WorkersCommands::Dlq { command } => cmd_workers_dlq(command).await,
        WorkersCommands::Doctor => cmd_workers_doctor().await,
        // List and Run never reach the async dispatcher.
        WorkersCommands::List { .. } | WorkersCommands::Run => unreachable!(),
    }
}

/// `ag workers dlq ...` against the durable backend.
#[cfg(feature = "workers-runtime")]
pub(crate) async fn cmd_workers_dlq(command: DlqCommands) -> Result<(), String> {
    let backend = workers_backend().await?;
    match command {
        DlqCommands::List { queue, limit } => {
            let entries = backend
                .list_dead_letters(queue.as_deref(), limit)
                .await
                .map_err(|e| e.to_string())?;
            if entries.is_empty() {
                println!("dead-letter queue is empty");
                return Ok(());
            }
            println!(
                "{:<38} {:<20} {:<14} {:>4} REASON",
                "JOB ID", "KIND", "QUEUE", "ATT"
            );
            for e in entries {
                println!(
                    "{:<38} {:<20} {:<14} {:>4} {}",
                    e.id,
                    e.kind.as_str(),
                    e.queue.as_str(),
                    e.attempts,
                    e.reason
                );
            }
            Ok(())
        }
        DlqCommands::Inspect { job_id } => {
            let id =
                ag_workers::JobId::parse(&job_id).map_err(|e| format!("invalid job id: {e}"))?;
            match backend
                .get_dead_letter(id)
                .await
                .map_err(|e| e.to_string())?
            {
                Some(e) => {
                    println!("id:               {}", e.id);
                    println!("kind:             {}", e.kind.as_str());
                    println!("queue:            {}", e.queue.as_str());
                    println!("attempts:         {} / {}", e.attempts, e.max_attempts);
                    println!("reason:           {}", e.reason);
                    println!(
                        "last_error:       {}",
                        e.last_error.unwrap_or_else(|| "-".to_owned())
                    );
                    println!("created_at:       {}", e.created_at);
                    println!("dead_lettered_at: {}", e.dead_lettered_at);
                    Ok(())
                }
                None => Err(format!("no dead-letter entry with id {job_id}")),
            }
        }
        DlqCommands::Retry {
            job_id,
            queue,
            kind,
            limit,
            dry_run,
        } => match (job_id, queue) {
            (Some(_), Some(_)) => {
                Err("pass either a JOB_ID or --queue for a bulk re-drive, not both".to_owned())
            }
            (None, None) => Err("provide a JOB_ID, or --queue for a bulk re-drive".to_owned()),
            (Some(job_id), None) => {
                let id = ag_workers::JobId::parse(&job_id)
                    .map_err(|e| format!("invalid job id: {e}"))?;
                backend
                    .redrive_dead_letter(id)
                    .await
                    .map_err(|e| e.to_string())?;
                println!("re-enqueued {job_id} from the dead-letter queue");
                Ok(())
            }
            (None, Some(queue)) => {
                let filter = ag_workers::DlqFilter::new(&queue, limit).with_kind(kind);
                if dry_run {
                    let preview = backend
                        .preview_dead_letters(&filter)
                        .await
                        .map_err(|e| e.to_string())?;
                    print_dry_run("re-drive", &filter, &preview);
                    return Ok(());
                }
                let outcome = backend
                    .redrive_dead_letters(&filter)
                    .await
                    .map_err(|e| e.to_string())?;
                println!(
                    "re-enqueued {} dead-letter entries from {queue}",
                    outcome.affected
                );
                Ok(())
            }
        },
        DlqCommands::Purge {
            older_than,
            queue,
            kind,
            limit,
            dry_run,
        } => match queue {
            // Age-only purge across all queues (backward compatible).
            None => {
                let dur = ag_workers::parse_duration(&older_than).map_err(|e| e.to_string())?;
                let n = backend
                    .purge_dead_letters(dur)
                    .await
                    .map_err(|e| e.to_string())?;
                let plural = if n == 1 { "entry" } else { "entries" };
                println!("purged {n} dead-letter {plural} older than {older_than}");
                Ok(())
            }
            // Filtered bulk purge; age composes with the queue/kind filter.
            Some(queue) => {
                let dur = ag_workers::parse_duration(&older_than).map_err(|e| e.to_string())?;
                let filter = ag_workers::DlqFilter::new(&queue, limit)
                    .with_kind(kind)
                    .with_older_than(Some(dur));
                if dry_run {
                    let preview = backend
                        .preview_dead_letters(&filter)
                        .await
                        .map_err(|e| e.to_string())?;
                    print_dry_run("purge", &filter, &preview);
                    return Ok(());
                }
                let outcome = backend
                    .purge_dead_letters_filtered(&filter)
                    .await
                    .map_err(|e| e.to_string())?;
                println!(
                    "purged {} dead-letter entries from {queue}",
                    outcome.affected
                );
                Ok(())
            }
        },
    }
}

/// Prints what a bulk `re-drive`/`purge` would affect, without changing anything.
#[cfg(feature = "workers-runtime")]
pub(crate) fn print_dry_run(
    action: &str,
    filter: &ag_workers::DlqFilter,
    preview: &ag_workers::BulkDlqOutcome,
) {
    let kind = filter.kind.as_deref().unwrap_or("*");
    println!(
        "dry-run: {action} would affect {} dead-letter entries (queue={}, kind={}, limit={})",
        preview.affected, filter.queue, kind, filter.limit
    );
    if !preview.sample.is_empty() {
        println!("sample (up to {}):", preview.sample.len());
        for id in &preview.sample {
            println!("  {id}");
        }
    }
}

/// `ag workers doctor`: configuration + durable-backend connectivity.
#[cfg(feature = "workers-runtime")]
pub(crate) async fn cmd_workers_doctor() -> Result<(), String> {
    let cfg = ag_workers::WorkersConfig::default()
        .apply_env()
        .map_err(|e| format!("configuration error: {e}"))?;
    println!(
        "config:  mode={:?} backend={:?} max_payload_bytes={} poison_guard_attempts={}",
        cfg.mode, cfg.backend, cfg.max_payload_bytes, cfg.poison_guard_attempts
    );
    if std::env::var("DATABASE_URL").is_ok() {
        let backend = workers_backend().await?;
        backend
            .queue_names()
            .await
            .map_err(|e| format!("backend query failed: {e}"))?;
        println!("backend: connected; ag_worker_jobs reachable");
    } else {
        println!("backend: DATABASE_URL not set (durable backend unavailable)");
    }
    Ok(())
}
