//! `#[tokio::test(start_paused = true)]` freezes Tokio's clock: when every
//! task is waiting on a timer, the runtime jumps straight to the next
//! deadline. A test of "two 1-second tasks take 1 second, not 2" runs in
//! microseconds and measures *exactly* 1s of virtual time.
//!
//! Tests that do real network I/O can't use a paused clock (the runtime
//! would fast-forward through timeouts while the socket is busy), so they
//! use plain `#[tokio::test]`.

use crate::sut::*;
use std::time::Duration;

const MS: fn(u64) -> Duration = Duration::from_millis;

// ---- Exercise 1 --------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn ex1_values_chaining_and_errors() {
    use ex01_basics::*;
    assert_eq!(say_hello().await, "Hello");
    assert_eq!(hello_world().await, "Hello World");
    assert_eq!(add_async(2, 3).await, 5);
    assert_eq!(sum_strings("10", " 32 ").await, Ok(42));
    assert!(sum_strings("10", "x").await.is_err());
}

#[tokio::test]
async fn ex1_futures_are_lazy() {
    assert_eq!(ex01_basics::futures_are_lazy().await, (false, true));
}

// ---- Exercise 2 --------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn ex2_concurrent_takes_the_longest_not_the_sum() {
    use ex02_concurrency::*;
    let (r, t) = sequential().await;
    assert_eq!(r, vec!["Task 1 complete", "Task 2 complete"]);
    assert_eq!(t.as_secs(), 3);
    let (r, t) = concurrent().await;
    assert_eq!(r, vec!["Task 1 complete", "Task 2 complete"]);
    assert_eq!(t.as_secs(), 2);
}

#[tokio::test(start_paused = true)]
async fn ex2_ten_tasks_in_input_order() {
    let (results, t) = ex02_concurrency::ten_concurrently().await;
    assert_eq!(results, (0..10).collect::<Vec<_>>());
    assert!(t <= MS(1_001), "all ten overlap: took {t:?}");
}

#[tokio::test(start_paused = true)]
async fn ex2_select_and_timeout() {
    use ex02_concurrency::*;
    assert_eq!(first_to_finish().await, "task one");
    assert_eq!(with_timeout(MS(500), task_two()).await, None);
    assert_eq!(
        with_timeout(MS(1_500), task_one()).await.as_deref(),
        Some("Task 1 complete")
    );
}

// ---- Exercise 3 --------------------------------------------------------------

#[tokio::test]
async fn ex3_file_round_trip_copy_and_lines() {
    use ex03_file_io::*;
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    assert_eq!(
        write_then_read(&a, "Hello, async world!").await.unwrap(),
        "Hello, async world!"
    );
    let b = dir.path().join("b.txt");
    assert_eq!(copy_file(&a, &b).await.unwrap(), 19);

    let results = read_many(&[a.clone(), dir.path().join("missing"), b.clone()]).await;
    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok() && results[2].is_ok());
    assert_eq!(
        results[1].as_ref().unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );

    let log = dir.path().join("log.txt");
    std::fs::write(&log, "ok\nERROR one\nok\nERROR two\n").unwrap();
    assert_eq!(count_matching_lines(&log, "ERROR").await.unwrap(), 2);
}

// ---- Exercise 4 --------------------------------------------------------------

#[tokio::test]
async fn ex4_fetch_and_decode() {
    use ex04_http_client::*;
    let base = mock_api::spawn().await;
    let http = client(MS(1_000));
    let post = fetch_post(&http, &base, 12).await.unwrap();
    assert_eq!(
        post,
        Post {
            user_id: 2,
            id: 12,
            title: "post number 12".into(),
            body: "lorem ipsum".into()
        }
    );
    let err = fetch_post(&http, &base, 0).await.unwrap_err();
    assert_eq!(
        err.status(),
        Some(reqwest::StatusCode::NOT_FOUND),
        "error_for_status turns 404 into Err"
    );
}

