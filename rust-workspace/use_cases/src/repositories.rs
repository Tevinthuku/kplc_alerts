use crate::authentication::{
    subscriber_authentication::SubscriberResolverRepo, SubscriberAuthenticationRepo,
};

use crate::{
    import_affected_areas::SaveBlackoutAffectedAreasRepo,
    notifications::notify_subscribers::SubscriberRepo,
};

pub trait Repository:
    SubscriberAuthenticationRepo
    + SaveBlackoutAffectedAreasRepo
    + SubscriberResolverRepo
    + SubscriberRepo
    + Clone
{
}

impl<T> Repository for T where
    T: Clone
        + SubscriberAuthenticationRepo
        + SaveBlackoutAffectedAreasRepo
        + SubscriberResolverRepo
        + SubscriberRepo
{
}
