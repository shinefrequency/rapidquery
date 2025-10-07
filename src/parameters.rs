pub enum OptionalParam<'a> {
    Undefined,
    Defined(pyo3::Bound<'a, pyo3::PyAny>),
}

impl<'a> pyo3::FromPyObject<'a> for OptionalParam<'a> {
    fn extract_bound(ob: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Ok(OptionalParam::Defined(ob.clone()))
    }
}

impl<'a> OptionalParam<'a> {
    #[inline]
    #[optimize(speed)]
    #[allow(dead_code)]
    pub fn is_undefined(&self) -> bool {
        matches!(self, OptionalParam::Undefined)
    }

    #[inline]
    #[allow(dead_code)]
    pub unsafe fn unwrap_unchecked(self) -> pyo3::Bound<'a, pyo3::PyAny> {
        match self {
            OptionalParam::Defined(x) => x,

            #[cfg(debug_assertions)]
            OptionalParam::Undefined => unreachable!(),

            #[cfg(not(debug_assertions))]
            OptionalParam::Undefined => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