#[tokio::test]
async fn ex4_fetch_many_keeps_order_and_individual_errors() {
    use ex04_http_client::*;
    let base = mock_api::spawn().await;
    let results = fetch_many(&client(MS(1_000)), &base, &[3, 0, 1]).await;
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].as_ref().unwrap().id, 3);
    assert!(results[1].is_err());
    assert_eq!(results[2].as_ref().unwrap().id, 1);
}

#[tokio::test]
async fn ex4_retry_recovers_from_5xx_and_gives_up_on_timeouts() {
    use ex04_http_client::*;
    let base = mock_api::spawn().await;
    let http = client(MS(300));
    let (text, attempts) = get_text_with_retry(&http, &format!("{base}/flaky"), 5, MS(1))
        .await
        .unwrap();
    assert_eq!(
        (text.as_str(), attempts),
        ("finally", 3),
        "the mock fails twice, then succeeds"
    );

    let err = get_text_with_retry(&http, &format!("{base}/slow"), 2, MS(1))
        .await
        .unwrap_err();
    assert!(err.is_timeout());

    let not_found = get_text_with_retry(&http, &format!("{base}/posts/0"), 5, MS(1))
        .await
        .unwrap_err();
    assert_eq!(
        not_found.status(),
        Some(reqwest::StatusCode::NOT_FOUND),
        "4xx is not retried"
    );
}

// ---- Exercise 5 --------------------------------------------------------------

#[tokio::test]
async fn ex5_mpsc() {
    use ex05_channels::*;
    assert_eq!(single_producer().await, (0..10).collect::<Vec<_>>());
    let got = multiple_producers(3, 4).await;
    assert_eq!(got.len(), 12);
    assert_eq!(got.first(), Some(&(0, 0)));
    assert_eq!(got.last(), Some(&(2, 3)));
}

#[tokio::test]
async fn ex5_broadcast_and_oneshot() {
    use ex05_channels::*;
    let all = broadcast_to(3, &["a", "b", "c"]).await;
    assert_eq!(
        all,
        vec![vec!["a", "b", "c"]; 3],
        "every subscriber gets every message"
    );
    let worker = spawn_squarer();
    assert_eq!(ask_square(&worker, 12).await, Some(144));
    assert_eq!(ask_square(&worker, 0).await, Some(0));
}

// ---- Exercise 6 --------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn ex6_spawn_join_abort() {
    use ex06_tasks::*;
    let results: Vec<u64> = spawn_ten().await.into_iter().map(Result::unwrap).collect();
    assert_eq!(results, (0..10).map(|i| i * i).collect::<Vec<_>>());
    assert!(abort_a_task().await);
    assert_eq!(cancel_cooperatively(MS(350)).await, 3);
    assert_eq!(joinset_sum(10).await, 385);
}

#[tokio::test(start_paused = true)]
async fn ex6_semaphore_limits_concurrency() {
    use ex06_tasks::limited_concurrency;
    assert_eq!(limited_concurrency(20, 3).await, 3);
    assert_eq!(limited_concurrency(2, 5).await, 2);
}

#[tokio::test]
async fn ex6_spawn_blocking() {
    assert_eq!(ex06_tasks::cpu_heavy_sum(100).await, 5050);
}

// ---- Exercise 7 --------------------------------------------------------------

