use crate::base::one_for_all::traits::relay::OfaBaseRelay;
use crate::base::relay::traits::ibc_message_sender::InjectMismatchIbcEventsCountError;
use crate::common::one_for_all::types::relay::OfaRelayWrapper;

impl<Relay: OfaBaseRelay> InjectMismatchIbcEventsCountError for OfaRelayWrapper<Relay> {
    fn mismatch_ibc_events_count_error(expected: usize, actual: usize) -> Self::Error {
        Relay::mismatch_ibc_events_count_error(expected, actual)
    }
}