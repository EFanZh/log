use perf_event::events::Hardware;
use perf_event::{Builder, Group};
use std::time::{Duration, Instant};
use std::{hint, io};

const WARM_UP: u32 = 10;
const BENCHMARK_ITERATIONS: u32 = 10001; // Odd number for median.

struct Stats {
    cycles: u64,
    instructions: u64,
    wall_time: Duration,
}

fn median_by_key<T>(value: &mut [Stats], mut f: impl FnMut(&Stats) -> T) -> T
where
    T: Copy + Ord,
{
    let middle = value.len() / 2;
    let value = value.select_nth_unstable_by_key(middle, &mut f).1;

    f(value)
}

// See <https://github.com/rust-lang/rustc-perf/blob/master/collector/benchlib/src/measure/perf_counter/linux.rs>.
fn benchmark<R>(mut f: impl FnMut() -> R) -> io::Result<Stats> {
    let mut run = move || -> io::Result<Stats> {
        let (cycles, instructions) = {
            let mut group = Group::new()?;
            let mut make_counter = |kind| Builder::new().group(&mut group).kind(kind).build();
            let cycles = make_counter(Hardware::CPU_CYCLES)?;
            let instructions = make_counter(Hardware::INSTRUCTIONS)?;
            let enable_result = group.enable();

            // Benchmark start.

            let output = f();

            // Benchmark end.

            group.disable()?;

            enable_result?;

            hint::black_box(output);

            let counts = group.read()?;

            (counts[&cycles], counts[&instructions])
        };

        // Wall time.

        let wall_time = {
            let start = Instant::now();

            f();

            start.elapsed()
        };

        Ok(Stats {
            cycles,
            instructions,
            wall_time,
        })
    };

    for _ in 0..WARM_UP {
        run()?;
    }

    let mut stats = (0..BENCHMARK_ITERATIONS)
        .map(|_| run())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Stats {
        cycles: median_by_key(&mut stats, |states| states.cycles),
        instructions: median_by_key(&mut stats, |states| states.instructions),
        wall_time: median_by_key(&mut stats, |states| states.wall_time),
    })
}

fn main() -> io::Result<()> {
    fn log_original() {
        log::info!("abc");
    }

    fn log_optimized() {
        log_optimized::info!("abc");
    }

    let original = benchmark(log_original)?;
    let optimized = benchmark(log_optimized)?;

    for (name, counters) in [("original ", original), ("optimized", optimized)] {
        println!(
            "{name} => cycles: {}, instructions: {}, wall time: {:?}.",
            counters.cycles, counters.instructions, counters.wall_time,
        );
    }

    Ok(())
}
