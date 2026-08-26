//! Resource Blocks Panel
//!
//! Displays a visual grid of Physical Resource Blocks (PRBs) × Symbols
//! representing the 5G NR resource grid allocation.
//!
//! Uses real PHY trace data (PDSCH/PUSCH) to populate the grid.

use crate::event_system::{EventContext, EventSubscriber};
use crate::theme::ThemeColors;
use egui::{Color32, PopupAnchor, Pos2, Rect, Stroke, StrokeKind, Vec2};
use tramex_tools::interface::parser::parser_phy::PHYInfos;
use tramex_tools::{data::AdditionalInfos, data::Trace, errors::TramexError};

/// Number of PRBs (rows) - typical 5G NR bandwidth
const DEFAULT_NUM_PRBS: usize = 51;

/// Number of frames per hyper frame
const DEFAULT_FRAME_PER_HFN: usize = 1024;

/// Number of subframes per frame
const DEFAULT_SUBFRAME_PER_FRAME: usize = 10;

/// Default slots per subframe
const DEFAULT_SLOTS_PER_SUBFRAME: u8 = 2;

/// Number of symbols per slot
const SYMBOLS_PER_SLOT: usize = 14;

/// Number of frames to display (1 frame = 10ms)
const DEFAULT_WINDOW_FRAMES: u8 = 10;

/// Stroke Kind style
const STROKE_KIND_STYLE: StrokeKind = StrokeKind::Middle;

/// Anchor
const TOOLTIP_ANCHOR: PopupAnchor = PopupAnchor::Pointer;

/// Cell type representing what occupies a resource element
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ResourceType {
    /// Empty resource element (white)
    #[default]
    Empty,
    /// PDCCH - Control channel (gray)
    Pdcch,
    /// PUCCH - Uplink control (yellow)
    Pucch,
    /// PDSCH - Downlink data (blue)
    Pdsch,
    /// PUSCH - Uplink data (cyan/teal)
    Pusch,
    /// PRACH - Random Access Channel (orange)
    Prach,
    /// SSB - Synchronization Signal Block (magenta)
    Ssb,
    /// DMRS - Demodulation Reference Signal (red dot)
    Dmrs,
    /// Guard period (dark gray)
    Guard,
}

impl ResourceType {
    /// Get the color for this resource type
    pub fn color(&self, is_dark: bool) -> Color32 {
        match self {
            // Empty uses theme background in dark mode, white in light mode
            ResourceType::Empty => {
                if is_dark {
                    Color32::from_gray(40) // Dark gray for dark mode
                } else {
                    Color32::WHITE
                }
            }
            ResourceType::Pdcch => Color32::from_rgb(0, 100, 0), // Dark green
            ResourceType::Pucch => Color32::from_rgb(0, 200, 0), // Light green
            ResourceType::Pdsch => Color32::from_rgb(0, 100, 200), // Blue
            ResourceType::Pusch => Color32::from_rgb(0, 180, 180), // Cyan/Teal
            ResourceType::Prach => Color32::from_rgb(255, 220, 0), // Yellow
            ResourceType::Ssb => Color32::from_rgb(200, 0, 200), // Magenta
            ResourceType::Dmrs => Color32::from_rgb(200, 0, 0),  // Red
            ResourceType::Guard => Color32::from_rgb(100, 100, 100), // Dark gray
        }
    }

    /// Get priority for slot view aggregation (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            ResourceType::Empty => 0,
            ResourceType::Dmrs => 1,
            ResourceType::Ssb => 2,
            ResourceType::Prach => 3,
            ResourceType::Pdcch => 4,
            ResourceType::Pucch => 5,
            ResourceType::Pusch => 6,
            ResourceType::Pdsch => 7,
            ResourceType::Guard => 0,
        }
    }
}

/// View mode for resource grid display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
pub enum ViewMode {
    /// Symbol-level granularity (1 square = 1 PRB × 1 symbol)
    #[default]
    Symbol,
    /// Slot-level granularity (1 square = 1 PRB × 1 slot)
    Slot,
}

/// A single slot's resource grid (PRBs × Symbols)
#[derive(Debug, Clone)]
pub struct SlotGrid {
    /// Hyper Frame Number
    pub hfn: u32,
    /// Frame number (0-1023)
    pub frame_number: u16,
    /// Slot number within the frame (0-19 for 2 slots/subframe)
    pub slot_number: u8,
    /// Grid data: [prb][symbol] -> ResourceType
    pub grid: Vec<Vec<ResourceType>>,
    /// Event indices for each cell (trace_index of the PHY event that filled it)
    pub event_indices: Vec<Vec<Option<usize>>>,
    /// SSB IDs for each cell (stores SSB config ID if cell contains SSB)
    pub ssb_ids: Vec<Vec<Option<usize>>>,
}

impl SlotGrid {
    /// Create a new empty slot grid
    pub fn new(hfn: u32, frame_number: u16, slot_number: u8, num_prbs: usize) -> Self {
        Self {
            hfn,
            frame_number,
            slot_number,
            grid: vec![vec![ResourceType::Empty; SYMBOLS_PER_SLOT]; num_prbs],
            event_indices: vec![vec![None; SYMBOLS_PER_SLOT]; num_prbs],
            ssb_ids: vec![vec![None; SYMBOLS_PER_SLOT]; num_prbs],
        }
    }

    /// Set a resource at a specific PRB and symbol
    pub fn set(&mut self, prb: usize, symbol: usize, resource_type: ResourceType, event_idx: Option<usize>) {
        if prb < self.grid.len() && symbol < SYMBOLS_PER_SLOT {
            self.grid[prb][symbol] = resource_type;
            self.event_indices[prb][symbol] = event_idx;
            // Clear SSB ID when setting non-SSB resources
            if resource_type != ResourceType::Ssb {
                self.ssb_ids[prb][symbol] = None;
            }
        }
    }

    /// Set a resource at a specific PRB and symbol with SSB ID
    pub fn set_ssb(&mut self, prb: usize, symbol: usize, ssb_id: usize) {
        if prb < self.grid.len() && symbol < SYMBOLS_PER_SLOT {
            self.grid[prb][symbol] = ResourceType::Ssb;
            self.ssb_ids[prb][symbol] = Some(ssb_id);
            self.event_indices[prb][symbol] = None;
        }
    }

    /// Set a range of resources (start:length format)
    pub fn set_range(
        &mut self,
        prb_start: usize,
        prb_length: usize,
        symbol_start: usize,
        symbol_length: usize,
        resource_type: ResourceType,
        event_idx: Option<usize>,
    ) {
        let prb_end = (prb_start + prb_length).min(self.grid.len());
        let symbol_end = (symbol_start + symbol_length).min(SYMBOLS_PER_SLOT);

        for prb in prb_start..prb_end {
            for symbol in symbol_start..symbol_end {
                self.grid[prb][symbol] = resource_type;
                self.event_indices[prb][symbol] = event_idx;
            }
        }
    }

