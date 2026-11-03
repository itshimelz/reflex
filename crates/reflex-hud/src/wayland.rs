//! Wayland layer-shell window integration via smithay-client-toolkit.

use crate::render::{render_pill, FontContext, PillItem};
use crate::spring::{Spring, BOUNCY, FADE, HIDDEN_SCALE, REDUCED_FADE};
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
    shm::{
        slot::SlotPool,
        Shm, ShmHandler,
    },
};
use std::time::{Duration, Instant};
use tiny_skia::PixmapMut;
use wayland_client::{
    protocol::{wl_output, wl_region, wl_shm, wl_surface},
    Connection, Dispatch, QueueHandle,
};

#[derive(Debug, PartialEq, Eq)]
pub enum PillPhase {
    Entering,
    Holding,
    Exiting,
    Paused,
}

pub struct HudState {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub pool: SlotPool,
    pub layer: Option<LayerSurface>,
    pub width: u32,
    pub height: u32,
    pub configured: bool,
    pub fonts: FontContext,
    #[allow(dead_code)]
    pub qh: QueueHandle<HudState>,

    // Demo sequence & active item
    pub demo_mode: bool,
    pub demo_items: Vec<PillItem>,
    pub demo_idx: usize,
    pub current_item: Option<PillItem>,

    // Hex analytical spring physics & animation phase
    pub phase: PillPhase,
    pub scale_spring: Spring,
    pub opacity_spring: Spring,
    pub reduce_motion: bool,
    pub phase_start: Instant,
    pub default_hold: Duration,
    pub hold_duration: Duration,
    pub pause_duration: Duration,
    pub last_tick: Instant,
    pub needs_clear: bool,
}

