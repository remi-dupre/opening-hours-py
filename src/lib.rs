mod errors;
mod types;

use std::sync::Arc;

use chrono::offset::Local;
use chrono::NaiveDateTime;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use opening_hours::{parser, time_domain};
use types::RangeIterator;

use crate::errors::ParserError;
use crate::types::{NaiveDateTimeWrapper, State};

fn get_time(datetime: Option<NaiveDateTime>) -> NaiveDateTime {
    datetime.unwrap_or_else(|| Local::now().naive_local())
}

/// Validate that input string is a correct opening hours description.
///
/// >>> opening_hours.validate("24/7")
/// True
///
/// >>> opening_hours.validate("24/77")
/// False
#[pyfunction]
#[text_signature = "(oh, /)"]
fn validate(oh: &str) -> bool {
    parser::parse(oh).is_ok()
}

#[pyclass]
#[text_signature = "(oh, /)"]
struct TimeDomain {
    inner: Arc<time_domain::TimeDomain>,
}

#[pymethods]
impl TimeDomain {
    /// Parse input opening hours description.
    ///
    /// If the input expression is not valid, raise a SyntaxError exception.
    #[new]
    fn new(oh: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Arc::new(parser::parse(oh).map_err(ParserError::from)?),
        })
    }

    /// Get current state of the time domain.
    ///
    /// Current time will be used if time is not specified.
    ///
    /// >>> opening_hours.TimeDomain("24/7").state()
    /// "open"
    ///
    /// >>> opening_hours.TimeDomain("24/7 off").state()
    /// "closed"
    ///
    /// >>> opening_hours.TimeDomain("24/7 unknown").state()
    /// "unknown"
    #[text_signature = "(self[, time])"]
    fn state(&self, time: Option<NaiveDateTimeWrapper>) -> State {
        self.inner.state(get_time(time.map(Into::into))).into()
    }

    #[text_signature = "(self[, time])"]
    fn is_open(&self, time: Option<NaiveDateTimeWrapper>) -> bool {
        self.inner.is_open(get_time(time.map(Into::into)))
    }

    #[text_signature = "(self[, time])"]
    fn is_closed(&self, time: Option<NaiveDateTimeWrapper>) -> bool {
        self.inner.is_closed(get_time(time.map(Into::into)))
    }

    #[text_signature = "(self[, time])"]
    fn is_unknown(&self, time: Option<NaiveDateTimeWrapper>) -> bool {
        self.inner.is_unknown(get_time(time.map(Into::into)))
    }

    #[text_signature = "(self[, time])"]
    fn next_change(&self, time: Option<NaiveDateTimeWrapper>) -> NaiveDateTimeWrapper {
        self.inner
            .next_change(get_time(time.map(Into::into)))
            .into()
    }

    fn intervals(
        &self,
        start: Option<NaiveDateTimeWrapper>,
        end: Option<NaiveDateTimeWrapper>,
    ) -> RangeIterator {
        RangeIterator::new(
            self.inner.clone(),
            get_time(start.map(Into::into)),
            end.map(Into::into),
        )
    }
}

#[pymodule]
/// TODO: documentation
fn opening_hours(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate, m)?).unwrap();
    m.add_class::<TimeDomain>()?;
    Ok(())
}
