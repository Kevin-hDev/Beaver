use super::emergency::{EmergencyInventory, VerifiedProcessIdentity};
use super::emergency_drain::{EmergencyObservation, EmergencySignaler};
use super::policy::{ShutdownPolicy, ShutdownTimeline};
use super::state::{ShutdownPhase, ShutdownState};
use super::ultimate::{RawExitActions, UltimateExit};
use super::watchdog::{WatchdogActions, WatchdogThread};
use super::ExitIntent;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Le calendrier d'arrêt, raccourci pour les tests.
///
/// Il doit rester plus long que la mise en route d'un fil d'exécution. Passé
/// l'échéance ultime, run_watchdog sort de sa boucle sans avoir drainé une
/// seule fois : le signaleur n'est jamais appelé et le test attend un événement
/// qui ne viendra pas. Les valeurs précédentes plaçaient cette échéance à 40 ms
/// de l'origine, ce que la mise en place tenait sur une machine libre et pas
/// sur un runner partagé. La production travaille en secondes, où le temps
/// d'ordonnancement ne pèse rien.
fn timeline(origin: Instant) -> ShutdownTimeline {
    ShutdownTimeline::from_origin(
        origin,
        ShutdownPolicy::new(
            Duration::from_millis(50),
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(400),
        )
        .expect("watchdog policy"),
    )
}

/// Attend `condition`, et nomme l'attente qui expire.
///
/// Les quatre appels de ce fichier échouaient sur le même message : un échec
/// en intégration continue ne disait pas laquelle des attentes avait expiré, et
/// la question ne se tranchait qu'en reproduisant.
/// La borne garde le test de rester pendu, elle ne mesure rien : elle est donc
/// très au-dessus du calendrier observé, et non ajustée sur lui.
fn wait_until(awaited: &str, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(Instant::now() < deadline, "attente expirée : {awaited}");
        std::thread::yield_now();
    }
}

struct CountingSignaler {
    calls: AtomicUsize,
}

impl EmergencySignaler for CountingSignaler {
    fn signal_or_recheck(
        &self,
        _identity: VerifiedProcessIdentity,
        _already_requested: bool,
    ) -> EmergencyObservation {
        self.calls.fetch_add(1, Ordering::AcqRel);
        EmergencyObservation::Terminating
    }
}

#[test]
fn watchdog_requests_tauri_exit_then_starts_emergency_drain() {
    let state = Arc::new(ShutdownState::new());
    assert_eq!(state.begin_closing(), super::state::BeginClosing::Started);
    let inventory = EmergencyInventory::new();
    let _registration = inventory
        .try_publish(VerifiedProcessIdentity::new(5, 15, 25).expect("identity"))
        .expect("registration");
    let exits = Arc::new(AtomicUsize::new(0));
    let exit_calls = Arc::clone(&exits);
    let signaler = Arc::new(CountingSignaler {
        calls: AtomicUsize::new(0),
    });
    let actions = WatchdogActions::testing(
        move |intent, _| {
            assert_eq!(intent, ExitIntent::Restart);
            exit_calls.fetch_add(1, Ordering::AcqRel);
        },
        signaler.clone(),
    );
    // L'origine se prend au plus près du lancement : tout ce qui la précède est
    // du temps retranché au calendrier que le chien de garde doit parcourir.
    let timeline = timeline(Instant::now());
    let watchdog = WatchdogThread::spawn(
        timeline,
        Arc::clone(&state),
        inventory,
        ExitIntent::Restart,
        9,
        actions,
    )
    .expect("watchdog");

    wait_until("sortie déclenchée une fois", || {
        exits.load(Ordering::Acquire) == 1
    });
    assert_eq!(state.phase(), ShutdownPhase::ReadyToExit);
    wait_until("processus d'urgence signalé", || {
        signaler.calls.load(Ordering::Acquire) > 0
    });
    watchdog.join_for_test();
}

#[test]
fn watchdog_spawn_failure_does_not_touch_existing_state() {
    let origin = Instant::now();
    let state = Arc::new(ShutdownState::new());
    assert_eq!(state.begin_closing(), super::state::BeginClosing::Started);
    let result = WatchdogThread::spawn_with(
        timeline(origin),
        Arc::clone(&state),
        EmergencyInventory::new(),
        ExitIntent::Exit,
        0,
        WatchdogActions::testing(
            |_, _| {},
            Arc::new(CountingSignaler {
                calls: AtomicUsize::new(0),
            }),
        ),
        |_| Err(std::io::Error::other("injected watchdog spawn failure")),
    );

    assert!(result.is_err());
    assert_eq!(state.phase(), ShutdownPhase::Closing);
}

struct BlockingSignaler {
    entered: AtomicBool,
    released: Mutex<bool>,
    wake: Condvar,
}

impl BlockingSignaler {
    fn release(&self) {
        let mut released = self.released.lock().expect("release lock");
        *released = true;
        self.wake.notify_all();
    }
}

impl EmergencySignaler for BlockingSignaler {
    fn signal_or_recheck(
        &self,
        _identity: VerifiedProcessIdentity,
        _already_requested: bool,
    ) -> EmergencyObservation {
        self.entered.store(true, Ordering::Release);
        let mut released = self.released.lock().expect("blocking lock");
        while !*released {
            released = self.wake.wait(released).expect("blocking wait");
        }
        EmergencyObservation::Exited
    }
}

#[test]
fn blocked_watchdog_cannot_delay_the_ultimate_exit() {
    let ultimate_calls = Arc::new(AtomicUsize::new(0));
    let raw_calls = Arc::clone(&ultimate_calls);
    let state = Arc::new(ShutdownState::new());
    assert_eq!(state.begin_closing(), super::state::BeginClosing::Started);
    let inventory = EmergencyInventory::new();
    let _registration = inventory
        .try_publish(VerifiedProcessIdentity::new(8, 18, 28).expect("identity"))
        .expect("registration");
    let signaler = Arc::new(BlockingSignaler {
        entered: AtomicBool::new(false),
        released: Mutex::new(false),
        wake: Condvar::new(),
    });
    let actions = WatchdogActions::testing(|_, _| {}, signaler.clone());

    // La sortie ultime et le chien de garde partagent la même origine : c'est
    // ce qui fait de l'échéance ultime un budget commun. Elle se prend donc
    // ici, une fois tout le reste en place.
    let origin = Instant::now();
    let timeline = timeline(origin);
    let mut ultimate = UltimateExit::initialize_for_test(
        origin,
        RawExitActions::testing(
            move |_| {
                raw_calls.fetch_add(1, Ordering::AcqRel);
            },
            |_| {},
        ),
    )
    .expect("ultimate");
    assert!(ultimate.arm(timeline.ultimate_deadline(), 1));
    let watchdog = WatchdogThread::spawn(timeline, state, inventory, ExitIntent::Exit, 0, actions)
        .expect("watchdog");

    wait_until("chien de garde entré dans le signaleur", || {
        signaler.entered.load(Ordering::Acquire)
    });
    wait_until("sortie ultime déclenchée malgré le blocage", || {
        ultimate_calls.load(Ordering::Acquire) == 1
    });
    signaler.release();
    watchdog.join_for_test();
    ultimate.stop_for_test();
}
