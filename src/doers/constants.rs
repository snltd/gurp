use crate::doers::types::ApplySummary;

pub const ONE_RESOURCE_ONE_CHANGE: ApplySummary = ApplySummary {
    resources: 1,
    changes: 1,
    errors: 0,
};

pub const ONE_RESOURCE_NOOP: ApplySummary = ApplySummary {
    resources: 1,
    changes: 0,
    errors: 0,
};

pub const ONE_RESOURCE_NO_CHANGE: ApplySummary = ApplySummary {
    resources: 1,
    changes: 0,
    errors: 0,
};

pub const ONE_RESOURCE_ONE_ERROR: ApplySummary = ApplySummary {
    resources: 1,
    changes: 0,
    errors: 1,
};

pub const NO_RESOURCES_TO_CHANGE: ApplySummary = ApplySummary {
    resources: 0,
    changes: 0,
    errors: 0,
};
