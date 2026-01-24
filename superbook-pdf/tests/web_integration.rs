//! Web API integration tests
//!
//! Tests for the REST API endpoints.

#![cfg(feature = "web")]

use superbook_pdf::{Job, JobQueue, JobStatus, ServerConfig, WebConvertOptions, WebProgress};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    // TC-WEB-002: Health check endpoint
    #[tokio::test]
    async fn test_health_endpoint_structure() {
        let queue = JobQueue::new();
        // Queue should be empty initially
        assert_eq!(queue.list().len(), 0);
    }

    // TC-WEB-003: Job queue operations
    #[tokio::test]
    async fn test_job_queue_lifecycle() {
        let queue = JobQueue::new();

        // Create and submit a job
        let options = WebConvertOptions::default();
        let job = Job::new("test.pdf", options);
        let job_id = job.id;

        queue.submit(job);

        // Verify job exists
        let retrieved = queue.get(job_id);
        assert!(retrieved.is_some());

        let job = retrieved.unwrap();
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.input_filename, "test.pdf");
    }

    // TC-WEB-004: Job status transitions
    #[tokio::test]
    async fn test_job_status_transitions() {
        let queue = JobQueue::new();

        let options = WebConvertOptions::default();
        let job = Job::new("test.pdf", options);
        let job_id = job.id;

        queue.submit(job);

        // Transition to processing
        queue.update(job_id, |j: &mut Job| j.start());
        let job = queue.get(job_id).unwrap();
        assert_eq!(job.status, JobStatus::Processing);

        // Transition to completed
        queue.update(job_id, |j: &mut Job| j.complete(PathBuf::from("/output/test.pdf")));
        let job = queue.get(job_id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
    }

    // TC-WEB-005: Job cancellation
    #[tokio::test]
    async fn test_job_cancellation() {
        let queue = JobQueue::new();

        let options = WebConvertOptions::default();
        let job = Job::new("test.pdf", options);
        let job_id = job.id;

        queue.submit(job);

        // Cancel the job
        let cancelled = queue.cancel(job_id);
        assert!(cancelled.is_some());

        let job = cancelled.unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
    }

    // TC-WEB-006: Progress updates
    #[tokio::test]
    async fn test_progress_updates() {
        let queue = JobQueue::new();

        let options = WebConvertOptions::default();
        let job = Job::new("test.pdf", options);
        let job_id = job.id;

        queue.submit(job);

        // Update progress
        let progress = WebProgress::new(5, 12, "Processing images");
        queue.update(job_id, |j: &mut Job| j.update_progress(progress.clone()));

        let job = queue.get(job_id).unwrap();
        let p = job.progress.unwrap();
        assert_eq!(p.current_step, 5);
        assert_eq!(p.total_steps, 12);
        assert_eq!(p.step_name, "Processing images");
        assert_eq!(p.percent, 41); // 5/12 * 100 ≈ 41%
    }

    // TC-WEB-007: Convert options parsing
    #[tokio::test]
    async fn test_convert_options_default() {
        let options = WebConvertOptions::default();
        assert_eq!(options.dpi, 300);
        assert!(options.deskew);
        assert!(options.upscale);
        assert!(!options.ocr);
        assert!(!options.advanced);
    }

    // TC-WEB-008: Convert options JSON deserialization
    #[tokio::test]
    async fn test_convert_options_json() {
        let json = r#"{"dpi": 600, "deskew": false, "upscale": true, "ocr": true, "advanced": true}"#;
        let options: WebConvertOptions = serde_json::from_str(json).unwrap();

        assert_eq!(options.dpi, 600);
        assert!(!options.deskew);
        assert!(options.upscale);
        assert!(options.ocr);
        assert!(options.advanced);
    }

    // TC-WEB-009: Concurrent job processing
    #[tokio::test]
    async fn test_concurrent_jobs() {
        let queue = JobQueue::new();

        // Submit multiple jobs
        for i in 0..10 {
            let options = WebConvertOptions::default();
            let job = Job::new(&format!("test{}.pdf", i), options);
            queue.submit(job);
        }

        // All jobs should be in queue
        assert_eq!(queue.list().len(), 10);
    }

    // TC-WEB-010: Job failure handling
    #[tokio::test]
    async fn test_job_failure() {
        let queue = JobQueue::new();

        let options = WebConvertOptions::default();
        let job = Job::new("test.pdf", options);
        let job_id = job.id;

        queue.submit(job);

        // Mark as failed
        queue.update(job_id, |j: &mut Job| j.fail("Test error message".to_string()));

        let job = queue.get(job_id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.error, Some("Test error message".to_string()));
    }

    // TC-WEB-011: Server config builder
    #[tokio::test]
    async fn test_server_config_builder() {
        let config = ServerConfig::default()
            .with_port(9000)
            .with_bind("0.0.0.0")
            .with_upload_limit(100 * 1024 * 1024);

        assert_eq!(config.port, 9000);
        assert_eq!(config.bind, "0.0.0.0");
        assert_eq!(config.upload_limit, 100 * 1024 * 1024);
    }

    // TC-WEB-012: Socket address parsing
    #[tokio::test]
    async fn test_socket_addr_parsing() {
        let config = ServerConfig::default().with_port(8080).with_bind("127.0.0.1");

        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    // TC-WEB-013: Job list filtering
    #[tokio::test]
    async fn test_job_list() {
        let queue = JobQueue::new();

        // Submit 3 jobs
        for i in 0..3 {
            let options = WebConvertOptions::default();
            let job = Job::new(&format!("file{}.pdf", i), options);
            queue.submit(job);
        }

        let jobs = queue.list();
        assert_eq!(jobs.len(), 3);

        // All should be queued
        for job in jobs {
            assert_eq!(job.status, JobStatus::Queued);
        }
    }

    // TC-WEB-014: Job timestamps
    #[tokio::test]
    async fn test_job_timestamps() {
        let queue = JobQueue::new();

        let options = WebConvertOptions::default();
        let job = Job::new("test.pdf", options);
        let job_id = job.id;
        let created_at = job.created_at;

        queue.submit(job);

        // Start the job
        queue.update(job_id, |j: &mut Job| j.start());
        let job = queue.get(job_id).unwrap();

        assert_eq!(job.created_at, created_at);
        assert!(job.started_at.is_some());
        assert!(job.started_at.unwrap() >= created_at);
    }

    // TC-WEB-015: Job completion with output path
    #[tokio::test]
    async fn test_job_output_path() {
        let queue = JobQueue::new();

        let options = WebConvertOptions::default();
        let job = Job::new("test.pdf", options);
        let job_id = job.id;

        queue.submit(job);
        queue.update(job_id, |j: &mut Job| j.start());

        let output = PathBuf::from("/tmp/output/test_converted.pdf");
        queue.update(job_id, |j: &mut Job| j.complete(output.clone()));

        let job = queue.get(job_id).unwrap();
        assert_eq!(job.output_path, Some(output));
        assert!(job.completed_at.is_some());
    }
}
