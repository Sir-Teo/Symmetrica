//! Integer Partition Functions
//!
//! This module provides functions for computing integer partitions:
//! - Unrestricted partitions: p(n)
//! - Restricted partitions: partitions with constraints
//! - Partition generation

use std::collections::HashMap;

/// Compute the partition function p(n) - the number of ways to write n as a sum of positive integers
/// Uses dynamic programming with memoization
pub fn partition_count(n: u64) -> u64 {
    let mut memo = HashMap::new();
    partition_count_memo(n, &mut memo)
}

fn partition_count_memo(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
    if n == 0 {
        return 1;
    }
    if n == 1 {
        return 1;
    }

    if let Some(&cached) = memo.get(&n) {
        return cached;
    }

    // Use Euler's pentagonal number theorem for efficient computation
    // p(n) = p(n-1) + p(n-2) - p(n-5) - p(n-7) + p(n-12) + p(n-15) - ...
    // where the indices are generalized pentagonal numbers: k(3k-1)/2 for k = ±1, ±2, ±3, ...

    let mut result = 0i64;
    let mut k = 1i64;

    loop {
        // Positive k
        let pent_pos = (k * (3 * k - 1)) / 2;
        if pent_pos > n as i64 {
            break;
        }
        let sign = if k % 2 == 1 { 1 } else { -1 };
        result += sign * partition_count_memo((n as i64 - pent_pos) as u64, memo) as i64;

        // Negative k
        let pent_neg = (k * (3 * k + 1)) / 2;
        if pent_neg > n as i64 {
            break;
        }
        result += sign * partition_count_memo((n as i64 - pent_neg) as u64, memo) as i64;

        k += 1;
    }

    let final_result = result as u64;
    memo.insert(n, final_result);
    final_result
}

/// Generate all partitions of n
/// Returns a vector of partitions, where each partition is a vector of parts in descending order
pub fn generate_partitions(n: u64) -> Vec<Vec<u64>> {
    if n == 0 {
        return vec![vec![]];
    }

    let mut result = Vec::new();
    generate_partitions_helper(n, n, &mut vec![], &mut result);
    result
}

fn generate_partitions_helper(
    n: u64,
    max_part: u64,
    current: &mut Vec<u64>,
    result: &mut Vec<Vec<u64>>,
) {
    if n == 0 {
        result.push(current.clone());
        return;
    }

    for part in (1..=max_part.min(n)).rev() {
        current.push(part);
        generate_partitions_helper(n - part, part, current, result);
        current.pop();
    }
}

/// Count partitions of n into exactly k parts
pub fn partition_count_k_parts(n: u64, k: u64) -> u64 {
    if k == 0 {
        return if n == 0 { 1 } else { 0 };
    }
    if n == 0 || k > n {
        return 0;
    }
    if k == 1 {
        return 1;
    }

    // Use dynamic programming: p(n, k) = p(n-1, k-1) + p(n-k, k)
    // p(n-1, k-1): partitions where smallest part is 1
    // p(n-k, k): partitions where all parts are ≥ 2 (subtract 1 from each part)
    let mut dp = vec![vec![0u64; (k + 1) as usize]; (n + 1) as usize];

    // Base cases
    for (i, row) in dp.iter_mut().enumerate().take(n as usize + 1) {
        row[0] = if i == 0 { 1 } else { 0 };
    }
    for row in dp.iter_mut().take(n as usize + 1).skip(1) {
        row[1] = 1;
    }

    // Fill DP table
    for i in 2..=n as usize {
        for j in 2..=k.min(i as u64) as usize {
            dp[i][j] = dp[i - 1][j - 1];
            if i >= j {
                dp[i][j] += dp[i - j][j];
            }
        }
    }

    dp[n as usize][k as usize]
}

