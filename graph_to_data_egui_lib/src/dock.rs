use crate::tab::Tab as DockItem;

#[derive(Default)]
pub(super) struct DockWidget {}
impl DockWidget {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, dock_state: &mut DockState) {
        if !dock_state.entries.iter_all_tabs().any(|_| true) {
            dock_state.add_new_item(DockItem::default());
        }
        let entries = &mut dock_state.entries;
        // ensure that all indices are non-None
        {
            let mut indices: Vec<_> = entries
                .iter_all_tabs()
                .filter_map(|x| x.1.index.map(|i| i.0 .0))
                .collect();
            entries
                .iter_all_tabs_mut()
                .map(|e| &mut e.1.index)
                .for_each(|index| {
                    if index.is_none() {
                        let i = (0..).find(|i| !indices.contains(i)).unwrap();
                        indices.push(i);
                        let id = ui.id().with(i);
                        *index = Some((DockIndex(i), id));
                    }
                })
        }
        // show dock area
        let mut dock_viewer = DockTabViewer {
            new_tab_requested: false,
        };
        egui_dock::DockArea::new(entries)
            .show_add_buttons(true)
            .style({
                let mut style = egui_dock::Style::from_egui(ui.ctx().style().as_ref());
                style.tab_bar.fill_tab_bar = true;
                style
            })
            .show_inside(ui, &mut dock_viewer);
        if dock_viewer.new_tab_requested {
            dock_state.add_new_item(Default::default());
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(super) struct DockState {
    entries: egui_dock::DockState<DockEntry>,
}
impl Default for DockState {
    fn default() -> Self {
        Self {
            entries: egui_dock::DockState::new(vec![]),
        }
    }
}
impl DockState {
    pub(crate) fn add_new_item(&mut self, new_item: DockItem) {
        self.entries.push_to_focused_leaf(new_item.into());
    }

    pub(crate) fn file_dropped(&mut self, file: egui::DroppedFile) {
        let mut file = file;
        for tab in self.entries.iter_all_tabs_mut() {
            match tab.1.item.file_dropped(file) {
                Ok(()) => return,
                Err(f) => file = f,
            }
        }
        self.add_new_item(DockItem::from_dropped_file(file))
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DockEntry {
    item: DockItem,
    #[serde(skip)]
    index: Option<(DockIndex, egui::Id)>,
}
impl From<DockItem> for DockEntry {
    fn from(value: DockItem) -> Self {
        Self {
            item: value,
            index: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DockIndex(usize);

struct DockTabViewer {
    new_tab_requested: bool,
}
impl egui_dock::TabViewer for DockTabViewer {
    type Tab = DockEntry;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.item.title()
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.item.show(ui)
    }
    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        tab.index
            .as_ref()
            .map(|(_, id)| *id)
            .unwrap_or(egui::Id::new(0))
    }
    fn on_add(&mut self, _surface: egui_dock::SurfaceIndex, _node: egui_dock::NodeIndex) {
        self.new_tab_requested = true;
    }
}
