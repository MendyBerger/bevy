// fake temporary winit, until winit adds wasi support

// wit_bindgen::generate!({
//     path: "../../wit",
//     world: "wgpu:backend/main",
//     generate_all,
// });

pub mod window {
    use std::sync::Arc;

    pub use super::event::WindowId;
    use wgpu::backend::wasi_webgpu::wasi::webgpu::{
        graphics_context::Context,
        surface::{CreateDesc, Surface},
    };
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

    use super::{dpi::PhysicalSize, event_loop::{EventLoop, EventLoopWindowTarget}};

    #[derive(Debug)]
    pub struct Window {
        pub surface: Arc<Surface>,
        pub graphics_context: Context,
        id: WindowId,
    }
    
    impl Window {
        pub fn id(&self) -> WindowId {
            self.id
        }
    }

    impl HasWindowHandle for Window {
        fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
            let handle = self.graphics_context.handle();
            let window_handle = unsafe {
                raw_window_handle::WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::Wasi(
                    raw_window_handle::WasiWindowHandle::new(handle),
                ))
            };
            Ok(window_handle)
        }
    }
    impl HasDisplayHandle for Window {
        fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
            let handle = self.graphics_context.handle();
            let display_handle: raw_window_handle::DisplayHandle = unsafe {
                raw_window_handle::DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Wasi(
                    raw_window_handle::WasiDisplayHandle::new(handle),
                ))
            };
            Ok(display_handle)
        }
    }
    impl Window {
        pub fn inner_size(&self) -> PhysicalSize {
            PhysicalSize {
                width: self.surface.width(),
                height: self.surface.height(),
            }
        }

        pub fn request_redraw(&self) {}
    }

    #[derive(Default)]
    pub struct WindowBuilder {}
    impl WindowBuilder {
        pub fn new() -> Self {
            Default::default()
        }
        pub fn build<T: 'static>(
            self,
            // window_target: &EventLoop<T>,
            window_target: &EventLoopWindowTarget<T>,
        ) -> Result<Window, ()> {
            log::info!("building window");
            let graphics_context = Context::new();
            let surface = Arc::new(Surface::new(CreateDesc {
                height: None,
                width: None,
            }));
            log::info!("new window handle: {:?}", graphics_context.handle());

            surface.connect_graphics_context(&graphics_context);
            // let surface = Arc::new(surface);
            // let window_target = window_target
            *window_target.wasi_surface.lock().unwrap() = Some(Arc::clone(&surface));
            Ok(Window {
                surface,
                graphics_context,
                id: WindowId(0),
            })
        }
        pub fn with_title<T: Into<String>>(mut self, _title: T) -> Self {
            self
        }
    }
}

pub mod event_loop {
    use crate::winit::event::{Event, KeyEvent};
    use std::{io::SeekFrom, marker::PhantomData, ops::Deref, sync::{Arc, Mutex}};
    use wgpu::backend::wasi_webgpu::wasi::webgpu::surface::Surface;

    use super::event::WindowId;

    pub struct EventLoop<T: 'static> {
        pub(crate) event_loop: PhantomData<T>,
        pub(crate) _marker: PhantomData<*mut ()>,

