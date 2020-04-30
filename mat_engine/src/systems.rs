//! In constructing this engine, I've noticed that, very often, one system wishes to
//! access another system. While perhaps there might be ways to avoid some of this
//! cross referencing, there is also value in allowing the possibility of quasi-global
//! systems that can easily be accessed and manipulated.
//!
//! The systems module implements the necessary utilities to manage this situation

use crate::{imgui::ImguiSystem, rendering::RenderingSystem, windowing::WindowingSystem};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

// TODO: Investigate Option<RefCell<T>> vs RefCell<Option<T>>
// I think Option<RefCell<T>> is probably a nicer API to use, even if it means
// you need another RefCell wrapping around Systems (see Engine).

type SysCell<T> = Option<RefCell<T>>;
type BoxErr = Box<dyn std::error::Error>;

/// The `Engine` object encapsulates all the state necessary for the engine to run.
/// It represents a form of quasi-global state, specifically subdivided into systems.
/// The user receives a reference to the engine in `Application` trait functions.
#[derive(Debug)]
pub struct Engine {
    systems: Rc<RefCell<Systems>>,
}

impl Engine {
    /// Create new Engine with all systems uninitialized.
    pub(crate) fn uninit() -> Engine {
        Engine {
            systems: Rc::new(RefCell::new(Systems::uninit())),
        }
    }

    // TODO: Maybe remove and simply pass rc to the systems
    /// Important note: When you borrow the `Systems` object from the `RefCell`, make sure
    /// NOT to store any of the `Ref` or `RefMut` objects, as that could cause conflicts if someone
    /// needs to mutably borrow the whole `Systems` object. Try to Drop them ASAP.
    ///
    /// On the other hand, `Weak<RefCell<Systems>>`, which is what this function returns,
    /// may be stored. Indeed, we expect that you will store it. As a matter of fact, the only
    /// reason the `Engine` object exists is to simplify `Application`'s API.
    /// In the future, `Engine` may gain further functionality, but for now, it is only a wrapper
    /// around `Rc<RefCell<Systems>>`.
    ///
    /// When you actually need to use, upgrade the `rc::Weak` into `rc::Rc`. In theory, storing the
    /// `Rc` shouldn't be a problem, and in fact might even be slightly faster. However, storing
    /// `Weak` better expresses the fact that `RefCell<Systems>` is NOT owned by the systems,
    /// there IS NO SHARED OWNERSHIP, we only use Rc/Weak because borrowck doesn't support
    /// self-referential objects.
    pub fn systems_ref(&self) -> Weak<RefCell<Systems>> {
        Rc::downgrade(&self.systems)
    }

    /// Attention: Do NOT store `Rc<RefCell<Systems>>` anywhere except inside of
    /// a specific system located inside Systems.
    ///
    /// See impl of `Drop` trait for `Engine` to get an explanation for this.
    pub(crate) fn systems_rc(&self) -> Rc<RefCell<Systems>> {
        self.systems.clone()
    }

    pub fn systems_borrow(&self) -> Ref<Systems> {
        self.systems.borrow()
    }

    pub fn systems_borrow_mut(&self) -> RefMut<Systems> {
        self.systems.borrow_mut()
    }
}

impl std::ops::Drop for Engine {
    /// Guards against leakage of `Systems`. We may, at some point, choose to give out `Rc`s to users
    /// or pass them into systems, but we wish to ensure that they don't outlive Engine.
    /// That is because we want to be sure that `Systems` will be dropped.
    fn drop(&mut self) {
        log::trace!("Dropping Engine");
        // The one place we expect `Rc` references to `Systems` is inside specific systems.
        // Therefore, we drop all systems here.
        // TODO: Test...
        std::mem::drop(self.systems.replace(Systems::uninit()));
        // Do the actual guarding
        if std::rc::Rc::strong_count(&self.systems) != 1 {
            log::error!(
                "Dropping Engine but unable to drop Systems because someone else holds a \
                 strong reference (Rc) to it"
            );
            panic!("Engine dropped, cannot drop Systems");
        }
        // Since the last Rc will be dropped automatically, `Systems` should be dropped.
    }
}

/// Important note: When you borrow (specific) systems from the `RefCell`, try not to keep references
/// alive. Instead, borrow when you need, and then drop those references once you finish using them.
/// Do NOT store the `Ref` or `RefMut` objects, that could cause conflicts if someone needs to
/// mutably borrow a system. Try to Drop them ASAP.
pub struct Systems {
    windowing: SysCell<WindowingSystem>,
    rendering: SysCell<RenderingSystem>,
    imgui: SysCell<ImguiSystem>,
}

impl Systems {
    fn uninit() -> Systems {
        Self {
            windowing: None,
            rendering: None,
            imgui: None,
        }
    }

    pub fn has_windowing(&self) -> bool {
        match self.windowing {
            None => false,
            Some(_) => true,
        }
    }

    pub fn set_windowing(&mut self, new_sys: SysCell<WindowingSystem>) {
        self.windowing = new_sys;
    }

    pub fn windowing(&self) -> Result<Ref<WindowingSystem>, BoxErr> {
        Ok(self
            .windowing
            .as_ref()
            .ok_or_else(|| Box::new(UninitializedSystemError))?
            .try_borrow()?)
    }

    pub fn windowing_mut(&self) -> Result<RefMut<WindowingSystem>, BoxErr> {
        Ok(self
            .windowing
            .as_ref()
            .ok_or_else(|| Box::new(UninitializedSystemError))?
            .try_borrow_mut()?)
    }

    pub fn has_rendering(&self) -> bool {
        match self.rendering {
            None => false,
            Some(_) => true,
        }
    }

    pub fn set_rendering(&mut self, new_sys: SysCell<RenderingSystem>) {
        self.rendering = new_sys;
    }

    pub fn rendering(&self) -> Result<Ref<RenderingSystem>, BoxErr> {
        Ok(self
            .rendering
            .as_ref()
            .ok_or_else(|| Box::new(UninitializedSystemError))?
            .try_borrow()?)
    }

    pub fn rendering_mut(&self) -> Result<RefMut<RenderingSystem>, BoxErr> {
        Ok(self
            .rendering
            .as_ref()
            .ok_or_else(|| Box::new(UninitializedSystemError))?
            .try_borrow_mut()?)
    }

    pub fn has_imgui(&self) -> bool {
        match self.imgui {
            None => false,
            Some(_) => true,
        }
    }

    pub fn set_imgui(&mut self, new_sys: SysCell<ImguiSystem>) {
        self.imgui = new_sys;
    }

    pub fn imgui(&self) -> Result<Ref<ImguiSystem>, BoxErr> {
        Ok(self
            .imgui
            .as_ref()
            .ok_or_else(|| Box::new(UninitializedSystemError))?
            .try_borrow()?)
    }

    pub fn imgui_mut(&self) -> Result<RefMut<ImguiSystem>, BoxErr> {
        Ok(self
            .imgui
            .as_ref()
            .ok_or_else(|| Box::new(UninitializedSystemError))?
            .try_borrow_mut()?)
    }
}

impl std::fmt::Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Systems>")
    }
}

#[derive(Debug, Copy, Clone)]
struct UninitializedSystemError;

impl std::fmt::Display for UninitializedSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The system you attempted to access is uninitialized")
    }
}

impl std::error::Error for UninitializedSystemError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
