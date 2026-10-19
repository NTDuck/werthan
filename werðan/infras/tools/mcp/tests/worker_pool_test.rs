//! Unit tests for worker-pool.

#[::tokio::test]
async fn test_worker_pool_concurrency() {
    let pool = ::mcp::worker_pool::WorkerPool::new(2, "qwen3.6-35b-a3b");
    let shards = ::std::vec!["file1.c".to_string(), "file2.c".to_string(), "file3.c".to_string(),];
    let res = pool.dispatch_shards(&shards).await;
    ::std::assert!(res.is_ok());
    let (cards, used_real, real_count) = res.unwrap();
    ::std::assert_eq!(cards.len(), 3);
    ::std::assert!(!used_real);
    ::std::assert_eq!(real_count, 0);
}