        pub(crate) window_target: EventLoopWindowTarget<T>,
    }
    impl<T: 'static> EventLoop<T> {
        pub fn new() -> Result<EventLoop<()>, ()> {
            Ok(EventLoop {
                event_loop: PhantomData,
                _marker: PhantomData,
                window_target: EventLoopWindowTarget {
                    p: PhantomData,
                    _marker: PhantomData,
                    wasi_surface: Default::default(),
                },
            })
        }
        pub fn run<F>(self, mut event_handler: F) -> Result<(), String>
        where
            F: FnMut(Event, &EventLoopWindowTarget<T>),
        {
            log::info!("running event loop");
            event_handler(Event::NewEvents, &self.window_target);
            wasi::clocks::monotonic_clock::subscribe_duration(500_000_000).block();
            event_handler(Event::Resumed, &self.window_target);
            // panic!("not implemented");
            // loop {
            //     let event = Event::WindowEvent {
            //         window_id: WindowId(0),
            //         event: super::event::WindowEvent::RedrawRequested,
            //     };
            //     event_handler(event, &self.window_target);
            // }



            // let surface = {
            //     let surface = self.window_target.wasi_surface.lock().unwrap();
            //     let surface = surface.as_ref().unwrap();
            //     Arc::clone(surface)
            // };
            // // let pointer_up_pollable = surface.subscribe_pointer_up();
            // // let pointer_down_pollable = surface.subscribe_pointer_down();
            // // let pointer_move_pollable = surface.subscribe_pointer_move();
            // let frame_pollable = surface.subscribe_frame();
            // let key_up_pollable = surface.subscribe_key_up();
            // // let key_down_pollable = surface.subscribe_key_down();
            // let resize_pollable = surface.subscribe_resize();
            // let pollables = vec![
            //     // &pointer_up_pollable,
            //     // &pointer_down_pollable,
            //     // &pointer_move_pollable,
            //     &key_up_pollable,
            //     // &key_down_pollable,
            //     &resize_pollable,
            //     &frame_pollable,
            // ];
            // let pollables_res = wgpu::backend::wasi_webgpu::wasi::io::poll::poll(&pollables[..]);
                let surface = {
                    let surface = self.window_target.wasi_surface.lock().unwrap();
                    surface.as_ref().cloned().unwrap()
                };
                        let key_down_pollable = surface.subscribe_key_down();
                        let key_up_pollable = surface.subscribe_key_up();
                        let resize_pollable = surface.subscribe_resize();
                        let frame_pollable = surface.subscribe_frame();
                        let pollables = vec![
                            &key_down_pollable,
                            &key_up_pollable,
                            &resize_pollable,
                            &frame_pollable,
                        ];
            loop {
                let mut event = None;
                // match surface {
                //     Some(surface) => {
                        let pollables_res = wgpu::backend::wasi_webgpu::wasi::io::poll::poll(&pollables[..]);
                        // log::info!("pollables_res: {:?}", pollables_res);
                        if pollables_res.contains(&0) {
                            let key = match surface.get_key_down() {
                                Some(key) => match key.key {
                                    Some(wgpu::backend::wasi_webgpu::wasi::webgpu::surface::Key::ArrowLeft) => Some(crate::winit::keyboard::KeyCode::ArrowLeft),
                                    Some(wgpu::backend::wasi_webgpu::wasi::webgpu::surface::Key::ArrowRight) => Some(crate::winit::keyboard::KeyCode::ArrowRight),
                                    _ => None,
                                },
                                None => panic!(),
                            };
                            if let Some(key) = key {
                                event = Some(super::event::WindowEvent::KeyboardInput {
                                    event: KeyEvent {
                                        // logical_key: crate::winit::keyboard::Key::Named(crate::winit::keyboard::NamedKey::R),
                                        state: crate::winit::event::ElementState::Pressed,
                                        physical_key: crate::winit::keyboard::PhysicalKey::Code(key),
                                    }
                                });
                            }
                        }
                        if pollables_res.contains(&1) {
                            let key = match surface.get_key_up() {
                                Some(key) => match key.key {
                                    Some(wgpu::backend::wasi_webgpu::wasi::webgpu::surface::Key::ArrowLeft) => Some(crate::winit::keyboard::KeyCode::ArrowLeft),
                                    Some(wgpu::backend::wasi_webgpu::wasi::webgpu::surface::Key::ArrowRight) => Some(crate::winit::keyboard::KeyCode::ArrowRight),
                                    _ => None,
                                },
                                None => panic!(),
                            };
                            if let Some(key) = key {
                                event = Some(super::event::WindowEvent::KeyboardInput {
                                    event: KeyEvent {
                                        // logical_key: crate::winit::keyboard::Key::Named(crate::winit::keyboard::NamedKey::R),
                                        state: crate::winit::event::ElementState::Released,
                                        physical_key: crate::winit::keyboard::PhysicalKey::Code(key),
                                    }
                                });
                            }
                        }
                        if pollables_res.contains(&2) {
                            // log::info!("resize");
                            // let event = canvas.get_pointer_down();
                            // print(&format!("pointer_down: {:?}", event));
                        }
                        if pollables_res.contains(&3) {
                            // log::info!("frame");
                            event = Some(super::event::WindowEvent::RedrawRequested);
                        }
                //     },
                //     None => {
                //         // log::info!("No surface yet");
                //         wasi::clocks::monotonic_clock::subscribe_duration(500_000_000).block();
                //         event = Some(super::event::WindowEvent::RedrawRequested);
                //     },
                // }

                if let Some(event) = event.take() {
                    log::info!("event {event:#?}");
                    let event = Event::WindowEvent {
                        window_id: WindowId(0),
                        event,
                    };
                    event_handler(event, &self.window_target);
                }
            }
            // Ok(())
        }
    }

    pub struct EventLoopWindowTarget<T: 'static> {
        pub(crate) p: PhantomData<T>,
        pub(crate) _marker: PhantomData<*mut ()>,
        pub(crate) wasi_surface: Mutex<Option<Arc<Surface>>>,
    }
    impl<T> EventLoopWindowTarget<T> {
        pub fn exit(&self) {}
    }

    impl<T> Deref for EventLoop<T> {
        type Target = EventLoopWindowTarget<T>;
        fn deref(&self) -> &EventLoopWindowTarget<T> {
            &self.window_target
        }
    }
}

