use num_cpus;

/// Calculates optimal concurrency settings for parallel processing
///
/// Returns a tuple of (max_concurrent, threads_per_job) where:
/// - `max_concurrent`: Maximum number of jobs to run simultaneously
/// - `threads_per_job`: Number of threads each job should use (for CPU-intensive tasks like video encoding)
///
/// # Arguments
/// * `job_count` - Total number of jobs to process
/// * `max_concurrent_cap` - Optional cap on max_concurrent (defaults to 4 if None)
///
/// # Examples
/// ```
/// // 4 videos, 4 CPUs → (4, 1) - 4 concurrent, 1 thread each
/// let (max, threads) = calculate_optimal_concurrency(4, None);
///
/// // 5 videos, 4 CPUs → (4, 1) - 4 concurrent, 1 thread each  
/// let (max, threads) = calculate_optimal_concurrency(5, None);
///
/// // 2 videos, 8 CPUs → (2, 4) - 2 concurrent, 4 threads each
/// let (max, threads) = calculate_optimal_concurrency(2, None);
/// ```
pub fn calculate_optimal_concurrency(
    job_count: usize,
    max_concurrent_cap: Option<usize>,
) -> (usize, usize) {
    let cpu_count = num_cpus::get();
    let cap = max_concurrent_cap.unwrap_or(4);

    // Calculate max_concurrent: min of job_count, cpu_count, and cap
    // This ensures we don't oversubscribe CPUs and don't create too much overhead
    let max_concurrent = job_count.min(cpu_count).min(cap);

    // Calculate threads per job: divide CPUs evenly among concurrent jobs
    // Ensure at least 1 thread per job
    let threads_per_job = (cpu_count / max_concurrent).max(1);

    (max_concurrent, threads_per_job)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrency_calculation() {
        // Mock CPU count by testing with known values
        // Note: These tests assume a system with at least 4 CPUs
        let cpu_count = num_cpus::get();

        // If we have 4+ CPUs, test the logic
        if cpu_count >= 4 {
            let (max, threads) = calculate_optimal_concurrency(4, None);
            assert_eq!(max, 4);
            assert!(threads >= 1);

            let (max, threads) = calculate_optimal_concurrency(5, None);
            assert_eq!(max, 4); // Capped at 4
            assert!(threads >= 1);

            let (max, threads) = calculate_optimal_concurrency(2, None);
            assert_eq!(max, 2);
            assert!(threads >= 1);
        }
    }
}
