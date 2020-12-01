use std::convert::TryInto;
use std::sync::Arc;

use chrono::prelude::*;
use chrono::NaiveDateTime;
use pyo3::prelude::*;
use pyo3::types::{PyDateAccess, PyDateTime, PyTimeAccess};
use pyo3::PyIterProtocol;

use opening_hours::time_domain;
use time_domain::{DateTimeRange, RuleKind};

// ---
// --- State
// ---

#[derive(Debug)]
pub enum State {
    Open,
    Closed,
    Unknown,
}

impl From<RuleKind> for State {
    fn from(kind: RuleKind) -> Self {
        match kind {
            RuleKind::Open => Self::Open,
            RuleKind::Closed => Self::Closed,
            RuleKind::Unknown => Self::Unknown,
        }
    }
}

impl<'p> IntoPy<Py<PyAny>> for State {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        match self {
            Self::Open => "open".into_py(py),
            Self::Closed => "closed".into_py(py),
            Self::Unknown => "unknown".into_py(py),
        }
    }
}

// ---
// --- NaiveDateTime wrapper
// ---

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NaiveDateTimeWrapper(NaiveDateTime);

impl NaiveDateTimeWrapper {
    pub fn max_py_value() -> NaiveDateTimeWrapper {
        NaiveDateTime::new(
            NaiveDate::from_ymd(9999, 12, 31),
            NaiveTime::from_hms(23, 59, 59),
        )
        .into()
    }
}

impl Into<NaiveDateTime> for NaiveDateTimeWrapper {
    fn into(self) -> NaiveDateTime {
        self.0
    }
}

impl From<NaiveDateTime> for NaiveDateTimeWrapper {
    fn from(dt: NaiveDateTime) -> Self {
        Self(dt)
    }
}

impl<'source> FromPyObject<'source> for NaiveDateTimeWrapper {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let py_datetime: &PyDateTime = ob.downcast()?;
        Ok({
            NaiveDateTime::new(
                NaiveDate::from_ymd(
                    py_datetime.get_year(),
                    py_datetime.get_month().into(),
                    py_datetime.get_day().into(),
                ),
                NaiveTime::from_hms(
                    py_datetime.get_hour().into(),
                    py_datetime.get_minute().into(),
                    py_datetime.get_second().into(),
                ),
            )
            .into()
        })
    }
}

impl<'p> IntoPy<PyResult<Option<Py<PyDateTime>>>> for NaiveDateTimeWrapper {
    fn into_py(self, py: Python<'_>) -> PyResult<Option<Py<PyDateTime>>> {
        Ok(if self >= Self::max_py_value() {
            None
        } else {
            Some(
                PyDateTime::new(
                    py,
                    self.0.date().year(),
                    self.0.date().month().try_into()?,
                    self.0.date().day().try_into()?,
                    self.0.time().hour().try_into()?,
                    self.0.time().minute().try_into()?,
                    0,
                    0,
                    None,
                )?
                .into(),
            )
        })
    }
}

impl<'p> IntoPy<Py<PyAny>> for NaiveDateTimeWrapper {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        let result: PyResult<_> = self.into_py(py);
        result
            .expect("failed at converting Rust date to Python")
            .into_py(py)
    }
}

// ---
// --- RangeIterator
// ---

#[pyclass(unsendable)]
pub struct RangeIterator {
    _td: Arc<time_domain::TimeDomain>,
    iter: Box<dyn Iterator<Item = DateTimeRange<'static>>>,
}

impl RangeIterator {
    pub fn new(
        td: Arc<time_domain::TimeDomain>,
        start: NaiveDateTime,
        end: Option<NaiveDateTime>,
    ) -> Self {
        let iter: Box<dyn Iterator<Item = DateTimeRange>> = {
            if let Some(end) = end {
                Box::new(td.iter_range(start, end)) as _
            } else {
                Box::new(td.iter_from(start))
            }
        };

        // This transmute will only change the lifetime specifier for resulting
        // iterator items.
        //
        // This is safe as long as we don't return any reference to these items
        // since self._td will live as long as self.
        //
        // TODO: there is probably a solution less agressive than transmute?
        let iter = unsafe { std::mem::transmute(iter) };

        Self { _td: td, iter }
    }
}

#[pyproto]
impl PyIterProtocol for RangeIterator {
    fn __iter__(slf: PyRef<Self>) -> Py<RangeIterator> {
        slf.into()
    }

    fn __next__(
        mut slf: PyRefMut<Self>,
    ) -> Option<(
        NaiveDateTimeWrapper,
        NaiveDateTimeWrapper,
        State,
        Vec<&'p str>,
    )> {
        let dt_range = slf.iter.next()?;
        Some((
            dt_range.range.start.into(),
            dt_range.range.end.into(),
            dt_range.kind.into(),
            dt_range.comments,
        ))
    }
}