fn sleep(milis: u32) {
    for i in 0..milis + 1 {
        if i == milis {
            return;
        }
    }
}

pub mod event {
    use super::dpi::PhysicalSize;
    // use crate::winit::window::PhysicalSize;

    #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
    pub enum ElementState {
        Pressed,
        Released,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum WindowEvent {
        CloseRequested,
        RedrawRequested,
        Resized(PhysicalSize),
        KeyboardInput {
            event: KeyEvent,
        },
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Event {
        NewEvents,
        Resumed,
        WindowEvent {
            window_id: WindowId,
            event: WindowEvent,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
    pub struct WindowId(pub(crate) u64);

    /// Describes the reason the event loop is resuming.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum StartCause {
        // /// Sent if the time specified by [`ControlFlow::WaitUntil`] has been reached. Contains the
        // /// moment the timeout was requested and the requested resume time. The actual resume time is
        // /// guaranteed to be equal to or after the requested resume time.
        // ///
        // /// [`ControlFlow::WaitUntil`]: crate::event_loop::ControlFlow::WaitUntil
        // ResumeTimeReached {
        //     start: Instant,
        //     requested_resume: Instant,
        // },

        // /// Sent if the OS has new events to send to the window, after a wait was requested. Contains
        // /// the moment the wait was requested and the resume time, if requested.
        // WaitCancelled {
        //     start: Instant,
        //     requested_resume: Option<Instant>,
        // },
        /// Sent if the event loop is being resumed after the loop's control flow was set to
        /// [`ControlFlow::Poll`].
        ///
        /// [`ControlFlow::Poll`]: crate::event_loop::ControlFlow::Poll
        Poll,

        /// Sent once, immediately after `run` is called. Indicates that the loop was just initialized.
        Init,
    }

    /// Describes a keyboard input targeting a window.
    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct KeyEvent {
        // /// Represents the position of a key independent of the currently active layout.
        // ///
        // /// It also uniquely identifies the physical key (i.e. it's mostly synonymous with a scancode).
        // /// The most prevalent use case for this is games. For example the default keys for the player
        // /// to move around might be the W, A, S, and D keys on a US layout. The position of these keys
        // /// is more important than their label, so they should map to Z, Q, S, and D on an "AZERTY"
        // /// layout. (This value is `KeyCode::KeyW` for the Z key on an AZERTY layout.)
        // ///
        // /// ## Caveats
        // ///
        // /// - Certain niche hardware will shuffle around physical key positions, e.g. a keyboard that
        // /// implements DVORAK in hardware (or firmware)
        // /// - Your application will likely have to handle keyboards which are missing keys that your
        // /// own keyboard has.
        // /// - Certain `KeyCode`s will move between a couple of different positions depending on what
        // /// layout the keyboard was manufactured to support.
        // ///
        // ///  **Because of these caveats, it is important that you provide users with a way to configure
        // ///  most (if not all) keybinds in your application.**
        // ///
        // /// ## `Fn` and `FnLock`
        // ///
        // /// `Fn` and `FnLock` key events are *exceedingly unlikely* to be emitted by Winit. These keys
        // /// are usually handled at the hardware or OS level, and aren't surfaced to applications. If
        // /// you somehow see this in the wild, we'd like to know :)
        pub physical_key: super::keyboard::PhysicalKey,

        // Allowing `broken_intra_doc_links` for `logical_key`, because
        // `key_without_modifiers` is not available on all platforms
        #[cfg_attr(
            not(any(windows_platform, macos_platform, x11_platform, wayland_platform)),
            allow(rustdoc::broken_intra_doc_links)
        )]
        /// This value is affected by all modifiers except <kbd>Ctrl</kbd>.
        ///
        /// This has two use cases:
        /// - Allows querying whether the current input is a Dead key.
        /// - Allows handling key-bindings on platforms which don't
        /// support [`key_without_modifiers`].
        ///
        /// If you use this field (or [`key_without_modifiers`] for that matter) for keyboard
        /// shortcuts, **it is important that you provide users with a way to configure your
        /// application's shortcuts so you don't render your application unusable for users with an
        /// incompatible keyboard layout.**
        ///
        /// ## Platform-specific
        /// - **Web:** Dead keys might be reported as the real key instead
        /// of `Dead` depending on the browser/OS.
        ///
        /// [`key_without_modifiers`]: crate::platform::modifier_supplement::KeyEventExtModifierSupplement::key_without_modifiers
        // pub logical_key: super::keyboard::Key,

        // /// Contains the text produced by this keypress.
        // ///
        // /// In most cases this is identical to the content
        // /// of the `Character` variant of `logical_key`.
        // /// However, on Windows when a dead key was pressed earlier
        // /// but cannot be combined with the character from this
        // /// keypress, the produced text will consist of two characters:
        // /// the dead-key-character followed by the character resulting
        // /// from this keypress.
        // ///
        // /// An additional difference from `logical_key` is that
        // /// this field stores the text representation of any key
        // /// that has such a representation. For example when
        // /// `logical_key` is `Key::Named(NamedKey::Enter)`, this field is `Some("\r")`.
        // ///
        // /// This is `None` if the current keypress cannot
        // /// be interpreted as text.
        // ///
        // /// See also: `text_with_all_modifiers()`
        // pub text: Option<SmolStr>,

        // /// Contains the location of this key on the keyboard.
        // ///
        // /// Certain keys on the keyboard may appear in more than once place. For example, the "Shift" key
        // /// appears on the left side of the QWERTY keyboard as well as the right side. However, both keys
        // /// have the same symbolic value. Another example of this phenomenon is the "1" key, which appears
        // /// both above the "Q" key and as the "Keypad 1" key.
        // ///
        // /// This field allows the user to differentiate between keys like this that have the same symbolic
        // /// value but different locations on the keyboard.
        // ///
        // /// See the [`KeyLocation`] type for more details.
        // ///
        // /// [`KeyLocation`]: crate::keyboard::KeyLocation
        // pub location: keyboard::KeyLocation,

        /// Whether the key is being pressed or released.
        ///
        /// See the [`ElementState`] type for more details.
        pub state: ElementState,

        // /// Whether or not this key is a key repeat event.
        // ///
        // /// On some systems, holding down a key for some period of time causes that key to be repeated
        // /// as though it were being pressed and released repeatedly. This field is `true` if and only if
        // /// this event is the result of one of those repeats.
        // pub repeat: bool,

        // /// Platform-specific key event information.
        // ///
        // /// On Windows, Linux and macOS, this type contains the key without modifiers and the text with all
        // /// modifiers applied.
        // ///
        // /// On Android, iOS, Redox and Web, this type is a no-op.
        // pub(crate) platform_specific: platform_impl::KeyEventExtra,
    }
}

pub mod dpi {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PhysicalSize<P = u32> {
        pub width: P,
        pub height: P,
    }

