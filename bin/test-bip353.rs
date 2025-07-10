/// This is real scenario testing for the library

use bip353::{Bip353Resolver, ResolverConfig};
use clap::{Parser, Subcommand};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "test-bip353")]
#[command(about = "Real-world testing tool for BIP-353 integration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Network (mainnet, testnet, signet, regtest)
    #[arg(short, long, default_value = "mainnet")]
    network: String,
    
    /// Timeout in seconds
    #[arg(short, long, default_value = "10")]
    timeout: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Test resolving a single BIP-353 address
    Resolve {
        /// The BIP-353 address (e.g., matt@mattcorallo.com)
        address: String,
    },
    /// Test known working BIP-353 addresses
    TestKnown,
    /// Test with enhanced features (cache, metrics)
    TestEnhanced {
        /// Address to test
        address: String,
        /// Number of repeated calls to test caching
        #[arg(short, long, default_value = "3")]
        repeat: usize,
    },
    /// Benchmark resolution performance
    Benchmark {
        /// Address to benchmark
        address: String,
        /// Number of iterations
        #[arg(short, long, default_value = "10")]
        iterations: usize,
    },
    /// Test FFI compatibility (if compiled with ffi feature)
    #[cfg(feature = "ffi")]
    TestFfi {
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }
    
    match cli.command {
        Commands::Resolve { ref address } => test_resolve(address.clone(), &cli).await,
        Commands::TestKnown => test_known_addresses(&cli).await,
        Commands::TestEnhanced { ref address, repeat } => test_enhanced(address.clone(), repeat, &cli).await,
        Commands::Benchmark { ref address, iterations } => benchmark_resolution(address.clone(), iterations, &cli).await,
        #[cfg(feature = "ffi")]
        Commands::TestFfi { ref address } => test_ffi_integration(address.clone(), &cli).await,
    }
}

