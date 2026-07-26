use super::pump_gate::PumpGate;
use objc2::{
    define_class, msg_send, rc::Retained, runtime::NSObjectProtocol, sel, AnyThread, DefinedClass,
};
use objc2_app_kit::NSEventTrackingRunLoopMode;
use objc2_foundation::{
    MainThreadMarker, NSNumber, NSObject, NSObjectNSThreadPerformAdditions, NSRunLoop,
    NSRunLoopCommonModes, NSThread, NSTimer,
};
use std::cell::RefCell;
use std::sync::Arc;

pub(super) use super::native_pump_wake::PumpWake;

define_class! {
    #[unsafe(super(NSObject))]
    #[ivars = Arc<PumpGate>]
    pub(super) struct PumpTarget;

    impl PumpTarget {
        #[unsafe(method(scheduleWork:))]
        fn schedule_work(&self, delay: &NSNumber) {
            super::ffi_guard::unit_or(
                || self.finish_dispatch(),
                || {
                    if !self.ivars().begin_dispatch() {
                        return;
                    }
                    if let Ok(delay_ms) = i64::try_from(delay.integerValue()) {
                        if delay_ms <= 0 {
                            run_message_loop_work(self);
                        } else {
                            schedule_timer(
                                delay_ms.min(
                                    super::pump_scheduler::fallback_pump_interval_ms() as i64
                                ),
                                self,
                            );
                        }
                    }
                    self.finish_dispatch();
                },
            );
        }

        #[unsafe(method(timerTimeout:))]
        fn timer_timeout(&self, _timer: &NSTimer) {
            super::ffi_guard::unit_or(
                || self.finish_dispatch(),
                || {
                    let _ = self.ivars().request();
                    if !self.ivars().begin_dispatch() {
                        return;
                    }
                    run_message_loop_work(self);
                    self.finish_dispatch();
                },
            );
        }
    }

    unsafe impl NSObjectProtocol for PumpTarget {}
}

impl PumpTarget {
    fn new(gate: Arc<PumpGate>) -> Retained<Self> {
        let target = Self::alloc().set_ivars(gate);
        unsafe { msg_send![super(target), init] }
    }

    fn finish_dispatch(&self) {
        if self.ivars().complete_and_requeue() {
            queue_on_thread(self, &NSThread::currentThread(), 0);
        }
    }
}

struct NativePump {
    timer: Option<Retained<NSTimer>>,
}

thread_local! {
    static NATIVE_PUMP: RefCell<Option<NativePump>> = const { RefCell::new(None) };
}

pub(super) fn start(gate: Arc<PumpGate>) -> Result<PumpWake, ()> {
    let marker = MainThreadMarker::new().ok_or(())?;
    let owner_thread = NSThread::currentThread();
    let target = PumpTarget::new(gate.clone());
    let wake = PumpWake::new(gate, owner_thread, target, marker);
    NATIVE_PUMP.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_some() {
            return Err(());
        }
        *slot = Some(NativePump { timer: None });
        Ok(())
    })?;
    wake.start();
    Ok(wake)
}

fn run_message_loop_work(target: &PumpTarget) {
    cancel_timer();
    cef::do_message_loop_work();
    if !target.ivars().is_stopped() {
        schedule_timer(
            super::pump_scheduler::fallback_pump_interval_ms() as i64,
            target,
        );
    }
}

fn schedule_timer(delay_ms: i64, target: &PumpTarget) {
    cancel_timer();
    if target.ivars().is_stopped() {
        return;
    }
    let timer = unsafe {
        NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
            delay_ms.max(1) as f64 / 1_000.0,
            target,
            sel!(timerTimeout:),
            None,
            false,
        )
    };
    let run_loop = NSRunLoop::currentRunLoop();
    unsafe {
        run_loop.addTimer_forMode(&timer, NSRunLoopCommonModes);
        run_loop.addTimer_forMode(&timer, NSEventTrackingRunLoopMode);
    }
    if target.ivars().is_stopped() {
        timer.invalidate();
    } else if let Err(timer) = store_timer(timer) {
        timer.invalidate();
    }
}

fn store_timer(timer: Retained<NSTimer>) -> Result<(), Retained<NSTimer>> {
    NATIVE_PUMP.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(pump) = slot.as_mut() else {
            return Err(timer);
        };
        if pump.timer.is_some() {
            Err(timer)
        } else {
            pump.timer = Some(timer);
            Ok(())
        }
    })
}

fn take_timer() -> Option<Retained<NSTimer>> {
    NATIVE_PUMP.with(|slot| {
        slot.borrow_mut()
            .as_mut()
            .and_then(|pump| pump.timer.take())
    })
}

fn cancel_timer() {
    if let Some(timer) = take_timer() {
        timer.invalidate();
    }
}

pub(super) fn queue_on_thread(target: &PumpTarget, thread: &NSThread, delay_ms: i64) {
    let number = NSNumber::numberWithInteger(delay_ms.clamp(0, isize::MAX as i64) as isize);
    unsafe {
        target.performSelector_onThread_withObject_waitUntilDone(
            sel!(scheduleWork:),
            thread,
            Some(&number),
            false,
        );
    }
}

pub(super) fn stop() {
    let mut pump = NATIVE_PUMP.with(|slot| slot.borrow_mut().take());
    if let Some(timer) = pump.as_mut().and_then(|pump| pump.timer.take()) {
        timer.invalidate();
    }
}
