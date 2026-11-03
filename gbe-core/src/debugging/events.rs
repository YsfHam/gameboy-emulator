use std::collections::VecDeque;

use crate::{interrupts::Interrupt, ppu::PpuMode};

pub(crate) const DEFAULT_EVENT_BUFFER_CAPACITY: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventComponent {
    Cpu,
    Bus,
    Ppu,
    Timer,
    Cartridge,
    Interrupts,
    Emulator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    CpuInstruction { pc: u16, opcode: u8 },
    InterruptServiced { interrupt: Interrupt },
    PpuModeChanged { mode: PpuMode, ly: u8 },
    LcdSwitch { on: bool },
    FrameCompleted,
    TimerReloaded { value: u8 },
}

impl EventKind {
    fn component(&self) -> EventComponent {
        match self {
            Self::CpuInstruction { .. } => EventComponent::Cpu,
            Self::InterruptServiced { .. } => EventComponent::Interrupts,
            Self::PpuModeChanged { .. } | Self::FrameCompleted => EventComponent::Ppu,
            Self::TimerReloaded { .. } => EventComponent::Timer,
            Self::LcdSwitch { .. } => EventComponent::Ppu,
        }
    }

    fn level(&self) -> EventLevel {
        match self {
            Self::CpuInstruction { .. } | Self::FrameCompleted => EventLevel::Trace,
            Self::InterruptServiced { .. } | Self::PpuModeChanged { .. } | Self::TimerReloaded { .. } => EventLevel::Debug,
            Self::LcdSwitch { .. } => EventLevel::Info,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    cycle: u64,
    kind: EventKind,
}

impl Event {
    pub(crate) fn new(cycle: u64, kind: EventKind) -> Self {
        Self { cycle, kind }
    }

    pub fn level(&self) -> EventLevel {
        self.kind.level()
    }

    pub fn component(&self) -> EventComponent {
        self.kind.component()
    }
}

#[derive(Debug, Clone)]
pub struct EventFilter {
    minimum_level: EventLevel,
    components: Vec<EventComponent>,
}

impl EventFilter {
    pub fn new(minimum_level: EventLevel) -> Self {
        Self {
            minimum_level,
            components: Vec::new(),
        }
    }

    pub fn with_components(mut self, components: impl IntoIterator<Item = EventComponent>) -> Self {
        self.components = components.into_iter().collect();
        self
    }

    fn accepts(&self, event: &Event) -> bool {
        event.level() >= self.minimum_level
            && (self.components.is_empty() || self.components.contains(&event.component()))
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new(EventLevel::Info)
    }
}

#[derive(Debug)]
pub(crate) struct EventBuffer {
    events: VecDeque<Event>,
    capacity: usize,
    filter: EventFilter,
    dropped_events: u64,
}

impl EventBuffer {
    pub(crate) fn new(capacity: usize, filter: EventFilter) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            filter,
            dropped_events: 0,
        }
    }

    pub(crate) fn set_filter(&mut self, filter: EventFilter) {
        self.filter = filter;
    }

    pub(crate) fn push(&mut self, event: Event) {
        if !self.filter.accepts(&event) {
            return;
        }
        if self.events.len() == self.capacity {
            self.dropped_events = self.dropped_events.saturating_add(1);
            return;
        }
        self.events.push_back(event);
    }

    pub(crate) fn drain(&mut self) -> Vec<Event> {
        self.events.drain(..).collect()
    }
}