    /// Clear all resources in this slot
    pub fn clear(&mut self) {
        for prb in &mut self.grid {
            for symbol in prb.iter_mut() {
                *symbol = ResourceType::Empty;
            }
        }
        for prb in &mut self.event_indices {
            for idx in prb.iter_mut() {
                *idx = None;
            }
        }
        for prb in &mut self.ssb_ids {
            for id in prb.iter_mut() {
                *id = None;
            }
        }
    }
}

/// Cached PHY event with its trace index
#[derive(Debug, Clone)]
struct CachedPHYEvent {
    /// Index in the events vector
    trace_index: usize,
    /// Hyper frame number
    hfn: u32,
    /// Parsed PHY information
    phy_info: PHYInfos,
}

/// Manual SSB paint configuration parsed from metadata header
#[derive(Debug, Clone)]
struct SsbConfig {
    /// SSB ID (to differentiate multiple SSBs)
    id: usize,
    /// SSB periodicity in milliseconds
    period_ms: usize,
    /// PRB start (inclusive)
    prb_start: usize,
    /// PRB length
    prb_length: usize,
    /// Symbol start (inclusive)
    symbol_start: usize,
    /// Symbol length
    symbol_length: usize,
}

impl Default for SsbConfig {
    fn default() -> Self {
        Self {
            id: 0,
            period_ms: 20,
            prb_start: 15,
            prb_length: 20,
            symbol_start: 2,
            symbol_length: 1,
        }
    }
}

/// Resource Blocks Panel
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ResourceBlocks {
    /// Number of PRBs to display
    #[serde(skip)]
    num_prbs: usize,

    /// Number of slots per subframe (configurable)
    #[serde(skip)]
    slots_per_subframe: u8,

    /// Start position as global slot index (frame * slots_per_frame + slot_in_frame)
    #[serde(skip)]
    start_slot: usize,

    /// Window size in frames
    #[serde(skip)]
    window_frames: u8,

    /// Slot grids for the current window
    #[serde(skip)]
    slots: Vec<SlotGrid>,

    /// Cached PHY events (PDSCH/PUSCH only)
    #[serde(skip)]
    phy_events: Vec<CachedPHYEvent>,

    /// Parsed manual SSB paint configurations from metadata (supports multiple SSBs)
    #[serde(skip)]
    ssb_configs: Vec<SsbConfig>,

    /// Cell size for rendering
    cell_size: f32,

    /// lower limit of received events (HFN.number.frame.slot)
    #[serde(skip)]
    start_limit: (u32, u16, u8),

    /// upper limit of received events (HFN.number.frame.slot)
    #[serde(skip)]
    end_limit: (u32, u16, u8),

    /// Currently focused event trace index (for highlight)
    #[serde(skip)]
    focused_trace_index: Option<usize>,

    /// PHY info of the currently focused event (used to recompute scroll target
    /// on rebuild / view mode switch, and to highlight headers/labels)
    #[serde(skip)]
    focused_phy_info: Option<PHYInfos>,

    /// Set when the view needs to auto-scroll to bring the focused event into view
    #[serde(skip)]
    scroll_pending: bool,

    /// Flag indicating grid needs rebuild
    #[serde(skip)]
    needs_rebuild: bool,

    /// Current view mode (symbol or slot level)
    view_mode: ViewMode,
}

impl Default for ResourceBlocks {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceBlocks {
    /// Create a new Resource Blocks panel
    pub fn new() -> Self {
        Self {
            num_prbs: DEFAULT_NUM_PRBS,
            slots_per_subframe: DEFAULT_SLOTS_PER_SUBFRAME,
            start_slot: 0,
            window_frames: DEFAULT_WINDOW_FRAMES,
            slots: Vec::new(),
            phy_events: Vec::new(),
            ssb_configs: Vec::new(),
            cell_size: 10.0,
            start_limit: (0, 0, 0),
            end_limit: (0, 0, 0),
            focused_trace_index: None,
            focused_phy_info: None,
            scroll_pending: false,
            needs_rebuild: true,
            view_mode: ViewMode::default(),
        }
    }

    /// Frame per hyper frame
    fn frame_per_hfn(&self) -> usize {
        DEFAULT_FRAME_PER_HFN
    }

    /// Number of slots per frame ()
    fn slots_per_frame(&self) -> usize {
        self.slots_per_subframe as usize * DEFAULT_SUBFRAME_PER_FRAME
    }

    /// Total number of slots in the display window
    fn window_total_slots(&self) -> usize {
        self.window_frames as usize * self.slots_per_frame()
    }

    /// Get the frame number for the start of the window
    fn start_frame(&self) -> u16 {
        (self.start_slot / self.slots_per_frame() % self.frame_per_hfn()) as u16
    }

    /// Get the frame number for the end of the window (exclusive)
    fn end_frame(&self) -> u16 {
        let end_slot = self.start_slot + self.window_total_slots();
        (end_slot.div_ceil(self.slots_per_frame()) % self.frame_per_hfn()) as u16
    }

    /// Navigate by frames
    pub fn navigate_frame(&mut self, delta_frames: i16) {
        let shift = delta_frames as isize * self.slots_per_frame() as isize;
        let new_start = (self.start_slot as isize + shift).max(0) as usize;
        if new_start != self.start_slot {
            self.start_slot = new_start;
            self.needs_rebuild = true;
        }
    }

    /// Navigate by subframes
    pub fn navigate_subframe(&mut self, delta_subframes: i16) {
        let shift = delta_subframes as isize * self.slots_per_subframe as isize;
        let new_start = (self.start_slot as isize + shift).max(0) as usize;
        if new_start != self.start_slot {
            self.start_slot = new_start;
            self.needs_rebuild = true;
        }
    }

    /// Rebuild the entire grid from scratch
    fn rebuild_grid(&mut self) {
        let frame_per_hfn = self.frame_per_hfn();
        let spf = self.slots_per_frame();
        let total_slots = self.window_total_slots();

        self.slots.clear();
        self.slots.reserve(total_slots);

        for i in 0..total_slots {
            let global_slot: usize = self.start_slot + i;
            let hfn: u32 = (global_slot / spf / frame_per_hfn) as u32;
            let frame: u16 = (global_slot / spf % frame_per_hfn) as u16;
            let slot_in_frame: u8 = (global_slot % spf) as u8;
            self.slots.push(SlotGrid::new(hfn, frame, slot_in_frame, self.num_prbs));
        }

        self.populate_grid_from_cache();
        self.paint_manual_ssb();
        self.needs_rebuild = false;
    }

