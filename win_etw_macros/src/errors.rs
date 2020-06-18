use proc_macro2::Span;
use syn::Error;

#[derive(Default)]
pub(crate) struct CombinedErrors {
    error: Option<Error>,
}

impl CombinedErrors {
    pub fn push(&mut self, error: Error) {
        match self.error.as_mut() {
            Some(existing) => existing.combine(error),
            None => self.error = Some(error),
        }
    }
    pub fn into_result<T>(self, value: T) -> Result<T, Error> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(value),
        }
    }
    pub fn scope<'a, F: FnOnce(&mut ErrorScope<'a>) -> Result<(), Error>>(
        &'a mut self,
        span: Span,
        f: F,
    ) {
        let mut scope = ErrorScope { errors: self, span };
        match f(&mut scope) {
            Ok(()) => {}
            Err(e) => {
                scope.errors.push(e);
            }
        }
    }
}

pub(crate) struct ErrorScope<'a> {
    span: Span,
    errors: &'a mut CombinedErrors,
}

impl<'a> ErrorScope<'a> {
    pub fn msg(&mut self, s: &str) {
        self.errors.push(Error::new(self.span, s));
    }
}