async fn test_resolve(address: String, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing BIP-353 resolution for: {}", address);
    
    let config = create_config(cli)?;
    let resolver = Bip353Resolver::with_config(config)?;
    
    let start = Instant::now();
    
    match resolver.resolve_address(&address).await {
        Ok(info) => {
            let duration = start.elapsed();
            
            println!("✅ Resolution successful! ({}ms)", duration.as_millis());
            println!("   🔗 URI: {}", info.uri);
            println!("   💳 Type: {:?}", info.payment_type);
            println!("   🔄 Reusable: {}", info.is_reusable);
            
            if cli.verbose && !info.parameters.is_empty() {
                println!("   📋 Parameters:");
                for (key, value) in &info.parameters {
                    println!("      {}: {}", key, value);
                }
            }
            
            // Test parsing the result
            if let Ok((parsed_user, parsed_domain)) = bip353::parse_address(&address) {
                println!("   📧 Parsed: {}@{}", parsed_user, parsed_domain);
            }
            
            // Test BIP-21 URI parsing if it's a bitcoin: URI
            if info.uri.starts_with("bitcoin:") {
                println!("   ✅ Valid BIP-21 URI format");
                
                // Extract address for validation
                let uri_without_scheme = &info.uri[8..];
                if let Some(addr_end) = uri_without_scheme.find('?') {
                    let addr = &uri_without_scheme[..addr_end];
                    if !addr.is_empty() {
                        println!("   📍 Bitcoin address: {}", addr);
                    }
                } else if !uri_without_scheme.is_empty() && !uri_without_scheme.starts_with('?') {
                    println!("   📍 Bitcoin address: {}", uri_without_scheme);
                }
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            eprintln!("❌ Resolution failed after {}ms: {}", duration.as_millis(), e);
            
            // Provide debugging info
            println!("\n🔍 Debugging information:");
            if let Ok((user, domain)) = bip353::parse_address(&address) {
                println!("   User: {}", user);
                println!("   Domain: {}", domain);
                println!("   Expected DNS record: {}._bitcoin-payment.{}", user, domain);
            } else {
                println!("   Invalid address format");
            }
            
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn test_known_addresses(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing known BIP-353 addresses...\n");
    
    // Known test addresses (these might work, might not - BIP-353 is still new)
    let test_addresses = vec![
        "matt@mattcorallo.com",        // Matt Corallo's test address
        "test@bitcoin.org",            // Hypothetical test
        "demo@btcpayserver.org",       // try see if BTCPay server might support this
        "₿test@example.com",           // having a bitcoin symbol
    ];
    
    let config = create_config(cli)?;
    let resolver = Bip353Resolver::with_config(config)?;
    
    let mut successful = 0;
    let mut total = 0;
    
    for address in test_addresses {
        total += 1;
        println!("Testing: {}", address);
        
        let start = Instant::now();
        match resolver.resolve_address(address).await {
            Ok(info) => {
                successful += 1;
                let duration = start.elapsed();
                println!("  ✅ Success ({}ms): {}", duration.as_millis(), info.uri);
            }
            Err(e) => {
                let duration = start.elapsed();
                println!("  ❌ Failed ({}ms): {}", duration.as_millis(), e);
            }
        }
        println!();
    }
    
    println!("📊 Results: {}/{} addresses resolved successfully", successful, total);
    
    if successful == 0 {
        println!("💡 This is normal - BIP-353 is very new and most domains don't support it yet!");
        println!("   Try setting up your own test domain or finding known working examples.");
    }
    
    Ok(())
}

async fn test_enhanced(address: String, repeat: usize, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing enhanced features for: {}", address);
    println!("   Repeat count: {} (to test caching)", repeat);
    
    let config = create_config(cli)?;
    let resolver = Bip353Resolver::with_enhanced_config(
        config,
        true, // enable cache
        Duration::from_secs(300), // 5 minute TTL
        true, // enable metrics
    )?;
    
    let mut times = Vec::new();
    
    for i in 1..=repeat {
        println!("\n📞 Call {} of {}:", i, repeat);
        
        let start = Instant::now();
        
        // FIX: Parse the address and call the correct method
        let (user, domain) = bip353::parse_address(&address)?;
        
        match resolver.resolve_with_safety_checks(&user, &domain).await {
            Ok(safe_info) => {
                let duration = start.elapsed();
                times.push(duration);
                
                println!("  ✅ Success ({}ms)", duration.as_millis());
                println!("     URI: {}", safe_info.payment_info.uri);
                println!("     Warnings: {}", safe_info.warnings.len());
                println!("     Last checked: {:?}", safe_info.last_checked);
            }
            Err(e) => {
                let duration = start.elapsed();
                times.push(duration);
                println!("  ❌ Failed ({}ms): {}", duration.as_millis(), e);
            }
        }
    }
    
    // Show performance analysis
    if !times.is_empty() {
        let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
        let min_time = times.iter().min().unwrap();
        let max_time = times.iter().max().unwrap();
        
        println!("\n📈 Performance Analysis:");
        println!("   Average: {}ms", avg_time.as_millis());
        println!("   Min: {}ms", min_time.as_millis());
        println!("   Max: {}ms", max_time.as_millis());
        
        // Cache effectiveness
        if times.len() > 1 {
            let first_call = times[0].as_millis();
            let second_call = times[1].as_millis();
            if second_call < first_call / 2 {
                println!("   🎯 Cache appears to be working! (2nd call much faster)");
            }
        }
    }
    
    // Show metrics if available
    if let Some(metrics) = resolver.get_metrics() {
        println!("\n📊 Metrics:");
        println!("   Total resolutions: {}", metrics.total);
        println!("   Successful: {}", metrics.success);
        println!("   Failed: {}", metrics.failed);
        println!("   Success rate: {:.1}%", metrics.success_rate * 100.0);
    }
    
    // Test cache management
    println!("\n🧹 Testing cache management:");
    resolver.invalidate_cache(&address).await;
    println!("   Cache invalidated for {}", address);
    
    resolver.clear_cache().await;
    println!("   Cache cleared");
    
    Ok(())
}

async fn benchmark_resolution(address: String, iterations: usize, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Benchmarking resolution performance");
    println!("   Address: {}", address);
    println!("   Iterations: {}", iterations);
    
    let config = create_config(cli)?;
    let resolver = Bip353Resolver::with_config(config)?;
    
    let mut times = Vec::with_capacity(iterations);
    let mut successful = 0;
    
    println!("\n🏃 Running benchmark...");
    let total_start = Instant::now();
    
    for i in 0..iterations {
        if i % 10 == 0 && i > 0 {
            println!("   Completed {}/{} iterations", i, iterations);
        }
        
        let start = Instant::now();
        match resolver.resolve_address(&address).await {
            Ok(_) => {
                successful += 1;
                times.push(start.elapsed());
            }
            Err(_) => {
                times.push(start.elapsed());
            }
        }
    }
    
    let total_time = total_start.elapsed();
    
    println!("\n📊 Benchmark Results:");
    println!("   Total time: {}ms", total_time.as_millis());
    println!("   Successful resolutions: {}/{}", successful, iterations);
    
    if !times.is_empty() {
        let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
        
        // Clone times for sorting to avoid borrow checker issues
        let mut sorted_times = times.clone();
        sorted_times.sort();
        
        let min_time = times.iter().min().unwrap();
        let max_time = times.iter().max().unwrap();
        let p50 = sorted_times[sorted_times.len() / 2];
        let p95 = sorted_times[(sorted_times.len() * 95) / 100];
        let p99 = sorted_times[(sorted_times.len() * 99) / 100];
        
        println!("   Average: {}ms", avg_time.as_millis());
        println!("   Min: {}ms", min_time.as_millis());
        println!("   Max: {}ms", max_time.as_millis());
        println!("   P50 (median): {}ms", p50.as_millis());
        println!("   P95: {}ms", p95.as_millis());
        println!("   P99: {}ms", p99.as_millis());
        println!("   Throughput: {:.1} resolutions/second", iterations as f64 / total_time.as_secs_f64());
    }
    
    Ok(())
}

#[cfg(feature = "ffi")]
async fn test_ffi_integration(address: String, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing FFI integration for: {}", address);
    
    // This would test the FFI bindings
    // For now, just show that FFI is available
    println!("   FFI feature is enabled");
    println!("   You can test C integration using the header file");
    
    Ok(())
}

fn create_config(cli: &Cli) -> Result<ResolverConfig, Box<dyn std::error::Error>> {
    let base_config = match cli.network.as_str() {
        "mainnet" | "main" => ResolverConfig::default(),
        "testnet" | "test" => ResolverConfig::testnet(),
        "signet" => ResolverConfig::signet(),
        "regtest" => ResolverConfig::regtest(),
        _ => return Err(format!("Unknown network: {}", cli.network).into()),
    };
    
    Ok(base_config.with_timeout(Duration::from_secs(cli.timeout)))
}