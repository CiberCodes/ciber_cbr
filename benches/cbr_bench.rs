//! Dependency-free CBR parser and formatter benchmark harness.
// The benchmark-local allocator must use unsafe `GlobalAlloc` forwarding.
#![allow(unsafe_code)]

use std::{
  alloc::{GlobalAlloc, Layout, System},
  hint::black_box,
  sync::atomic::{AtomicUsize, Ordering},
  time::Instant,
};

use ciber_cbr::{CbrFile, ParseLimits, Value, formatter};

struct TrackingAllocator;
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);

// SAFETY: all allocation operations are forwarded unchanged to `System`;
// atomics only track the requested live allocation sizes.
unsafe impl GlobalAlloc for TrackingAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    // SAFETY: forwarding the caller-provided layout to the system allocator.
    let pointer = unsafe { System.alloc(layout) };
    if !pointer.is_null() {
      record_alloc(layout.size());
    }
    pointer
  }

  unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
    LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    // SAFETY: pointer and layout are passed through from the matching allocation.
    unsafe {
      System.dealloc(pointer, layout);
    }
  }

  unsafe fn realloc(
    &self,
    pointer: *mut u8,
    layout: Layout,
    new_size: usize,
  ) -> *mut u8 {
    // SAFETY: forwarding the original allocation metadata to the system allocator.
    let resized = unsafe { System.realloc(pointer, layout, new_size) };
    if !resized.is_null() {
      if new_size >= layout.size() {
        record_alloc(new_size - layout.size());
      } else {
        LIVE_BYTES.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
      }
    }
    resized
  }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn record_alloc(size: usize) {
  let live = LIVE_BYTES.fetch_add(size, Ordering::Relaxed).saturating_add(size);
  PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
}

fn measure(label: &str, iterations: usize, mut operation: impl FnMut()) {
  let start = Instant::now();
  for _ in 0..iterations {
    operation();
  }
  let elapsed = start.elapsed();
  println!(
    "{label}: {iterations} iterations in {elapsed:?} ({:?}/iteration)",
    elapsed / u32::try_from(iterations).unwrap_or(u32::MAX)
  );
}

fn measure_peak(label: &str, operation: impl FnOnce()) {
  let baseline = LIVE_BYTES.load(Ordering::Relaxed);
  PEAK_BYTES.store(baseline, Ordering::Relaxed);
  operation();
  let peak = PEAK_BYTES.load(Ordering::Relaxed).saturating_sub(baseline);
  println!("{label}: peak additional live heap {peak} bytes");
}

fn main() {
  let ascii =
    (0..500).map(|i| format!("key{i} = {i}")).collect::<Vec<_>>().join("\n");
  let unicode = format!("name = \"{}\"", "東京🦀é".repeat(1_000));
  let long_string = format!("text = \"{}\"", "a".repeat(64 * 1024));
  let nested = format!("data = {}1{}", "[".repeat(64), "]".repeat(64));
  let cases = [
    ("ascii-500-properties", ascii),
    ("unicode-string", unicode),
    ("long-string-64KiB", long_string),
    ("nested-depth-64", nested),
  ];

  for (name, source) in cases {
    measure(
      &format!("parse/{name}"),
      200,
      || match CbrFile::parse_with_limits(
        black_box(&source),
        ParseLimits::default(),
      ) {
        Ok(value) => {
          black_box(value);
        }
        Err(error) => panic!("invalid benchmark fixture: {error}"),
      },
    );
  }

  let value = Value::Array((0..10_000).map(Value::I64).collect());
  let parse_input = (0..10_000)
    .map(|i| format!("item{i} = {i}"))
    .collect::<Vec<_>>()
    .join("\n");
  measure("format/array-10k-compact", 100, || {
    match formatter::to_string(black_box(&value)) {
      Ok(output) => {
        black_box(output);
      }
      Err(error) => panic!("formatting failed: {error}"),
    }
  });
  measure(
    "format/array-10k-pretty",
    100,
    || match formatter::to_string_pretty(black_box(&value)) {
      Ok(output) => {
        black_box(output);
      }
      Err(error) => panic!("formatting failed: {error}"),
    },
  );
  measure_peak("memory/format-10k-to-string", || {
    match formatter::to_string(&value) {
      Ok(output) => {
        black_box(output);
      }
      Err(error) => panic!("formatting failed: {error}"),
    }
  });
  measure_peak("memory/format-10k-incremental-sink", || {
    if let Err(error) = formatter::to_writer(&value, std::io::sink()) {
      panic!("formatting failed: {error}");
    }
  });
  measure_peak("memory/parse-10k-properties", || {
    match CbrFile::parse_with_limits(&parse_input, ParseLimits::default()) {
      Ok(parsed) => {
        black_box(parsed);
      }
      Err(error) => panic!("parsing failed: {error}"),
    }
  });
}