    /// Parse `key=value` from a metadata/header line
    fn parse_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
        let start = line.find(key)? + key.len();
        let rest = &line[start..];
        let end = rest.find(' ').unwrap_or(rest.len());
        Some(&rest[..end])
    }

    /// Parse SSB configuration from metadata line, e.g.:
    /// `# SSB: id=0 arfcn=630336 mu=1 L=8 period=20 offset=0 k_ssb=20 prb=15:21 symb=2`
    fn parse_ssb_config(ssb_info: &str) -> Option<SsbConfig> {
        let id = Self::parse_value(ssb_info, "id=")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let period_ms = Self::parse_value(ssb_info, "period=")?.parse::<usize>().ok()?;

        let prb_raw = Self::parse_value(ssb_info, "prb=")?;
        let (a, b) = prb_raw.split_once(':')?;
        let prb_start = a.parse::<usize>().ok()?;
        let prb_length = b.parse::<usize>().ok()?;

        let symbol_start = Self::parse_value(ssb_info, "symb=")?.parse::<usize>().ok()?;

        Some(SsbConfig {
            id,
            period_ms,
            prb_start,
            prb_length,
            symbol_start,
            symbol_length: 1,
        })
    }

    /// Update cached SSB configs from file metadata (multiple SSBs)
    fn update_ssb_config_from_metadata(&mut self, ssb_lines: &[String]) {
        self.ssb_configs.clear();
        for line in ssb_lines {
            if let Some(cfg) = Self::parse_ssb_config(line) {
                self.ssb_configs.push(cfg);
            }
        }
        // If no valid SSB configs parsed, add a default one
        if self.ssb_configs.is_empty() {
            self.ssb_configs.push(SsbConfig::default());
        }
        self.needs_rebuild = true;
    }

    /// Paint all SSBs manually based on parsed antenna/header params
    fn paint_manual_ssb(&mut self) {
        if self.slots_per_subframe == 0 || self.ssb_configs.is_empty() {
            return;
        }

        for cfg in &self.ssb_configs {
            // 1 subframe = 1 ms => slots/ms = slots_per_subframe
            let period_slots = cfg.period_ms.saturating_mul(self.slots_per_subframe as usize);
            if period_slots == 0 {
                continue;
            }

            let prb_start = cfg.prb_start.min(self.num_prbs.saturating_sub(1));
            let prb_end = (cfg.prb_start + cfg.prb_length).min(self.num_prbs);
            let symbol_start = cfg.symbol_start;
            let symbol_end = (cfg.symbol_start + cfg.symbol_length).min(SYMBOLS_PER_SLOT);

            if prb_end <= prb_start || symbol_end <= symbol_start {
                continue;
            }

            for (slot_idx, slot) in self.slots.iter_mut().enumerate() {
                let global_slot = self.start_slot + slot_idx;

                // SSB repeats every period_slots
                if !global_slot.is_multiple_of(period_slots) {
                    continue;
                }

                // Paint the SSB resource block range
                for prb in prb_start..prb_end {
                    for symbol in symbol_start..symbol_end {
                        // Store SSB ID directly in ssb_ids field
                        slot.set_ssb(prb, symbol, cfg.id);
                    }
                }
            }
        }
    }

    /// Populate the grid from cached PHY events
    fn populate_grid_from_cache(&mut self) {
        let spf = self.slots_per_frame();
        let frame_per_hfn = self.frame_per_hfn();

        if self.slots.is_empty() {
            return;
        }

        let end_slot = self.start_slot + self.slots.len();

        for event in &self.phy_events {
            let phy = &event.phy_info;

            // Calculate the global slot index for this event
            let event_global_slot = (event.hfn as usize * frame_per_hfn + phy.frame as usize) * spf + phy.slot as usize;

            // Check if in our window
            if event_global_slot < self.start_slot || event_global_slot >= end_slot {
                continue;
            }

            let slot_idx = event_global_slot - self.start_slot;
            let slot = &mut self.slots[slot_idx];

            // Map PHY channel to resource type
            let resource_type = match phy.channel_type {
                tramex_tools::interface::parser::parser_phy::PHYChannelType::PDSCH => ResourceType::Pdsch,
                tramex_tools::interface::parser::parser_phy::PHYChannelType::PUSCH => ResourceType::Pusch,
                tramex_tools::interface::parser::parser_phy::PHYChannelType::PUCCH => ResourceType::Pucch,
                tramex_tools::interface::parser::parser_phy::PHYChannelType::PRACH => ResourceType::Prach,
                _ => continue,
            };

            slot.set_range(
                phy.prb_start as usize,
                phy.prb_length as usize,
                phy.symb_start as usize,
                phy.symb_length as usize,
                resource_type,
                Some(event.trace_index),
            );
        }
    }

    /// Compute the absolute global slot index of the currently focused PHY event, if any
    fn focused_global_slot(&self) -> Option<usize> {
        let phy = self.focused_phy_info.as_ref()?;
        let spf = self.slots_per_frame();
        let frame_per_hfn = self.frame_per_hfn();
        let hfn = self
            .phy_events
            .iter()
            .find(|e| e.phy_info.frame == phy.frame && e.phy_info.slot == phy.slot)
            .map(|e| e.hfn)
            .unwrap_or(self.end_limit.0);
        Some((hfn as usize * frame_per_hfn + phy.frame as usize) * spf + phy.slot as usize)
    }

    /// Check if a slot is within the received event limits
    fn is_slot_within_limits(&self, hfn: u32, frame: u16, slot: u8) -> bool {
        // If limits not initialized (still at (0,0)), consider all slots valid
        if self.start_limit == (0, 0, 0) && self.end_limit == (0, 0, 0) {
            return true;
        }
        if (hfn, frame, slot) < self.start_limit || (hfn, frame, slot) > self.end_limit {
            return false;
        }

        true
    }

    /// Get the aggregated resource type for a PRB in slot view mode
    /// Returns the resource type with highest priority found in any symbol of that PRB
    fn _get_slot_resource_type(&self, slot_idx: usize, prb: usize) -> ResourceType {
        if let Some(slot) = self.slots.get(slot_idx)
            && prb < slot.grid.len()
        {
            let mut best_type = ResourceType::Empty;
            let mut best_priority = 0u8;

            for symbol in 0..SYMBOLS_PER_SLOT {
                let rt = slot.grid[prb][symbol];
                let priority = rt.priority();
                if priority > best_priority {
                    best_priority = priority;
                    best_type = rt;
                }
            }

            return best_type;
        }
        ResourceType::Empty
    }