    impl<P> PhysicalSize<P> {
        pub fn new(width: P, height: P) -> Self {
            Self { width, height }
        }
    }
}

pub mod keyboard {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum NamedKey {
        /// The `Alt` (Alternative) key.
        ///
        /// This key enables the alternate modifier function for interpreting concurrent or subsequent
        /// keyboard input. This key value is also used for the Apple <kbd>Option</kbd> key.
        Alt,
        /// The Alternate Graphics (<kbd>AltGr</kbd> or <kbd>AltGraph</kbd>) key.
        ///
        /// This key is used enable the ISO Level 3 shift modifier (the standard `Shift` key is the
        /// level 2 modifier).
        AltGraph,
        /// The `Caps Lock` (Capital) key.
        ///
        /// Toggle capital character lock function for interpreting subsequent keyboard input event.
        CapsLock,
        /// The `Control` or `Ctrl` key.
        ///
        /// Used to enable control modifier function for interpreting concurrent or subsequent keyboard
        /// input.
        Control,
        /// The Function switch `Fn` key. Activating this key simultaneously with another key changes
        /// that key’s value to an alternate character or function. This key is often handled directly
        /// in the keyboard hardware and does not usually generate key events.
        Fn,
        /// The Function-Lock (`FnLock` or `F-Lock`) key. Activating this key switches the mode of the
        /// keyboard to changes some keys' values to an alternate character or function. This key is
        /// often handled directly in the keyboard hardware and does not usually generate key events.
        FnLock,
        /// The `NumLock` or Number Lock key. Used to toggle numpad mode function for interpreting
        /// subsequent keyboard input.
        NumLock,
        /// Toggle between scrolling and cursor movement modes.
        ScrollLock,
        /// Used to enable shift modifier function for interpreting concurrent or subsequent keyboard
        /// input.
        Shift,
        /// The Symbol modifier key (used on some virtual keyboards).
        Symbol,
        SymbolLock,
        // Legacy modifier key. Also called "Super" in certain places.
        Meta,
        // Legacy modifier key.
        Hyper,
        /// Used to enable "super" modifier function for interpreting concurrent or subsequent keyboard
        /// input. This key value is used for the "Windows Logo" key and the Apple `Command` or `⌘` key.
        ///
        /// Note: In some contexts (e.g. the Web) this is referred to as the "Meta" key.
        Super,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum Key {
        /// A simple (unparameterised) action
        Named(NamedKey),

        // /// A key string that corresponds to the character typed by the user, taking into account the
        // /// user’s current locale setting, and any system-level keyboard mapping overrides that are in
        // /// effect.
        // Character(Str),

        // /// This variant is used when the key cannot be translated to any other variant.
        // ///
        // /// The native key is provided (if available) in order to allow the user to specify keybindings
        // /// for keys which are not defined by this API, mainly through some sort of UI.
        // Unidentified(NativeKey),
        /// Contains the text representation of the dead-key when available.
        ///
        /// ## Platform-specific
        /// - **Web:** Always contains `None`
        Dead(Option<char>),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum PhysicalKey {
        /// A known key code
        Code(KeyCode),
        // /// This variant is used when the key cannot be translated to a [`KeyCode`]
        // ///
        // /// The native keycode is provided (if available) so you're able to more reliably match
        // /// key-press and key-release events by hashing the [`PhysicalKey`]. It is also possible to use
        // /// this for keybinds for non-standard keys, but such keybinds are tied to a given platform.
        // Unidentified(NativeKeyCode),
    }

    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum KeyCode {
        /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
        /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
        /// (hankaku/zenkaku/kanji) key on Japanese keyboards
        Backquote,
        /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
        /// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
        /// 104- and 106-key layouts.
        /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
        Backslash,
        /// <kbd>[</kbd> on a US keyboard.
        BracketLeft,
        /// <kbd>]</kbd> on a US keyboard.
        BracketRight,
        /// <kbd>,</kbd> on a US keyboard.
        Comma,
        /// <kbd>0</kbd> on a US keyboard.
        Digit0,
        /// <kbd>1</kbd> on a US keyboard.
        Digit1,
        /// <kbd>2</kbd> on a US keyboard.
        Digit2,
        /// <kbd>3</kbd> on a US keyboard.
        Digit3,
        /// <kbd>4</kbd> on a US keyboard.
        Digit4,
        /// <kbd>5</kbd> on a US keyboard.
        Digit5,
        /// <kbd>6</kbd> on a US keyboard.
        Digit6,
        /// <kbd>7</kbd> on a US keyboard.
        Digit7,
        /// <kbd>8</kbd> on a US keyboard.
        Digit8,
        /// <kbd>9</kbd> on a US keyboard.
        Digit9,
        /// <kbd>=</kbd> on a US keyboard.
        Equal,
        /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
        /// Labeled <kbd>\\</kbd> on a UK keyboard.
        IntlBackslash,
        /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
        /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
        IntlRo,
        /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
        /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
        /// Russian keyboard.
        IntlYen,
        /// <kbd>a</kbd> on a US keyboard.
        /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
        KeyA,
        /// <kbd>b</kbd> on a US keyboard.
        KeyB,
        /// <kbd>c</kbd> on a US keyboard.
        KeyC,
        /// <kbd>d</kbd> on a US keyboard.
        KeyD,
        /// <kbd>e</kbd> on a US keyboard.
        KeyE,
        /// <kbd>f</kbd> on a US keyboard.
        KeyF,
        /// <kbd>g</kbd> on a US keyboard.
        KeyG,
        /// <kbd>h</kbd> on a US keyboard.
        KeyH,
        /// <kbd>i</kbd> on a US keyboard.
        KeyI,
        /// <kbd>j</kbd> on a US keyboard.
        KeyJ,
        /// <kbd>k</kbd> on a US keyboard.
        KeyK,
        /// <kbd>l</kbd> on a US keyboard.
        KeyL,
        /// <kbd>m</kbd> on a US keyboard.
        KeyM,
        /// <kbd>n</kbd> on a US keyboard.
        KeyN,
        /// <kbd>o</kbd> on a US keyboard.
        KeyO,
        /// <kbd>p</kbd> on a US keyboard.
        KeyP,
        /// <kbd>q</kbd> on a US keyboard.
        /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
        KeyQ,
        /// <kbd>r</kbd> on a US keyboard.
        KeyR,
        /// <kbd>s</kbd> on a US keyboard.
        KeyS,
        /// <kbd>t</kbd> on a US keyboard.
        KeyT,
        /// <kbd>u</kbd> on a US keyboard.
        KeyU,
        /// <kbd>v</kbd> on a US keyboard.
        KeyV,
        /// <kbd>w</kbd> on a US keyboard.
        /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
        KeyW,
        /// <kbd>x</kbd> on a US keyboard.
        KeyX,
        /// <kbd>y</kbd> on a US keyboard.
        /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
        KeyY,
        /// <kbd>z</kbd> on a US keyboard.
        /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
        /// QWERTZ (e.g., German) keyboard.
        KeyZ,
        /// <kbd>-</kbd> on a US keyboard.
        Minus,
        /// <kbd>.</kbd> on a US keyboard.
        Period,
        /// <kbd>'</kbd> on a US keyboard.
        Quote,
        /// <kbd>;</kbd> on a US keyboard.
        Semicolon,
        /// <kbd>/</kbd> on a US keyboard.
        Slash,
        /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
        AltLeft,
        /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
        /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
        AltRight,
        /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
        /// Labeled <kbd>Delete</kbd> on Apple keyboards.
        Backspace,
        /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
        CapsLock,
        /// The application context menu key, which is typically found between the right
        /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
        ContextMenu,
        /// <kbd>Control</kbd> or <kbd>⌃</kbd>
        ControlLeft,
        /// <kbd>Control</kbd> or <kbd>⌃</kbd>
        ControlRight,
        /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
        Enter,
        /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
        SuperLeft,
        /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
        SuperRight,
        /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
        ShiftLeft,
        /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
        ShiftRight,
        /// <kbd> </kbd> (space)
        Space,
        /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
        Tab,
        /// Japanese: <kbd>変</kbd> (henkan)
        Convert,
        /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd>
        /// (katakana/hiragana/romaji)
        KanaMode,
        /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
        ///
        /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
        Lang1,
        /// Korean: Hanja <kbd>한</kbd> (hanja)
        ///
        /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
        Lang2,
        /// Japanese (word-processing keyboard): Katakana
        Lang3,
        /// Japanese (word-processing keyboard): Hiragana
        Lang4,
        /// Japanese (word-processing keyboard): Zenkaku/Hankaku
        Lang5,
        /// Japanese: <kbd>無変換</kbd> (muhenkan)
        NonConvert,
        /// <kbd>⌦</kbd>. The forward delete key.
        /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
        /// the keyboard is encoded as [`Backspace`].
        ///
        /// [`Backspace`]: Self::Backspace
        Delete,
        /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
        End,
        /// <kbd>Help</kbd>. Not present on standard PC keyboards.
        Help,
        /// <kbd>Home</kbd> or <kbd>↖</kbd>
        Home,
        /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
        Insert,
        /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
        PageDown,
        /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
        PageUp,
        /// <kbd>↓</kbd>
        ArrowDown,
        /// <kbd>←</kbd>
        ArrowLeft,
        /// <kbd>→</kbd>
        ArrowRight,
        /// <kbd>↑</kbd>
        ArrowUp,
        /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
        NumLock,
        /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
        Numpad0,
        /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote
        /// control
        Numpad1,
        /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
        Numpad2,
        /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
        Numpad3,
        /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
        Numpad4,
        /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
        Numpad5,
        /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
        Numpad6,
        /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
        /// or remote control
        Numpad7,
        /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
        Numpad8,
        /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
        /// or remote control
        Numpad9,
        /// <kbd>+</kbd>
        NumpadAdd,
        /// Found on the Microsoft Natural Keyboard.
        NumpadBackspace,
        /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
        /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
        /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
        ///
        /// [`NumLock`]: Self::NumLock
        NumpadClear,
        /// <kbd>C</kbd> (Clear Entry)
        NumpadClearEntry,
        /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
        /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
        NumpadComma,
        /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
        /// Brazil), this key may generate a <kbd>,</kbd>.
        NumpadDecimal,
        /// <kbd>/</kbd>
        NumpadDivide,
        NumpadEnter,
        /// <kbd>=</kbd>
        NumpadEqual,
        /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
        /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
        NumpadHash,
        /// <kbd>M</kbd> Add current entry to the value stored in memory.
        NumpadMemoryAdd,
        /// <kbd>M</kbd> Clear the value stored in memory.
        NumpadMemoryClear,
        /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
        NumpadMemoryRecall,
        /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
        NumpadMemoryStore,
        /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
        NumpadMemorySubtract,
        /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
        /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
        ///
        /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
        NumpadMultiply,
        /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
        NumpadParenLeft,
        /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
        NumpadParenRight,
        /// <kbd>*</kbd> on a phone or remote control device.
        ///
        /// This key is typically found below the <kbd>7</kbd> key and to the left of
        /// the <kbd>0</kbd> key.
        ///
        /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
        /// numeric keypads.
        NumpadStar,
        /// <kbd>-</kbd>
        NumpadSubtract,
        /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
        Escape,
        /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
        Fn,
        /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
        /// Natural Keyboard.
        FnLock,
        /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
        PrintScreen,
        /// <kbd>Scroll Lock</kbd>
        ScrollLock,
        /// <kbd>Pause Break</kbd>
        Pause,
        /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
        ///
        /// This also the "back" button (triangle) on Android.
        BrowserBack,
        BrowserFavorites,
        /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
        BrowserForward,
        /// The "home" button on Android.
        BrowserHome,
        BrowserRefresh,
        BrowserSearch,
        BrowserStop,
        /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
        /// keyboards.
        Eject,
        /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
        LaunchApp1,
        /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
        LaunchApp2,
        LaunchMail,
        MediaPlayPause,
        MediaSelect,
        MediaStop,
        MediaTrackNext,
        MediaTrackPrevious,
        /// This key is placed in the function section on some Apple keyboards, replacing the
        /// <kbd>Eject</kbd> key.
        Power,
        Sleep,
        AudioVolumeDown,
        AudioVolumeMute,
        AudioVolumeUp,
        WakeUp,
        // Legacy modifier key. Also called "Super" in certain places.
        Meta,
        // Legacy modifier key.
        Hyper,
        Turbo,
        Abort,
        Resume,
        Suspend,
        /// Found on Sun’s USB keyboard.
        Again,
        /// Found on Sun’s USB keyboard.
        Copy,
        /// Found on Sun’s USB keyboard.
        Cut,
        /// Found on Sun’s USB keyboard.
        Find,
        /// Found on Sun’s USB keyboard.
        Open,
        /// Found on Sun’s USB keyboard.
        Paste,
        /// Found on Sun’s USB keyboard.
        Props,
        /// Found on Sun’s USB keyboard.
        Select,
        /// Found on Sun’s USB keyboard.
        Undo,
        /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
        Hiragana,
        /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
        Katakana,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F1,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F2,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F3,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F4,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F5,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F6,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F7,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F8,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F9,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F10,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F11,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F12,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F13,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F14,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F15,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F16,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F17,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F18,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F19,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F20,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F21,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F22,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F23,
        /// General-purpose function key.
        /// Usually found at the top of the keyboard.
        F24,
        /// General-purpose function key.
        F25,
        /// General-purpose function key.
        F26,
        /// General-purpose function key.
        F27,
        /// General-purpose function key.
        F28,
        /// General-purpose function key.
        F29,
        /// General-purpose function key.
        F30,
        /// General-purpose function key.
        F31,
        /// General-purpose function key.
        F32,
        /// General-purpose function key.
        F33,
        /// General-purpose function key.
        F34,
        /// General-purpose function key.
        F35,
    }
}