#[tokio::test]
async fn ex7_routes_state_errors_and_middleware() {
    let base = ex07_web_server::spawn("127.0.0.1:0").await;
    let http = reqwest::Client::new();

    let root: serde_json::Value = http.get(&base).send().await.unwrap().json().await.unwrap();
    assert_eq!(root["content"], "Hello, async!");

    let created = http
        .post(format!("{base}/todos"))
        .json(&serde_json::json!({"title": " write tests "}))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);
    let todo: ex07_web_server::Todo = created.json().await.unwrap();
    assert_eq!(
        (todo.id, todo.title.as_str(), todo.done),
        (1, "write tests", false)
    );

    let bad = http
        .post(format!("{base}/todos"))
        .json(&serde_json::json!({"title": ""}))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 422);

    let done: ex07_web_server::Todo = http
        .patch(format!("{base}/todos/1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(done.done);

    let list: Vec<ex07_web_server::Todo> = http
        .get(format!("{base}/todos"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list, vec![done]);

    let missing = http.get(format!("{base}/todos/99")).send().await.unwrap();
    assert_eq!(missing.status(), 404);
    assert_eq!(
        missing.headers()["x-request-count"],
        "6",
        "middleware counted every request"
    );
    let body: serde_json::Value = missing.json().await.unwrap();
    assert_eq!(body["error"], "todo 99 not found");
}

// ---- Exercise 8 --------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn ex8_streams() {
    use ex08_streams::*;
    use futures::StreamExt;
    assert_eq!(
        iterate_and_double().await,
        (vec![1, 2, 3, 4, 5], vec![2, 4, 6])
    );
    let ticks = ticks(4, MS(20)).await;
    let ms: Vec<u128> = ticks.iter().map(|d| d.as_millis()).collect();
    assert_eq!(ms, vec![0, 20, 40, 60]);
    assert_eq!(even_squares(10).await, vec![4, 16, 36, 64, 100]);
    assert_eq!(countdown(3).collect::<Vec<_>>().await, vec![3, 2, 1]);
}

#[tokio::test(start_paused = true)]
async fn ex8_merge_interleaves_by_arrival() {
    let merged = ex08_streams::merged(9).await;
    assert_eq!(merged.len(), 9);
    // In the first 50ms: fast ticks at 0,10,20,30,40 and slow ticks at 0,25.
    assert_eq!(
        merged.iter().filter(|s| **s == "slow").count(),
        2 + 1,
        "slow at 0, 25 and 50"
    );
}

// ---- Bonus -------------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn bonus_shared_receiver_pool_processes_every_job_once() {
    use bonus_task_queue::*;
    let jobs = (0..20).map(|i| Job::new(i, 0, "x")).collect();
    let mut done = shared_receiver_pool(4, jobs).await;
    assert_eq!(done.len(), 20);
    done.sort_by_key(|(_, id)| *id);
    assert_eq!(
        done.iter().map(|(_, id)| *id).collect::<Vec<_>>(),
        (0..20).collect::<Vec<_>>()
    );
    let workers_used: std::collections::BTreeSet<usize> = done.iter().map(|(w, _)| *w).collect();
    assert_eq!(workers_used.len(), 4, "the work was shared");
}

#[tokio::test(start_paused = true)]
async fn bonus_priority_retry_and_results() {
    use bonus_task_queue::*;
    let jobs = vec![
        Job::new(1, 1, "low"),
        Job::new(2, 9, "urgent"),
        Job::new(3, 5, "flaky").failing(2),
        Job::new(4, 5, "doomed").failing(10),
        Job::new(5, 9, "also urgent"),
    ];
    let config = QueueConfig {
        workers: 1,
        max_attempts: 3,
        ..QueueConfig::default()
    };
    let results = run_queue(jobs, config).await;
    let order: Vec<u64> = results.iter().map(|r| r.id).collect();
    assert_eq!(
        order,
        vec![2, 5, 3, 4, 1],
        "highest priority first, lower id breaks ties"
    );
    assert_eq!(
        results[2].outcome,
        Outcome::Succeeded {
            attempts: 3,
            output: "FLAKY".into()
        }
    );
    assert_eq!(results[3].outcome, Outcome::Failed { attempts: 3 });
}

#[tokio::test(start_paused = true)]
async fn bonus_rate_limit() {
    use bonus_task_queue::*;
    let jobs = (0..10).map(|i| Job::new(i, 0, "x")).collect();
    let config = QueueConfig {
        workers: 4,
        jobs_per_second: Some(5),
        work_time: MS(1),
        ..QueueConfig::default()
    };
    let start = tokio::time::Instant::now();
    let results = run_queue(jobs, config).await;
    assert_eq!(results.len(), 10);
    // 5/s: ticks at 0, 200, ..., 1800ms -- four workers can't go faster.
    assert!(start.elapsed() >= MS(1_800), "took {:?}", start.elapsed());
}