    /// Get resource summary for slot view tooltip
    /// Returns list of (resource_type, start_symbol, end_symbol) for contiguous ranges
    fn get_slot_resource_summary(&self, slot_idx: usize, prb: usize) -> Vec<(ResourceType, usize, usize)> {
        let mut result = Vec::new();

        if let Some(slot) = self.slots.get(slot_idx) {
            if prb >= slot.grid.len() {
                return result;
            }

            let mut current_type = slot.grid[prb][0];
            let mut start_symbol = 0;

            for symbol in 1..SYMBOLS_PER_SLOT {
                let rt = slot.grid[prb][symbol];
                if rt != current_type {
                    if current_type != ResourceType::Empty {
                        result.push((current_type, start_symbol, symbol - 1));
                    }
                    current_type = rt;
                    start_symbol = symbol;
                }
            }

            // Don't forget the last range
            if current_type != ResourceType::Empty {
                result.push((current_type, start_symbol, SYMBOLS_PER_SLOT - 1));
            }
        }

        result
    }

    /// Draw the resource grid
    ///
    /// # Panics
    ///
    /// Should not panic since the config should be here
    fn draw_grid(&mut self, ui: &mut egui::Ui) {
        let theme = ThemeColors::get(ui);

        // Rebuild grid if needed
        if self.needs_rebuild {
            self.rebuild_grid();
        }

        // Control bar at the top
        ui.horizontal(|ui| {
            ui.label("Cell size:");
            ui.add(egui::Slider::new(&mut self.cell_size, 6.0..=24.0).show_value(false));

            ui.separator();

            // View mode toggle
            let mode_text = match self.view_mode {
                ViewMode::Symbol => "Symbol",
                ViewMode::Slot => "Slot",
            };
            if ui.button(mode_text).clicked() {
                self.view_mode = match self.view_mode {
                    ViewMode::Symbol => ViewMode::Slot,
                    ViewMode::Slot => ViewMode::Symbol,
                };
                if self.focused_phy_info.is_some() {
                    self.scroll_pending = true;
                }
            }

            ui.separator();

            // Navigation buttons
            if ui.button("◀◀").clicked() {
                self.navigate_frame(-1);
            }
            if ui.button("◀").clicked() {
                self.navigate_subframe(-1);
            }

            // Display current frame range (wrapped to 0-1023)
            let sf: u16 = self.start_frame() % 1024;
            let ef: u16 = (self.end_frame().saturating_sub(1)) % 1024;
            ui.label(format!("Frames {}-{}", sf, ef));

            if ui.button("▶").clicked() {
                self.navigate_subframe(1);
            }
            if ui.button("▶▶").clicked() {
                self.navigate_frame(1);
            }

            ui.separator();

            // Resource type legend
            let is_dark = ui.visuals().dark_mode;
            for (label, rt) in [
                ("PDCCH", ResourceType::Pdcch),
                ("PDSCH", ResourceType::Pdsch),
                ("PUSCH", ResourceType::Pusch),
                ("PUCCH", ResourceType::Pucch),
                ("PRACH", ResourceType::Prach),
                ("SSB", ResourceType::Ssb),
            ] {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, rt.color(is_dark));
                ui.label(egui::RichText::new(label).small());
            }
        });

        ui.separator();

        // Dimensions
        let label_width = 40.0_f32;
        let header_height = 30.0_f32;
        let cell_size = self.cell_size;
        let slot_width = match self.view_mode {
            ViewMode::Symbol => SYMBOLS_PER_SLOT as f32 * cell_size,
            ViewMode::Slot => cell_size,
        };

        // Border widths for visual separation (no gaps)
        let sm_border = 0.5_f32; // Between symbols
        let md_border = 1.0_f32; // Between slots
        let lg_border = 2.0_f32; // Between subframes
        let sps = self.slots_per_subframe as usize;
        let spf = self.slots_per_frame();
        let start_slot = self.start_slot;

        // Helper to get X position - no gaps, simple calculation
        let get_slot_rel_x = move |i: usize| -> f32 { i as f32 * slot_width };

        // Compute the focused event's global slot / PRB range once, used both for
        // auto-scrolling and for highlighting the corresponding header/label.
        let focused_global_slot = self.focused_global_slot();
        let focused_prb_range: Option<(usize, usize)> = self.focused_phy_info.as_ref().map(|phy| {
            let start = phy.prb_start as usize;
            let end = start + (phy.prb_length as usize).max(1) - 1;
            (start, end)
        });

        // Single ScrollArea for the entire grid.
        // Headers and labels are drawn as overlays pinned to the viewport edges.
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            // Calculate total width: position of the hypothetic next slot after the last one
            let total_grid_width = get_slot_rel_x(self.slots.len());
            // But strictly speaking, the last slot doesn't need a gap after it, just its width.
            // Using get_slot_rel_x(len) effectively adds a slot_gap (and maybe extra_gap) at the end. This is fine.

            let total_width = label_width + total_grid_width;
            let total_height = header_height + (self.num_prbs as f32 * cell_size);

            let (content_rect, _) = ui.allocate_exact_size(Vec2::new(total_width, total_height), egui::Sense::hover());

            // Auto-scroll to bring the focused event into view (once per focus change
            // or view-mode switch, so it also works before any manual scroll happens).
            if self.scroll_pending {
                if let Some(global_slot) = focused_global_slot
                    && let Some(slot_idx) = global_slot.checked_sub(self.start_slot)
                    && slot_idx < self.slots.len()
                {
                    let slot_rel_x = get_slot_rel_x(slot_idx);
                    let slot_x = content_rect.left() + label_width + slot_rel_x;
                    let (y_top, y_height) = match focused_prb_range {
                        Some((prb_start, prb_end)) => {
                            let top = content_rect.top() + header_height + prb_start as f32 * cell_size;
                            let height = ((prb_end - prb_start + 1) as f32 * cell_size).max(cell_size);
                            (top, height)
                        }
                        None => (content_rect.top() + header_height, self.num_prbs as f32 * cell_size),
                    };
                    let target_rect = Rect::from_min_size(Pos2::new(slot_x, y_top), Vec2::new(slot_width, y_height));
                    ui.scroll_to_rect(target_rect, Some(egui::Align::Center));
                }
                self.scroll_pending = false;
            }

            let painter = ui.painter();
            let visible_rect = ui.clip_rect();
            let panel_bg = ui.visuals().panel_fill;

            // --- Calculate visible ranges for culling ---
            let rel_x = visible_rect.min.x - content_rect.min.x;
            let rel_y = visible_rect.min.y - content_rect.min.y;

            // Simple pitch calculation (no gaps)
            let avg_pitch = slot_width;
            let safety_margin = sps.max(1); // At least one slot

