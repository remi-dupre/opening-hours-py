use std::convert::TryInto;

use opening_hours::time_domain;

use chrono::offset::Local;
use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use pyo3::prelude::*;
use pyo3::types::{PyDateAccess, PyDateTime, PyTimeAccess};

fn rs_datetime(datetime: &PyDateTime) -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd(
            datetime.get_year(),
            datetime.get_month().into(),
            datetime.get_day().into(),
        ),
        NaiveTime::from_hms(
            datetime.get_hour().into(),
            datetime.get_minute().into(),
            datetime.get_second().into(),
        ),
    )
}

fn py_datetime<'p>(py: Python<'p>, datetime: NaiveDateTime) -> PyResult<&'p PyDateTime> {
    PyDateTime::new(
        py,
        datetime.date().year(),
        datetime.date().month().try_into()?,
        datetime.date().day().try_into()?,
        datetime.time().hour().try_into()?,
        datetime.time().minute().try_into()?,
        0,
        0,
        None,
    )
}

fn get_time(datetime: Option<&PyDateTime>) -> NaiveDateTime {
    datetime
        .map(rs_datetime)
        .unwrap_or_else(|| Local::now().naive_local())
}

#[pyclass]
struct TimeDomain {
    inner: time_domain::TimeDomain,
}

#[pymethods]
impl TimeDomain {
    #[new]
    fn new(oh: &str) -> PyResult<Self> {
        Ok(Self {
            inner: opening_hours::parser::parse(oh).unwrap(),
        })
    }

    fn state(&self, time: Option<&PyDateTime>) -> &str {
        match self.inner.state(get_time(time)) {
            time_domain::RuleKind::Open => "open",
            time_domain::RuleKind::Closed => "closed",
            time_domain::RuleKind::Unknown => "unkown",
        }
    }

    fn is_open(&self, time: Option<&PyDateTime>) -> bool {
        self.inner.is_open(get_time(time))
    }

    fn is_closed(&self, time: Option<&PyDateTime>) -> bool {
        self.inner.is_closed(get_time(time))
    }

    fn is_unknown(&self, time: Option<&PyDateTime>) -> bool {
        self.inner.is_unknown(get_time(time))
    }

    fn next_change<'p>(
        &self,
        py: Python<'p>,
        time: Option<&PyDateTime>,
    ) -> PyResult<&'p PyDateTime> {
        py_datetime(py, self.inner.next_change(get_time(time)))
    }
}

#[pymodule]
/// A Python module implemented in Rust.
fn opening_hours(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TimeDomain>()?;
    Ok(())
}
