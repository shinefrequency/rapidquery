pub enum OptionalParam<'a, 'py> {
    Undefined,
    Defined(pyo3::Borrowed<'a, 'py, pyo3::PyAny>),
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for OptionalParam<'a, 'py> {
    type Error = pyo3::PyErr;

    fn extract(obj: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> Result<Self, Self::Error> {
        Ok(Self::Defined(obj))
    }
}

impl<'a, 'py> OptionalParam<'a, 'py> {
    #[inline]
    #[optimize(speed)]
    #[allow(dead_code)]
    pub fn is_undefined(&self) -> bool {
        matches!(self, OptionalParam::Undefined)
    }

    #[inline]
    #[allow(dead_code)]
    pub unsafe fn unwrap_unchecked(self) -> pyo3::Borrowed<'a, 'py, pyo3::PyAny> {
        match self {
            OptionalParam::Defined(x) => x,

            #[cfg(debug_assertions)]
            OptionalParam::Undefined => unreachable!(),

            #[cfg(not(debug_assertions))]
            OptionalParam::Undefined => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