            // Estimate start/end with safety margin
            let est_start = ((rel_x - label_width).max(0.0) / avg_pitch).floor() as usize;
            let vis_start_slot = est_start.saturating_sub(safety_margin);

            let visible_slots_count = (visible_rect.width() / avg_pitch).ceil() as usize;
            let vis_end_slot = (est_start + visible_slots_count + safety_margin).min(self.slots.len());

            let vis_start_prb = ((rel_y - header_height).max(0.0) / cell_size).floor() as usize;
            let vis_end_prb =
                (((rel_y + visible_rect.height() - header_height) / cell_size).ceil() as usize).min(self.num_prbs);

            // --- 1. GRID CELLS (scrolls both ways) ---
            // Track hovered cell for tooltip
            let mut hovered_event_idx: Option<usize> = None;
            let mut hovered_ssb_id: Option<usize> = None;
            let mut hovered_slot_idx: Option<usize> = None;
            let mut hovered_prb: Option<usize> = None;
            let pointer_pos = ui.ctx().pointer_hover_pos();

            match self.view_mode {
                ViewMode::Symbol => {
                    // Symbol-level view: 1 square = 1 PRB x 1 symbol
                    let grid_origin_y = content_rect.top() + header_height; // Hoist calculation
                    for slot_idx in vis_start_slot..vis_end_slot {
                        if let Some(slot) = self.slots.get(slot_idx) {
                            let slot_rel_x = get_slot_rel_x(slot_idx);
                            let slot_x = content_rect.left() + label_width + slot_rel_x;
                            // Compute once per slot, not per cell
                            let within_limits = self.is_slot_within_limits(slot.hfn, slot.frame_number, slot.slot_number);

                            for prb in vis_start_prb..vis_end_prb {
                                let y = grid_origin_y + (prb as f32 * cell_size);
                                for symbol in 0..SYMBOLS_PER_SLOT {
                                    let x = slot_x + (symbol as f32 * cell_size);
                                    let cell_rect =
                                        Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_size - 1.0, cell_size - 1.0));

                                    let resource_type = slot.grid[prb][symbol];
                                    let color = if within_limits {
                                        resource_type.color(ui.visuals().dark_mode)
                                    } else {
                                        Color32::from_gray(180)
                                    };
                                    painter.rect_filled(cell_rect, 0.0, color);
                                    painter.rect_stroke(
                                        cell_rect,
                                        0.0,
                                        Stroke::new(sm_border, theme.line_small),
                                        STROKE_KIND_STYLE,
                                    );

                                    // Highlight focused event cells
                                    if let Some(focused_idx) = self.focused_trace_index
                                        && slot.event_indices[prb][symbol] == Some(focused_idx)
                                    {
                                        painter.rect_stroke(
                                            cell_rect,
                                            0.0,
                                            Stroke::new(1.5, theme.text_strong),
                                            STROKE_KIND_STYLE,
                                        );
                                    }

                                    if resource_type == ResourceType::Dmrs {
                                        painter.circle_filled(
                                            cell_rect.center(),
                                            cell_size / 4.0,
                                            Color32::from_rgb(180, 30, 30),
                                        );
                                    }

                                    // Check hover
                                    if let Some(pos) = pointer_pos
                                        && cell_rect.contains(pos)
                                    {
                                        hovered_event_idx = slot.event_indices[prb][symbol];
                                        hovered_ssb_id = slot.ssb_ids[prb][symbol];
                                        hovered_slot_idx = Some(slot_idx);
                                        hovered_prb = Some(prb);
                                    }
                                }
                            }

                            // Draw vertical line at LEFT edge of slot (border between slots)
                            // Vary width based on subframe boundary
                            let global_slot = start_slot + slot_idx;
                            let is_subframe_boundary = global_slot.is_multiple_of(sps);
                            let border_width = if is_subframe_boundary { lg_border } else { md_border };

                            let grid_bottom = grid_origin_y + self.num_prbs as f32 * cell_size;

                            // Left edge of this slot
                            painter.line_segment(
                                [Pos2::new(slot_x, grid_origin_y), Pos2::new(slot_x, grid_bottom)],
                                Stroke::new(
                                    border_width,
                                    if is_subframe_boundary {
                                        theme.line_strong
                                    } else {
                                        theme.line_medium
                                    },
                                ),
                            );

                            // Draw symbol separators (thin vertical lines within slot)
                            for symbol in 1..SYMBOLS_PER_SLOT {
                                let x = slot_x + (symbol as f32 * cell_size);
                                painter.line_segment(
                                    [Pos2::new(x, grid_origin_y), Pos2::new(x, grid_bottom)],
                                    Stroke::new(sm_border, theme.line_small),
                                );
                            }
                        }
                    }
                }
                ViewMode::Slot => {
                    // Slot-level view: 1 square = 1 PRB x 1 slot (14 symbols aggregated)
                    let grid_origin_y = content_rect.top() + header_height; // Hoist calculation
                    for slot_idx in vis_start_slot..vis_end_slot {
                        // Cache slot reference once per slot iteration
                        let slot = match self.slots.get(slot_idx) {
                            Some(s) => s,
                            None => continue,
                        };
                        let slot_rel_x = get_slot_rel_x(slot_idx);
                        let slot_x = content_rect.left() + label_width + slot_rel_x;
                        // Compute once per slot, not per PRB
                        let within_limits = self.is_slot_within_limits(slot.hfn, slot.frame_number, slot.slot_number);

                        for prb in vis_start_prb..vis_end_prb {
                            let y = grid_origin_y + (prb as f32 * cell_size);
                            let cell_rect =
                                Rect::from_min_size(Pos2::new(slot_x, y), Vec2::new(cell_size - 1.0, cell_size - 1.0));

                            // Inline get_slot_resource_type: find best resource type with early exit on highest priority
                            let mut best_type = ResourceType::Empty;
                            let mut best_priority = 0u8;
                            for symbol in 0..SYMBOLS_PER_SLOT {
                                let rt = slot.grid[prb][symbol];
                                let priority = rt.priority();
                                if priority > best_priority {
                                    best_priority = priority;
                                    best_type = rt;
                                    // Early exit if we hit the max possible priority
                                    if priority >= 100 {
                                        // Adjust if you have higher priorities
                                        break;
                                    }
                                }
                            }

                            let color = if within_limits {
                                best_type.color(ui.visuals().dark_mode)
                            } else {
                                Color32::from_gray(180)
                            };
                            painter.rect_filled(cell_rect, 0.0, color);
                            painter.rect_stroke(cell_rect, 0.0, Stroke::new(sm_border, theme.line_small), STROKE_KIND_STYLE);

                            // Highlight focused event cells - single pass through symbols
                            if let Some(focused_idx) = self.focused_trace_index
                                && prb < slot.grid.len()
                            {
                                let mut has_focused = false;
                                for symbol in 0..SYMBOLS_PER_SLOT {
                                    if slot.event_indices[prb][symbol] == Some(focused_idx) {
                                        has_focused = true;
                                        break;
                                    }
                                }
                                if has_focused {
                                    painter.rect_stroke(
                                        cell_rect,
                                        0.0,
                                        Stroke::new(1.5, theme.text_strong),
                                        STROKE_KIND_STYLE,
                                    );
                                }
                            }

                            // Check hover - single pass for both SSB and events
                            if let Some(pos) = pointer_pos
                                && cell_rect.contains(pos)
                            {
                                hovered_slot_idx = Some(slot_idx);
                                hovered_prb = Some(prb);
                                if prb < slot.grid.len() {
                                    // Check SSB first, then events - single pass
                                    for symbol in 0..SYMBOLS_PER_SLOT {
                                        if slot.grid[prb][symbol] == ResourceType::Ssb {
                                            hovered_ssb_id = slot.ssb_ids[prb][symbol];
                                            break;
                                        }
                                        if let Some(idx) = slot.event_indices[prb][symbol] {
                                            hovered_event_idx = Some(idx);
                                            // Continue checking for SSB which has higher priority
                                        }
                                    }
                                }
                            }
                        }

                        // Draw vertical line at LEFT edge of slot
                        // Vary width based on frame boundary
                        let global_slot = start_slot + slot_idx;
                        let is_frame_boundary = global_slot.is_multiple_of(spf);
                        let border_width = if is_frame_boundary { lg_border } else { sm_border };

                        let grid_bottom = grid_origin_y + self.num_prbs as f32 * cell_size;

                        painter.line_segment(
                            [Pos2::new(slot_x, grid_origin_y), Pos2::new(slot_x, grid_bottom)],
                            Stroke::new(
                                border_width,
                                if is_frame_boundary {
                                    theme.line_strong
                                } else {
                                    theme.line_medium
                                },
                            ),
                        );
                    }
                }
            }

            // Show tooltip for hovered cell
            if let Some(event_idx) = hovered_event_idx {
                if let Some(event) = self.phy_events.iter().find(|e| e.trace_index == event_idx) {
                    let phy = &event.phy_info;
                    // Use egui's show_tooltip_at_pointer directly without pre-building string
                    egui::Tooltip::always_open(ui.ctx().clone(), ui.layer_id(), egui::Id::new("rb_tooltip"), TOOLTIP_ANCHOR)
                        .at_pointer()
                        .show(|ui| {
                            let channel_name = match phy.channel_type {
                                tramex_tools::interface::parser::parser_phy::PHYChannelType::PDSCH => "PDSCH (DL Data)",
                                tramex_tools::interface::parser::parser_phy::PHYChannelType::PUSCH => "PUSCH (UL Data)",
                                tramex_tools::interface::parser::parser_phy::PHYChannelType::PUCCH => "PUCCH (UL Control)",
                                tramex_tools::interface::parser::parser_phy::PHYChannelType::PRACH => {
                                    "PRACH (Random Access)"
                                }
                                _ => "Unknown",
                            };

                            if self.view_mode == ViewMode::Symbol {
                                ui.label(format!(
                                    "{}\nFrame {} | SubFrame {} | Slot {}\nPRB: {}-{} \nSymbols: {}-{}",
                                    channel_name,
                                    phy.frame,
                                    phy.slot / self.slots_per_subframe,
                                    phy.slot % self.slots_per_subframe,
                                    phy.prb_start,
                                    phy.prb_start + phy.prb_length - 1,
                                    phy.symb_start,
                                    phy.symb_start + phy.symb_length - 1,
                                ));
                            } else {
                                // Slot view: show symbol range summary
                                let slot_idx = hovered_slot_idx.unwrap_or(0);
                                let prb = hovered_prb.unwrap_or(0);
                                let summary = self.get_slot_resource_summary(slot_idx, prb);
                                ui.label(format!("{} (Slot view)\nFrame {} | PRB {}", channel_name, phy.frame, prb));
                                for (rt, start, end) in summary {
                                    // Map PHY channel type to resource type for comparison
                                    let expected_rt = match phy.channel_type {
                                        tramex_tools::interface::parser::parser_phy::PHYChannelType::PDSCH => {
                                            ResourceType::Pdsch
                                        }
                                        tramex_tools::interface::parser::parser_phy::PHYChannelType::PUSCH => {
                                            ResourceType::Pusch
                                        }
                                        tramex_tools::interface::parser::parser_phy::PHYChannelType::PUCCH => {
                                            ResourceType::Pucch
                                        }
                                        tramex_tools::interface::parser::parser_phy::PHYChannelType::PRACH => {
                                            ResourceType::Prach
                                        }
                                        _ => ResourceType::Empty,
                                    };
                                    if rt == expected_rt {
                                        ui.label(format!("Symbols: {}-{}", start, end));
                                        break;
                                    }
                                }
                            }
                        });
                }
            } else if let Some(ssb_id) = hovered_ssb_id {
                let ssb_cfg = self
                    .ssb_configs
                    .iter()
                    .find(|cfg| cfg.id == ssb_id)
                    .unwrap_or_else(|| self.ssb_configs.first().unwrap());

                egui::Tooltip::always_open(ui.ctx().clone(), ui.layer_id(), egui::Id::new("rb_tooltip"), TOOLTIP_ANCHOR)
                    .at_pointer()
                    .show(|ui| {
                        if self.view_mode == ViewMode::Symbol {
                            ui.label(format!(
                                "SSB {} (Synchronization Signal Block)\nPeriod: {}ms\nPRB: {}-{} \nSymbols: {}-{}",
                                ssb_cfg.id,
                                ssb_cfg.period_ms,
                                ssb_cfg.prb_start,
                                ssb_cfg.prb_start + ssb_cfg.prb_length - 1,
                                ssb_cfg.symbol_start,
                                ssb_cfg.symbol_start + ssb_cfg.symbol_length - 1,
                            ));
                        } else {
                            // Slot view: show SSB presence in slot
                            let slot_idx = hovered_slot_idx.unwrap_or(0);
                            let prb = hovered_prb.unwrap_or(0);
                            let summary = self.get_slot_resource_summary(slot_idx, prb);
                            ui.label(format!(
                                "SSB {} (Slot view)\nPeriod: {}ms | PRB: {}-{}",
                                ssb_cfg.id,
                                ssb_cfg.period_ms,
                                ssb_cfg.prb_start,
                                ssb_cfg.prb_start + ssb_cfg.prb_length - 1
                            ));
                            for (rt, start, end) in summary {
                                if rt == ResourceType::Ssb {
                                    ui.label(format!("Symbols: {}-{}", start, end));
                                    break;
                                }
                            }
                        }
                    });
            } else if let (Some(slot_idx), Some(prb)) = (hovered_slot_idx, hovered_prb) {
                // Slot view: show summary of all resources in this PRB
                if self.view_mode == ViewMode::Slot {
                    let summary = self.get_slot_resource_summary(slot_idx, prb);
                    if !summary.is_empty() {
                        let slot = &self.slots[slot_idx];
                        egui::Tooltip::always_open(
                            ui.ctx().clone(),
                            ui.layer_id(),
                            egui::Id::new("rb_tooltip"),
                            TOOLTIP_ANCHOR,
                        )
                        .at_pointer()
                        .show(|ui| {
                            ui.label(format!("PRB {} in Slot {}", prb, slot.slot_number));
                            ui.label(format!(
                                "Frame {}.{} | {} symbols active",
                                slot.frame_number,
                                slot.slot_number / self.slots_per_subframe,
                                summary.len()
                            ));
                            for (rt, start, end) in summary {
                                let rt_name = match rt {
                                    ResourceType::Pdsch => "PDSCH",
                                    ResourceType::Pusch => "PUSCH",
                                    ResourceType::Pucch => "PUCCH",
                                    ResourceType::Prach => "PRACH",
                                    ResourceType::Pdcch => "PDCCH",
                                    ResourceType::Ssb => "SSB",
                                    ResourceType::Dmrs => "DMRS",
                                    _ => "?",
                                };
                                ui.label(format!("{}: symbols {}-{}", rt_name, start, end));
                            }
                        });
                    }
                }
            }

            // --- 2. SLOT HEADERS (fixed at top, scrolls horizontally) ---
            let header_bg = Rect::from_min_size(
                Pos2::new(visible_rect.left(), visible_rect.top()),
                Vec2::new(visible_rect.width(), header_height),
            );
            painter.rect_filled(header_bg, 0.0, panel_bg);
            painter.rect_stroke(header_bg, 0.0, Stroke::new(1.0, theme.text_weak), STROKE_KIND_STYLE);

            if self.view_mode == ViewMode::Symbol {
                for slot_idx in vis_start_slot..vis_end_slot {
                    if let Some(slot) = self.slots.get(slot_idx) {
                        let slot_rel_x = get_slot_rel_x(slot_idx);
                        let slot_x = content_rect.left() + label_width + slot_rel_x;

                        let sps_u8 = self.slots_per_subframe;
                        let slot_in_sf = slot.slot_number % sps_u8;
                        let subframe = slot.slot_number / sps_u8;

                        // Highlight the header of the slot containing the focused event
                        let is_focused_slot = focused_global_slot == Some(start_slot + slot_idx);
                        if is_focused_slot {
                            let hl_rect = Rect::from_min_size(
                                Pos2::new(slot_x, visible_rect.top()),
                                Vec2::new(slot_width, header_height),
                            );
                            let fill =
                                Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 70);
                            painter.rect_filled(hl_rect, 0.0, fill);
                            painter.rect_stroke(hl_rect, 0.0, Stroke::new(1.5, theme.accent), STROKE_KIND_STYLE);
                        }

                        // Frame.subframe on top line (wrap frame to 0-1023)
                        let display_frame = slot.frame_number % 1024;
                        let frame_text = format!("{}.{}", display_frame, subframe);
                        painter.text(
                            Pos2::new(slot_x + slot_width / 2.0, visible_rect.top() + 10.0),
                            egui::Align2::CENTER_CENTER,
                            frame_text,
                            egui::FontId::proportional(10.0),
                            if is_focused_slot { theme.text_strong } else { theme.text },
                        );

                        // Slot index below
                        let slot_text = format!("Slot {}", slot_in_sf);
                        painter.text(
                            Pos2::new(slot_x + slot_width / 2.0, visible_rect.top() + 22.0),
                            egui::Align2::CENTER_CENTER,
                            slot_text,
                            egui::FontId::proportional(9.0),
                            if is_focused_slot { theme.text_strong } else { theme.text_weak },
                        );
                    }
                }
            } else {
                // Highlight the header column of the slot containing the focused event
                if let Some(g) = focused_global_slot
                    && let Some(slot_idx) = g.checked_sub(start_slot)
                    && slot_idx >= vis_start_slot
                    && slot_idx < vis_end_slot
                {
                    let slot_rel_x = get_slot_rel_x(slot_idx);
                    let slot_x = content_rect.left() + label_width + slot_rel_x;
                    let hl_rect =
                        Rect::from_min_size(Pos2::new(slot_x, visible_rect.top()), Vec2::new(slot_width, header_height));
                    let fill = Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 70);
                    painter.rect_filled(hl_rect, 0.0, fill);
                    painter.rect_stroke(hl_rect, 0.0, Stroke::new(1.5, theme.accent), STROKE_KIND_STYLE);
                }

                // Slot view: show frame number in middle of each frame
                let first_visible_frame = (start_slot + vis_start_slot) / spf;
                let last_visible_frame = (start_slot + vis_end_slot.saturating_sub(1)) / spf;

                for frame in first_visible_frame..=last_visible_frame {
                    // Find middle slot of this frame
                    let middle_slot_global = frame * spf + spf / 2;

                    // Check if middle slot is in visible range
                    if middle_slot_global >= start_slot + vis_start_slot && middle_slot_global < start_slot + vis_end_slot {
                        let display_frame = frame % self.frame_per_hfn();
                        let slot_idx = middle_slot_global - start_slot;
                        let slot_rel_x = get_slot_rel_x(slot_idx);
                        let slot_x = content_rect.left() + label_width + slot_rel_x;

                        painter.text(
                            Pos2::new(slot_x + slot_width / 2.0, visible_rect.top() + 15.0),
                            egui::Align2::CENTER_CENTER,
                            display_frame.to_string(),
                            egui::FontId::proportional(10.0),
                            theme.text,
                        );
                    }
                }
            }
            // --- 3. PRB LABELS (fixed at left, scrolls vertically) ---
            let label_bg = Rect::from_min_size(
                Pos2::new(visible_rect.left(), visible_rect.top()),
                Vec2::new(label_width, visible_rect.height()),
            );
            painter.rect_filled(label_bg, 0.0, panel_bg);
            painter.rect_stroke(label_bg, 0.0, Stroke::new(1.0, theme.text_weak), STROKE_KIND_STYLE);

            let prb_label_x = visible_rect.left() + label_width / 2.0;
            let prb_origin_y = content_rect.top() + header_height + cell_size / 2.0;
            for prb in vis_start_prb..vis_end_prb {
                let y = prb_origin_y + (prb as f32 * cell_size);
                let is_focused_prb = focused_prb_range.is_some_and(|(s, e)| prb >= s && prb <= e);
                if is_focused_prb {
                    let hl_rect = Rect::from_min_size(
                        Pos2::new(visible_rect.left(), y - cell_size / 2.0),
                        Vec2::new(label_width, cell_size),
                    );
                    let fill = Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 70);
                    painter.rect_filled(hl_rect, 0.0, fill);
                }
                painter.text(
                    Pos2::new(prb_label_x, y),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", prb),
                    egui::FontId::proportional(9.0),
                    if is_focused_prb { theme.text_strong } else { theme.text_weak },
                );
            }

            // --- 4. CORNER (fixed top-left) ---
            let corner = Rect::from_min_size(
                Pos2::new(visible_rect.left(), visible_rect.top()),
                Vec2::new(label_width, header_height),
            );
            painter.rect_filled(corner, 0.0, panel_bg);
            painter.rect_stroke(corner, 0.0, Stroke::new(1.0, theme.text_weak), STROKE_KIND_STYLE);
            painter.text(
                corner.center(),
                egui::Align2::CENTER_CENTER,
                "PRB",
                egui::FontId::proportional(10.0),
                theme.text,
            );
        });
    }

    /// Add a PHY event to the cache
    fn add_phy_event(&mut self, trace_index: usize, phy_info: PHYInfos) {
        let mut hfn = self.end_limit.0; // hyper frame number
        let frame = phy_info.frame;
        let slot = phy_info.slot;
        let spf = self.slots_per_frame();
        let frame_per_hfn = self.frame_per_hfn();

        // Update limits
        if self.start_limit == (0, 0, 0) {
            // First event ever
            self.start_limit = (0, frame, slot);
            self.end_limit = (0, frame, slot);
        }
        // Determine if this event extends our range forward or backward
        // Compare using current HFN (will adjust if needed)
        let candidate_pos = (frame, slot);

        if candidate_pos > (self.end_limit.1, self.end_limit.2) {
            // Event extends range backward (old event from earlier HFN)
            // Check for anti-wrap
            let end_frame = self.end_limit.1;
            if frame > end_frame && (frame - end_frame) > 512 {
                // This frame is much larger than start but position is earlier
                // Means this is from a previous HFN
                hfn = hfn.saturating_sub(1);
            } else {
                self.end_limit = (hfn, frame, slot);
            }
        } else if candidate_pos < (self.end_limit.1, self.end_limit.2) {
            // Event extends the range forward
            // Check for wrap
            let end_frame = self.end_limit.1;
            if frame < end_frame && (end_frame - frame) > 512 {
                // Frame wrapped around (e.g., 1023 -> 0), increment HFN
                hfn += 1;
                self.end_limit = (hfn, frame, slot);
            }
        }

        // println!("Updated limits: {:?}, {:?}", self.start_limit, self.end_limit);

        // Check if we already have this event (by trace_index)
        // if let Some(existing) = self.phy_events.iter_mut().find(|e| e.trace_index == trace_index) {
        //     existing.phy_info = phy_info;
        // } else {
        self.phy_events.push(CachedPHYEvent {
            trace_index,
            hfn,
            phy_info,
        });
        // }

        // Check if this event affects the current window
        let event_global_slot = (hfn as usize * frame_per_hfn + frame as usize) * spf + slot as usize;
        let end_slot = self.start_slot + self.window_total_slots();
        if event_global_slot >= self.start_slot && event_global_slot < end_slot {
            self.needs_rebuild = true;
        }
    }

    /// Clear all slots
    pub fn clear_slots(&mut self) {
        self.slots.clear();
        self.phy_events.clear();
        self.start_slot = 0;
        self.needs_rebuild = true;
    }

    /// Get a mutable reference to a slot by index
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut SlotGrid> {
        self.slots.get_mut(index)
    }
}

