use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    size: Option<usize>,
    mean_time_ns: f64,
    outliers: OutlierInfo,
}

#[derive(Debug, Default)]
struct OutlierInfo {
    low_severe: usize,
    low_mild: usize,
    high_mild: usize,
    high_severe: usize,
}

impl OutlierInfo {
    fn total(&self) -> usize {
        self.low_severe + self.low_mild + self.high_mild + self.high_severe
    }

    fn percentage(&self, sample_count: usize) -> f64 {
        (self.total() as f64 / sample_count as f64) * 100.0
    }
}

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let criterion_dir = cwd.join("target/criterion");

    println!("Criterion directory: {:?}", criterion_dir);

    // Parse all benchmark results
    let mut results: Vec<BenchmarkResult> = Vec::new();

    let entries: Vec<_> = fs::read_dir(&criterion_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    println!("Found {} entries in criterion directory", entries.len());

    for entry in entries {
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }

        let bench_name = entry.file_name().to_string_lossy().to_string();
        if bench_name == "report" {
            continue;
        }

        // Look for size-based benchmarks
        let sub_entries: Vec<_> = fs::read_dir(entry.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        for sub_entry in sub_entries {
            if !sub_entry.file_type().unwrap().is_dir() {
                continue;
            }

            let sub_name = sub_entry.file_name().to_string_lossy().to_string();

            // Try to parse size from directory name
            let size = sub_name.parse::<usize>().ok().or_else(|| {
                // For nested benchmarks like "parent_idxs/10"
                if let Ok(nested) = fs::read_dir(sub_entry.path()) {
                    for nested_entry in nested {
                        if let Ok(ne) = nested_entry {
                            if let Ok(s) = ne.file_name().to_string_lossy().parse::<usize>() {
                                return Some(s);
                            }
                        }
                    }
                }
                None
            });

            // Try to find benchmark data directory (prefer new/ over base/)
            let data_dir = if sub_entry.path().join("new").exists() {
                Some(sub_entry.path().join("new"))
            } else if sub_entry.path().join("base").exists() {
                Some(sub_entry.path().join("base"))
            } else {
                None
            };

            if let Some(dir) = data_dir {
                if let Some(result) = parse_benchmark(&bench_name, &sub_name, size, &dir) {
                    results.push(result);
                }
            } else {
                // Try nested structure (e.g., collect_all_versions/10/new)
                if let Ok(nested) = fs::read_dir(sub_entry.path()) {
                    for nested_entry in nested {
                        if let Ok(ne) = nested_entry {
                            if !ne.file_type().unwrap().is_dir() {
                                continue;
                            }

                            let nested_name = ne.file_name().to_string_lossy().to_string();
                            if nested_name == "report" {
                                continue;
                            }

                            let nested_size = nested_name.parse::<usize>().ok();
                            let full_name = format!("{}/{}", sub_name, nested_name);

                            let nested_data_dir = if ne.path().join("new").exists() {
                                Some(ne.path().join("new"))
                            } else if ne.path().join("base").exists() {
                                Some(ne.path().join("base"))
                            } else {
                                None
                            };

                            if let Some(dir) = nested_data_dir {
                                if let Some(result) =
                                    parse_benchmark(&bench_name, &full_name, nested_size, &dir)
                                {
                                    results.push(result);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\nTotal benchmarks parsed: {}", results.len());

    // Group by base benchmark name (remove size-specific suffixes)
    let mut grouped: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
    for result in &results {
        // Extract base name by removing trailing /number patterns
        let base_name = extract_base_name(&result.name);
        grouped.entry(base_name).or_default().push(result);
    }

    println!("Total benchmark groups: {}", grouped.len());

    // Calculate Big O complexity for each benchmark group
    println!("\n=== BIG O COMPLEXITY ANALYSIS ===\n");

    let mut complexity_scores: Vec<(String, f64, String)> = Vec::new();

    for (name, group) in &grouped {
        if group.len() < 2 {
            continue;
        }

        let mut sorted = group.clone();
        sorted.sort_by_key(|r| r.size.unwrap_or(0));

        // Calculate growth rate
        if let (Some(first), Some(last)) = (sorted.first(), sorted.last()) {
            if let (Some(size1), Some(size2)) = (first.size, last.size) {
                if size1 > 0 && size2 > size1 {
                    let size_ratio = size2 as f64 / size1 as f64;
                    let time_ratio = last.mean_time_ns / first.mean_time_ns;

                    // Calculate complexity score (higher = worse)
                    // O(1) would have time_ratio ~= 1
                    // O(n) would have time_ratio ~= size_ratio
                    // O(n²) would have time_ratio ~= size_ratio²
                    let complexity_score = time_ratio / size_ratio;

                    let complexity_label = if complexity_score < 1.5 {
                        "O(1) - Constant"
                    } else if complexity_score < 2.5 {
                        "O(n) - Linear"
                    } else if complexity_score < 10.0 {
                        "O(n log n)"
                    } else {
                        "O(n²) or worse"
                    };

                    complexity_scores.push((
                        name.clone(),
                        complexity_score,
                        complexity_label.to_string(),
                    ));

                    println!("{}", name);
                    println!("  Size range: {} → {}", size1, size2);
                    println!(
                        "  Time range: {:.2}µs → {:.2}µs",
                        first.mean_time_ns / 1000.0,
                        last.mean_time_ns / 1000.0
                    );
                    println!("  Size ratio: {:.2}x", size_ratio);
                    println!("  Time ratio: {:.2}x", time_ratio);
                    println!("  Complexity score: {:.2}", complexity_score);
                    println!("  Estimated: {}", complexity_label);
                    println!();
                }
            }
        }
    }

    // Sort by complexity score (worst first)
    complexity_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\n=== WORST BIG O PERFORMERS (by complexity score) ===\n");
    for (i, (name, score, label)) in complexity_scores.iter().take(10).enumerate() {
        println!("{}. {} - Score: {:.2} ({})", i + 1, name, score, label);
    }

    // Analyze outliers
    println!("\n\n=== OUTLIER ANALYSIS ===\n");

    let mut outlier_scores: Vec<(String, f64, usize)> = Vec::new();

    for result in &results {
        let total_outliers = result.outliers.total();
        if total_outliers > 0 {
            let percentage = result.outliers.percentage(100); // Assuming 100 samples
            outlier_scores.push((
                format!("{}/{:?}", result.name, result.size),
                percentage,
                total_outliers,
            ));
        }
    }

    outlier_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top 20 benchmarks by outlier percentage:\n");
    for (i, (name, percentage, count)) in outlier_scores.iter().take(20).enumerate() {
        println!(
            "{}. {} - {:.1}% ({} outliers)",
            i + 1,
            name,
            percentage,
            count
        );
    }
}

fn extract_base_name(name: &str) -> String {
    // Remove trailing /number patterns to get base benchmark name
    // e.g., "parse_log/10" -> "parse_log"
    // e.g., "graph_parent_child_ops/child_idxs/100" -> "graph_parent_child_ops/child_idxs"
    let parts: Vec<&str> = name.split('/').collect();

    // Find the last part that's not a number
    let mut base_parts = Vec::new();
    for part in &parts {
        if part.parse::<usize>().is_ok() {
            break;
        }
        base_parts.push(*part);
    }

    if base_parts.is_empty() {
        name.to_string()
    } else {
        base_parts.join("/")
    }
}

fn parse_benchmark(
    bench_name: &str,
    sub_name: &str,
    size: Option<usize>,
    data_dir: &Path,
) -> Option<BenchmarkResult> {
    let estimates_path = data_dir.join("estimates.json");
    let sample_path = data_dir.join("sample.json");

    if !estimates_path.exists() || !sample_path.exists() {
        return None;
    }

    // Parse estimates.json for mean time
    let estimates_content = fs::read_to_string(&estimates_path).ok()?;
    let mean_time_ns = extract_mean_time(&estimates_content)?;

    // Parse sample.json and calculate outliers
    let sample_content = fs::read_to_string(&sample_path).ok()?;
    let outliers = calculate_outliers_from_samples(&sample_content);

    Some(BenchmarkResult {
        name: format!("{}/{}", bench_name, sub_name),
        size,
        mean_time_ns,
        outliers,
    })
}

fn extract_mean_time(json: &str) -> Option<f64> {
    // Parse JSON manually for mean.point_estimate
    if let Some(mean_section) = json.split("\"mean\":{").nth(1) {
        if let Some(point_estimate) = mean_section.split("\"point_estimate\":").nth(1) {
            if let Some(value_str) = point_estimate.split(',').next() {
                return value_str.trim().parse::<f64>().ok();
            }
        }
    }
    None
}

fn calculate_outliers_from_samples(json: &str) -> OutlierInfo {
    let mut info = OutlierInfo::default();

    // Extract times array from JSON
    if let Some(times_section) = json.split("\"times\":").nth(1) {
        if let Some(array_str) = times_section.split(']').next() {
            let array_content = array_str.trim_start_matches('[');
            let times: Vec<f64> = array_content
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();

            if times.len() < 10 {
                return info;
            }

            // Calculate outliers using Tukey's method
            let mut sorted_times = times.clone();
            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let n = sorted_times.len();
            let q1_idx = n / 4;
            let q3_idx = (3 * n) / 4;
            let q1 = sorted_times[q1_idx];
            let q3 = sorted_times[q3_idx];
            let iqr = q3 - q1;

            // Tukey's fences
            let lower_fence_mild = q1 - 1.5 * iqr;
            let lower_fence_severe = q1 - 3.0 * iqr;
            let upper_fence_mild = q3 + 1.5 * iqr;
            let upper_fence_severe = q3 + 3.0 * iqr;

            // Count outliers
            for &time in &times {
                if time < lower_fence_severe {
                    info.low_severe += 1;
                } else if time < lower_fence_mild {
                    info.low_mild += 1;
                } else if time > upper_fence_severe {
                    info.high_severe += 1;
                } else if time > upper_fence_mild {
                    info.high_mild += 1;
                }
            }
        }
    }

    info
}
