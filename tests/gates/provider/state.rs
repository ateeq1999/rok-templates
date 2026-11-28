
impl AppState {
    /// This state, charging through `payments` instead of the configured
    /// transport: a test's `FakePayments`.
    #[must_use]
    pub fn with_payments(mut self, payments: Arc<dyn Payments>) -> Self {
        self.payments = payments;
        self
    }
}
