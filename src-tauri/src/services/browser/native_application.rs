use cef::application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol};
use objc2::{
    define_class, extern_methods, msg_send,
    rc::Retained,
    runtime::{Bool, NSObjectProtocol},
    ClassType, DefinedClass, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{NSApp, NSApplication, NSEvent};
use std::cell::Cell;

#[derive(Default)]
struct BrowserApplicationIvars {
    handling_send_event: Cell<Bool>,
}

define_class! {
    #[unsafe(super(NSApplication))]
    #[thread_kind = MainThreadOnly]
    #[ivars = BrowserApplicationIvars]
    struct BrowserApplication;

    impl BrowserApplication {
        #[unsafe(method(sendEvent:))]
        unsafe fn send_event(&self, event: &NSEvent) {
            let was_handling = self.is_handling_send_event();
            if !was_handling {
                self.set_handling_send_event(true);
            }
            let _: () = unsafe { msg_send![super(self), sendEvent: event] };
            if !was_handling {
                self.set_handling_send_event(false);
            }
        }
    }

    unsafe impl CrAppControlProtocol for BrowserApplication {
        #[unsafe(method(setHandlingSendEvent:))]
        unsafe fn set_cef_handling_send_event(&self, value: Bool) {
            self.ivars().handling_send_event.set(value);
        }
    }

    unsafe impl CrAppProtocol for BrowserApplication {
        #[unsafe(method(isHandlingSendEvent))]
        unsafe fn cef_is_handling_send_event(&self) -> Bool {
            self.ivars().handling_send_event.get()
        }
    }

    unsafe impl CefAppProtocol for BrowserApplication {}
}

impl BrowserApplication {
    extern_methods! {
        #[unsafe(method(sharedApplication))]
        fn shared_application() -> Retained<Self>;

        #[unsafe(method(setHandlingSendEvent:))]
        fn set_handling_send_event(&self, value: bool);

        #[unsafe(method(isHandlingSendEvent))]
        fn is_handling_send_event(&self) -> bool;
    }
}

pub(super) fn prepare() -> Result<(), ()> {
    let marker = MainThreadMarker::new().ok_or(())?;
    let _application = BrowserApplication::shared_application();
    NSApp(marker)
        .isKindOfClass(BrowserApplication::class())
        .then_some(())
        .ok_or(())
}
