use crate::{Gray8, HitTarget, Layout, Rect};
pub use eink_ui::analysis_panel::{AnalysisMoveChip, AnalysisMoveChipSource, AnalysisView};
pub type AnalysisPanelOutput = eink_ui::analysis_panel::AnalysisPanelOutput<HitTarget>;
impl From<eink_ui::analysis_panel::AnalysisHit> for HitTarget {
    fn from(hit: eink_ui::analysis_panel::AnalysisHit) -> Self {
        match hit {
            eink_ui::analysis_panel::AnalysisHit::AnalysisMove(node) => Self::AnalysisMove(node),
            eink_ui::analysis_panel::AnalysisHit::AnalysisPreviousPage => {
                Self::AnalysisPreviousPage
            }
            eink_ui::analysis_panel::AnalysisHit::AnalysisNextPage => Self::AnalysisNextPage,
        }
    }
}
pub(crate) fn draw_analysis_panel(
    frame: &mut Gray8,
    view: AnalysisView<'_>,
    layout: Layout,
    content: Rect,
    scale: u32,
) -> Option<AnalysisPanelOutput> {
    eink_ui::analysis_panel::draw_analysis_panel(frame, view, layout.status, content, scale)
        .map(|panel| panel.map_targets())
}
