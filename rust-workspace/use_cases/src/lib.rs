use authentication::subscriber_authentication::SubscriberResolverInteractorImpl;
use search_for_locations::{
    LocationSearchApi, LocationSearchInteractor, LocationSearchInteractorImpl,
};

use crate::authentication::{AuthenticationInteractor, AuthenticationInteractorImpl};
use crate::repositories::Repository;
use std::sync::Arc;

pub mod actor;
pub mod authentication;
pub mod import_affected_areas;
pub mod notifications;
mod repositories;
pub mod search_for_locations;
pub mod subscriber_locations;

pub trait App {
    fn authentication(&self) -> &dyn AuthenticationInteractor;
    fn location_searcher(&self) -> &dyn LocationSearchInteractor;
}

pub struct AppImpl {
    authentication: Arc<dyn AuthenticationInteractor>,
    location_searcher_interactor: Arc<dyn LocationSearchInteractor>,
}

impl App for AppImpl {
    fn authentication(&self) -> &dyn AuthenticationInteractor {
        self.authentication.as_ref()
    }

    fn location_searcher(&self) -> &dyn LocationSearchInteractor {
        self.location_searcher_interactor.as_ref()
    }
}

impl AppImpl {
    pub fn new<R: Repository + 'static>(
        repo: R,
        location_searcher: Arc<dyn LocationSearchApi>,
    ) -> Self {
        let repository = Arc::new(repo);
        let subscriber_authentication_checker =
            Arc::new(SubscriberResolverInteractorImpl::new(repository.clone()));
        let authentication_interactor = AuthenticationInteractorImpl::new(repository.clone());
        let location_searcher_interactor =
            LocationSearchInteractorImpl::new(location_searcher, subscriber_authentication_checker);

        Self {
            authentication: Arc::new(authentication_interactor),
            location_searcher_interactor: Arc::new(location_searcher_interactor),
        }
    }
}
