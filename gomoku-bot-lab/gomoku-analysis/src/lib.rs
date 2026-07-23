mod annotations;
mod bridge;
mod failure;
mod model;
mod onset;
mod replay;
mod trace;
mod types;
mod util;

pub use annotations::replay_frame_annotations_for_analysis;
pub use bridge::{analysis_options_from_json, replay_analysis_error, ReplayAnalysisStepEnvelope};
pub use model::{corridor_analysis_model, rule_label};
pub use replay::{analyze_replay, ReplayAnalysisSession};
pub use trace::{
    analyze_alternate_defender_reply_options, analyze_defender_reply_options,
    defender_reply_candidates, defender_reply_roles_for_move, visible_defender_reply_candidates,
};
pub use types::*;

#[cfg(test)]
pub(crate) use annotations::{push_lethal_onset_annotations, replay_frame_annotations_from_proof};
#[cfg(test)]
pub(crate) use failure::{failure_analysis, FailureAnalysisInput};
#[cfg(test)]
pub(crate) use onset::lethal_onset_from_threat;
#[cfg(test)]
pub(crate) use replay::replay_prefix_boards;
#[cfg(test)]
pub(crate) use trace::{
    classify_actual_corridor_reply, corridor_defender_reply_moves, corridor_proof_result,
    with_limit_causes, CorridorReplyStatus, ThreatReplySet,
};

#[cfg(test)]
mod tests;