impl EventSubscriber for ResourceBlocks {
    fn on_event_added(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // Check if this is a PHY event with PDSCH/PUSCH info
        if let AdditionalInfos::PHYInfos(phy_info) = &event.additional_infos {
            // Only cache PDSCH/PUSCH events
            if matches!(
                phy_info.channel_type,
                tramex_tools::interface::parser::parser_phy::PHYChannelType::PDSCH
                    | tramex_tools::interface::parser::parser_phy::PHYChannelType::PUSCH
                    | tramex_tools::interface::parser::parser_phy::PHYChannelType::PUCCH
                    | tramex_tools::interface::parser::parser_phy::PHYChannelType::PRACH
            ) {
                self.add_phy_event(index, phy_info.clone());
            }
        }
    }

    fn on_event_focused(&mut self, event: &Trace, index: usize, _context: &EventContext) {
        // Track focused event for highlight
        self.focused_trace_index = Some(index);

        // If focused event is a PHY trace, center the grid on it
        if let AdditionalInfos::PHYInfos(phy_info) = &event.additional_infos {
            self.focused_phy_info = Some(phy_info.clone());
            self.scroll_pending = true;

            if let Some(event_global_slot) = self.focused_global_slot() {
                let half_window = self.window_total_slots() / 2;
                let new_start = event_global_slot.saturating_sub(half_window);

                if new_start != self.start_slot {
                    self.start_slot = new_start;
                    self.needs_rebuild = true;
                }
            }
        }
    }

    fn on_events_cleared(&mut self) {
        log::debug!("ResourceBlocks: Clearing grid and PHY event cache");
        self.clear_slots();
        self.ssb_configs.clear();
        self.start_limit = (0, 0, 0);
        self.end_limit = (0, 0, 0);
        self.focused_trace_index = None;
        self.focused_phy_info = None;
        self.scroll_pending = false;
    }

    fn on_metadata_changed(&mut self, metadata: &tramex_tools::interface::parse_config::FileMetadata) {
        log::debug!("ResourceBlocks: Metadata changed, parsing SSB config");
        self.num_prbs = metadata.n_rb.map(|v| v as usize).unwrap_or(DEFAULT_NUM_PRBS);
        self.needs_rebuild = true;
        self.update_ssb_config_from_metadata(&metadata.ssb_info);
    }

    fn name(&self) -> &'static str {
        "Resource Blocks"
    }

    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), TramexError> {
        egui::Window::new("Resource Blocks")
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0)
            .open(open)
            .show(ctx, |ui| {
                self.draw_grid(ui);
            });
        Ok(())
    }
}

impl super::PanelView for ResourceBlocks {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.draw_grid(ui);
    }
}
