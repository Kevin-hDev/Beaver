use super::emergency_slots_test_support::AdmittedKey;
use super::process_state::{
    MacProcessActions, MacProcessObservation, MacSignalObservation, MacSignalResult,
};
use super::MacProcessIdentity;
use crate::services::browser::cef_supervision::CefUnavailableCategory;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

pub(super) struct ScriptedProcess {
    observations: VecDeque<MacProcessObservation>,
    revalidations: VecDeque<MacSignalObservation>,
    signals: VecDeque<Result<MacSignalResult, CefUnavailableCategory>>,
}

pub(super) struct ScriptedMacProcessActions {
    processes: Mutex<HashMap<u32, ScriptedProcess>>,
    signal_counts: Mutex<HashMap<u32, usize>>,
}

impl ScriptedProcess {
    pub(super) fn new(
        observations: impl IntoIterator<Item = MacProcessObservation>,
        revalidations: impl IntoIterator<Item = MacSignalObservation>,
        signals: impl IntoIterator<Item = Result<MacSignalResult, CefUnavailableCategory>>,
    ) -> Self {
        Self {
            observations: observations.into_iter().collect(),
            revalidations: revalidations.into_iter().collect(),
            signals: signals.into_iter().collect(),
        }
    }

    pub(super) fn observing(observations: impl IntoIterator<Item = MacProcessObservation>) -> Self {
        Self::new(observations, [], [])
    }

    pub(super) fn ready_and_signalling(
        signal: Result<MacSignalResult, CefUnavailableCategory>,
    ) -> Self {
        Self::new(
            [MacProcessObservation::Alive],
            [MacSignalObservation::Ready],
            [signal],
        )
    }
}

impl ScriptedMacProcessActions {
    pub(super) fn single(
        key: AdmittedKey,
        observations: impl IntoIterator<Item = MacProcessObservation>,
        revalidations: impl IntoIterator<Item = MacSignalObservation>,
        signals: impl IntoIterator<Item = Result<MacSignalResult, CefUnavailableCategory>>,
    ) -> Self {
        Self::new([(
            key,
            ScriptedProcess::new(observations, revalidations, signals),
        )])
    }

    pub(super) fn new<const N: usize>(entries: [(AdmittedKey, ScriptedProcess); N]) -> Self {
        let processes = entries
            .into_iter()
            .map(|(key, process)| (key.pid, process))
            .collect();
        Self {
            processes: Mutex::new(processes),
            signal_counts: Mutex::new(HashMap::new()),
        }
    }

    pub(super) fn signal_count(&self, key: AdmittedKey) -> usize {
        self.signal_counts
            .lock()
            .expect("signal counts")
            .get(&key.pid)
            .copied()
            .unwrap_or(0)
    }

    fn with_process<T>(
        &self,
        identity: &MacProcessIdentity,
        action: impl FnOnce(&mut ScriptedProcess) -> T,
    ) -> T {
        let mut processes = self.processes.lock().expect("scripted processes");
        let process = processes
            .get_mut(&identity.pid)
            .expect("scripted process identity");
        action(process)
    }
}

impl MacProcessActions for ScriptedMacProcessActions {
    fn observe(&self, identity: &MacProcessIdentity) -> MacProcessObservation {
        self.with_process(identity, |process| {
            process
                .observations
                .pop_front()
                .expect("scripted observation exhausted")
        })
    }

    fn revalidate_before_signal(&self, identity: &MacProcessIdentity) -> MacSignalObservation {
        self.with_process(identity, |process| {
            process
                .revalidations
                .pop_front()
                .expect("scripted revalidation exhausted")
        })
    }

    fn signal_group(
        &self,
        identity: &MacProcessIdentity,
    ) -> Result<MacSignalResult, CefUnavailableCategory> {
        *self
            .signal_counts
            .lock()
            .expect("signal counts")
            .entry(identity.pid)
            .or_insert(0) += 1;
        self.with_process(identity, |process| {
            process
                .signals
                .pop_front()
                .expect("scripted signal exhausted")
        })
    }
}