impl HudState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry_state: RegistryState,
        output_state: OutputState,
        shm: Shm,
        pool: SlotPool,
        width: u32,
        height: u32,
        qh: QueueHandle<HudState>,
        demo_items: Vec<PillItem>,
        default_hold: Duration,
        reduce_motion: bool,
        demo_mode: bool,
    ) -> Self {
        let fonts = FontContext::load_system_fonts();

        // Exact Hex initial state: scale at HIDDEN_SCALE (0.9), opacity at 0.0 (or 1.0 if reduced motion)
        let init_scale = if reduce_motion { 1.0 } else { HIDDEN_SCALE };
        let mut scale_spring = Spring::at(init_scale);
        let mut opacity_spring = Spring::at(0.0);

        let (phase, current_item) = if demo_mode {
            scale_spring.target = 1.0;
            opacity_spring.target = 1.0;
            (PillPhase::Entering, demo_items.first().cloned())
        } else {
            scale_spring.target = init_scale;
            opacity_spring.target = 0.0;
            (PillPhase::Paused, None)
        };

        Self {
            registry_state,
            output_state,
            shm,
            pool,
            layer: None,
            width,
            height,
            configured: false,
            fonts,
            qh,
            demo_mode,
            demo_items,
            demo_idx: 0,
            current_item,
            phase,
            scale_spring,
            opacity_spring,
            reduce_motion,
            phase_start: Instant::now(),
            default_hold,
            hold_duration: default_hold,
            pause_duration: Duration::from_millis(500),
            last_tick: Instant::now(),
            needs_clear: true,
        }
    }

    pub fn show_event(&mut self, event: reflex_core::hud::HudEvent) {
        let hold = event
            .hold_ms
            .map(Duration::from_millis)
            .unwrap_or(self.default_hold);
        let item: PillItem = event.into();

        if self.phase == PillPhase::Holding || self.phase == PillPhase::Entering {
            // Already visible: in-place tactile update and reset hold timer
            self.current_item = Some(item);
            self.hold_duration = hold;
            self.phase = PillPhase::Holding;
            self.phase_start = Instant::now();
            self.scale_spring.v = 0.35; // tactile micro-pop
            self.needs_clear = true;
        } else {
            // Emerge from hidden state
            self.current_item = Some(item);
            self.hold_duration = hold;
            self.phase = PillPhase::Entering;
            self.phase_start = Instant::now();
            if self.reduce_motion {
                self.scale_spring.jump(1.0);
                self.opacity_spring.jump(0.0);
                self.scale_spring.target = 1.0;
                self.opacity_spring.target = 1.0;
            } else {
                self.scale_spring.jump(HIDDEN_SCALE);
                self.opacity_spring.jump(0.0);
                self.scale_spring.target = 1.0;
                self.opacity_spring.target = 1.0;
            }
            self.needs_clear = true;
        }
    }

    pub fn tick_animation(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f32().min(0.05);
        self.last_tick = now;

        if !self.demo_mode && self.phase == PillPhase::Paused {
            if self.needs_clear {
                self.clear_frame();
                self.needs_clear = false;
            }
            return;
        }

        if self.reduce_motion {
            // Accessible reduced motion: zero bounce, quick opacity fade
            match self.phase {
                PillPhase::Entering => {
                    self.scale_spring.jump(1.0);
                    self.opacity_spring.step_to(1.0, REDUCED_FADE, dt);
                    if self.opacity_spring.x >= 0.98 || self.phase_start.elapsed() > Duration::from_millis(200) {
                        self.phase = PillPhase::Holding;
                        self.phase_start = Instant::now();
                    }
                }
                PillPhase::Holding => {
                    self.scale_spring.jump(1.0);
                    self.opacity_spring.jump(1.0);
                    if self.phase_start.elapsed() >= self.hold_duration {
                        self.phase = PillPhase::Exiting;
                        self.phase_start = Instant::now();
                    }
                }
                PillPhase::Exiting => {
                    self.scale_spring.jump(1.0);
                    self.opacity_spring.step_to(0.0, REDUCED_FADE, dt);
                    if self.opacity_spring.x <= 0.02 || self.phase_start.elapsed() > Duration::from_millis(200) {
                        self.phase = PillPhase::Paused;
                        self.phase_start = Instant::now();
                        self.clear_frame();
                        self.needs_clear = false;
                    }
                }
                PillPhase::Paused => {
                    if self.demo_mode && self.phase_start.elapsed() >= self.pause_duration {
                        self.demo_idx = (self.demo_idx + 1) % self.demo_items.len();
                        self.current_item = self.demo_items.get(self.demo_idx).cloned();
                        self.phase = PillPhase::Entering;
                        self.scale_spring.jump(1.0);
                        self.opacity_spring.jump(0.0);
                        self.phase_start = Instant::now();
                    }
                }
            }
        } else {
            // Exact Hex spring physics with Bouncy scale overshoot & settle
            match self.phase {
                PillPhase::Entering => {
                    self.scale_spring.step_to(1.0, BOUNCY, dt);
                    self.opacity_spring.step_to(1.0, FADE, dt);

                    if (self.scale_spring.is_settled() && self.opacity_spring.is_settled())
                        || self.phase_start.elapsed() > Duration::from_millis(380)
                    {
                        self.phase = PillPhase::Holding;
                        self.phase_start = Instant::now();
                    }
                }
                PillPhase::Holding => {
                    self.scale_spring.step_to(1.0, BOUNCY, dt);
                    self.opacity_spring.step_to(1.0, FADE, dt);

                    if self.phase_start.elapsed() >= self.hold_duration {
                        self.phase = PillPhase::Exiting;
                        self.phase_start = Instant::now();
                    }
                }
                PillPhase::Exiting => {
                    self.scale_spring.step_to(HIDDEN_SCALE, BOUNCY, dt);
                    self.opacity_spring.step_to(0.0, FADE, dt);

                    if self.opacity_spring.x <= 0.02 || self.phase_start.elapsed() > Duration::from_millis(380) {
                        self.phase = PillPhase::Paused;
                        self.phase_start = Instant::now();
                        self.clear_frame();
                        self.needs_clear = false;
                    }
                }
                PillPhase::Paused => {
                    if self.demo_mode && self.phase_start.elapsed() >= self.pause_duration {
                        self.demo_idx = (self.demo_idx + 1) % self.demo_items.len();
                        self.current_item = self.demo_items.get(self.demo_idx).cloned();
                        self.phase = PillPhase::Entering;
                        self.scale_spring.jump(HIDDEN_SCALE);
                        self.opacity_spring.jump(0.0);
                        self.scale_spring.target = 1.0;
                        self.opacity_spring.target = 1.0;
                        self.phase_start = Instant::now();
                    }
                }
            }
        }

        let scale = self.scale_spring.x.max(0.0);
        let alpha = self.opacity_spring.x.clamp(0.0, 1.0);

        if alpha <= 0.001 {
            if self.needs_clear {
                self.clear_frame();
                self.needs_clear = false;
            }
            return;
        }

        // Render current frame
        if let Some(item) = self.current_item.clone() {
            self.draw_frame(&item, scale, alpha);
        }
    }

    pub fn clear_frame(&mut self) {
        let Some(ref layer) = self.layer else {
            return;
        };

        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(buf) => buf,
            Err(_) => return,
        };

        // Fill with zero alpha (transparent)
        canvas.fill(0);

        let wl_surf = layer.wl_surface();
        wl_surf.damage_buffer(0, 0, width as i32, height as i32);
        let _ = buffer.attach_to(wl_surf);
        layer.commit();
    }

    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn draw_frame(
        &mut self,
        item: &PillItem,
        spring_scale: f32,
        alpha: f32,
    ) {
        let Some(ref layer) = self.layer else {
            return;
        };

        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(buf) => buf,
            Err(_) => return,
        };

        // Render directly to the shared memory canvas
        if let Some(mut pixmap) = PixmapMut::from_bytes(canvas, width, height) {
            render_pill(&mut pixmap, &self.fonts, item, spring_scale, alpha);

            // Convert tiny-skia's RGBA pixel order to Wayland's Argb8888 (BGRA on little endian x86_64)
            for pixel in canvas.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
        }

        let wl_surf = layer.wl_surface();
        wl_surf.damage_buffer(0, 0, width as i32, height as i32);
        let _ = buffer.attach_to(wl_surf);
        layer.commit();
    }
}

impl CompositorHandler for HudState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for HudState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl ShmHandler for HudState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl LayerShellHandler for HudState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        self.configured = true;
    }
}

impl Dispatch<wl_region::WlRegion, ()> for HudState {
    fn event(
        _state: &mut Self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_registry!(HudState);

impl ProvidesRegistryState for HudState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

smithay_client_toolkit::delegate_dispatch2!(HudState);
