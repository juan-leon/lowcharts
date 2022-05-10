//! # Getting Started
//! Add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! lowcharts = "*"
//! ```
//!
//! ```rust,no_run
//! use lowcharts::plot;
//!
//! let vec = &[-1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99];
//! // Plot a histogram of the above vector, with 4 buckets and a precision
//! // choosen by library
//! let histogram = plot::Histogram::new(vec, 4, None);
//! print!("{}", histogram);
//! ```

mod format;
pub mod plot;
pub mod stats;