/// Count partitions of n into distinct parts (no repeated parts)
pub fn partition_count_distinct(n: u64) -> u64 {
    if n == 0 {
        return 1;
    }

    // Use DP: q(n, k) = number of partitions of n using parts ≤ k
    let mut dp = vec![0u64; (n + 1) as usize];
    dp[0] = 1;

    for part in 1..=n {
        // Process in reverse to avoid using the same part twice
        for i in (part..=n).rev() {
            dp[i as usize] += dp[(i - part) as usize];
        }
    }

    dp[n as usize]
}

/// Count partitions of n into odd parts only
pub fn partition_count_odd_parts(n: u64) -> u64 {
    if n == 0 {
        return 1;
    }

    let mut dp = vec![0u64; (n + 1) as usize];
    dp[0] = 1;

    // Only use odd parts
    let mut part = 1u64;
    while part <= n {
        for i in part..=n {
            dp[i as usize] += dp[(i - part) as usize];
        }
        part += 2; // Next odd number
    }

    dp[n as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_count_small() {
        assert_eq!(partition_count(0), 1); // Empty partition
        assert_eq!(partition_count(1), 1); // {1}
        assert_eq!(partition_count(2), 2); // {2}, {1,1}
        assert_eq!(partition_count(3), 3); // {3}, {2,1}, {1,1,1}
        assert_eq!(partition_count(4), 5); // {4}, {3,1}, {2,2}, {2,1,1}, {1,1,1,1}
        assert_eq!(partition_count(5), 7);
    }

    #[test]
    fn test_partition_count_larger() {
        assert_eq!(partition_count(10), 42);
        assert_eq!(partition_count(15), 176);
        assert_eq!(partition_count(20), 627);
    }

    #[test]
    fn test_generate_partitions_small() {
        let partitions = generate_partitions(4);
        assert_eq!(partitions.len(), 5);
        assert!(partitions.contains(&vec![4]));
        assert!(partitions.contains(&vec![3, 1]));
        assert!(partitions.contains(&vec![2, 2]));
        assert!(partitions.contains(&vec![2, 1, 1]));
        assert!(partitions.contains(&vec![1, 1, 1, 1]));
    }

    #[test]
    fn test_generate_partitions_verify_count() {
        for n in 1..=7 {
            let partitions = generate_partitions(n);
            assert_eq!(partitions.len() as u64, partition_count(n));
        }
    }

    #[test]
    fn test_partition_count_k_parts() {
        // p(5, 2) = 2: {4,1}, {3,2}
        assert_eq!(partition_count_k_parts(5, 2), 2);

        // p(6, 3) = 3: {4,1,1}, {3,2,1}, {2,2,2}
        assert_eq!(partition_count_k_parts(6, 3), 3);

        // p(n, 1) = 1 for all n > 0
        assert_eq!(partition_count_k_parts(10, 1), 1);

        // p(n, n) = 1 for all n > 0 (all 1's)
        assert_eq!(partition_count_k_parts(5, 5), 1);
    }

    #[test]
    fn test_partition_count_distinct() {
        // Partitions of 5 into distinct parts: {5}, {4,1}, {3,2}
        assert_eq!(partition_count_distinct(5), 3);

        // Partitions of 6 into distinct parts: {6}, {5,1}, {4,2}, {3,2,1}
        assert_eq!(partition_count_distinct(6), 4);

        assert_eq!(partition_count_distinct(10), 10);
    }

    #[test]
    fn test_partition_count_odd_parts() {
        // Partitions of 5 into odd parts: {5}, {3,1,1}, {1,1,1,1,1}
        assert_eq!(partition_count_odd_parts(5), 3);

        // Partitions of 6 into odd parts: {5,1}, {3,3}, {3,1,1,1}, {1,1,1,1,1,1}
        assert_eq!(partition_count_odd_parts(6), 4);
    }

    #[test]
    fn test_euler_theorem() {
        // Euler's theorem: number of partitions into distinct parts
        // equals number of partitions into odd parts
        for n in 1..=15 {
            assert_eq!(
                partition_count_distinct(n),
                partition_count_odd_parts(n),
                "Euler's theorem failed for n={}",
                n
            );
        }
    }
}
